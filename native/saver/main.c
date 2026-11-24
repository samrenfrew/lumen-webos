#define SDL_VIDEO_DRIVER_WAYLAND 1
#include <SDL.h>
#include <SDL_image.h>
#include <SDL_ttf.h>
#include <SDL_syswm.h>
#include <wayland-client-core.h>
#include <wayland-client-protocol.h>
#include <wayland-webos-shell-client-protocol.h>
#include <dirent.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define LOG_PATH "/tmp/nsaver.log"
#define PHOTO_DIR "/media/developer/apps/usr/palm/applications/com.samrenfrew.lumen/wallpapers"
#define CLOCK_FONT "/usr/share/fonts/LG_Smart_UI-Light.ttf"
#define MAX_PHOTOS 64

#define TUNING_PATH "/tmp/nsaver-tune"

enum { FADE_IN_MS = 600, FADE_OUT_MS = 500, PRELOAD_MS = 2000, SLOW_FRAME_MS = 20, IDLE_WAIT_MS = 250 };

static Uint32 crossfade_ms = 1600;
static Uint32 photo_ms = 8000;

static Uint32 started_at;
static struct wl_webos_shell *webos_shell;

static void note(const char *fmt, ...) {
    FILE *f = fopen(LOG_PATH, "a");
    if (!f) return;
    char stamp[32];
    time_t now = time(NULL);
    strftime(stamp, sizeof stamp, "%Y-%m-%dT%H:%M:%SZ", gmtime(&now));
    fprintf(f, "%s nsaver ", stamp);
    va_list ap;
    va_start(ap, fmt);
    vfprintf(f, fmt, ap);
    va_end(ap);
    fputc('\n', f);
    fclose(f);
}

static long status_kb(const char *field) {
    FILE *f = fopen("/proc/self/status", "r");
    if (!f) return -1;
    char line[128];
    long kb = -1;
    size_t n = strlen(field);
    while (fgets(line, sizeof line, f))
        if (!strncmp(line, field, n)) { kb = strtol(line + n + 1, NULL, 10); break; }
    fclose(f);
    return kb;
}

static void shell_global(void *data, struct wl_registry *reg, uint32_t name, const char *iface, uint32_t version) {
    (void)data; (void)version;
    if (!strcmp(iface, "wl_webos_shell"))
        webos_shell = wl_registry_bind(reg, name, &wl_webos_shell_interface, 1);
}
static void shell_removed(void *data, struct wl_registry *reg, uint32_t name) { (void)data; (void)reg; (void)name; }

static void claim_window(SDL_Window *window, const char *window_type) {
    SDL_SysWMinfo info;
    SDL_VERSION(&info.version);
    if (!SDL_GetWindowWMInfo(window, &info) || info.subsystem != SDL_SYSWM_WAYLAND) {
        note("window: no wayland info, type left as SDL set it");
        return;
    }
    struct wl_registry *registry = wl_display_get_registry(info.info.wl.display);
    static const struct wl_registry_listener listener = {shell_global, shell_removed};
    wl_registry_add_listener(registry, &listener, NULL);
    wl_display_roundtrip(info.info.wl.display);
    wl_registry_destroy(registry);
    if (!webos_shell) {
        note("window: no wl_webos_shell");
        return;
    }
    struct wl_webos_shell_surface *shell_surface = wl_webos_shell_get_shell_surface(webos_shell, info.info.wl.surface);
    wl_webos_shell_surface_set_property(shell_surface, "_WEBOS_WINDOW_TYPE", window_type);
    wl_webos_shell_surface_set_property(shell_surface, "_WEBOS_ACCESS_POLICY_KEYS_BACK", "true");
    wl_webos_shell_surface_set_property(shell_surface, "_WEBOS_ACCESS_POLICY_KEYS_EXIT", "true");
    wl_proxy_marshal((struct wl_proxy *)info.info.wl.surface, WL_SURFACE_SET_OPAQUE_REGION, NULL);
    wl_display_flush(info.info.wl.display);
    note("window: %s", window_type);
}

static int is_photo(const char *name) {
    size_t n = strlen(name);
    return n > 4 && !strcmp(name + n - 4, ".jpg") && !strstr(name, "-bg.");
}

static int list_photos(char paths[][256]) {
    DIR *d = opendir(PHOTO_DIR);
    if (!d) return 0;
    int count = 0;
    struct dirent *e;
    while ((e = readdir(d)) && count < MAX_PHOTOS)
        if (is_photo(e->d_name))
            snprintf(paths[count++], 256, "%s/%s", PHOTO_DIR, e->d_name);
    closedir(d);
    return count;
}

static SDL_Texture *load_photo(SDL_Renderer *r, const char *path) {
    Uint32 t0 = SDL_GetTicks();
    SDL_Surface *s = IMG_Load(path);
    if (!s) { note("photo: %s did not load: %s", path, IMG_GetError()); return NULL; }
    Uint32 t1 = SDL_GetTicks();
    SDL_Texture *t = SDL_CreateTextureFromSurface(r, s);
    SDL_FreeSurface(s);
    note("photo: %s decoded in %u ms, uploaded in %u ms", strrchr(path, '/') + 1, t1 - t0, SDL_GetTicks() - t1);
    return t;
}

static SDL_Texture *render_clock(SDL_Renderer *r, TTF_Font *font, int *w, int *h) {
    char text[8];
    time_t now = time(NULL);
    strftime(text, sizeof text, "%H:%M", localtime(&now));
    SDL_Color white = {255, 255, 255, 255};
    SDL_Surface *s = TTF_RenderUTF8_Blended(font, text, white);
    if (!s) return NULL;
    SDL_Texture *t = SDL_CreateTextureFromSurface(r, s);
    *w = s->w;
    *h = s->h;
    SDL_FreeSurface(s);
    return t;
}

static void read_tuning(void) {
    FILE *f = fopen(TUNING_PATH, "r");
    if (!f) return;
    char line[64];
    unsigned v;
    while (fgets(line, sizeof line, f)) {
        if (sscanf(line, "crossfade=%u", &v) == 1 && v >= 100 && v <= 10000) crossfade_ms = v;
        if (sscanf(line, "photo=%u", &v) == 1 && v >= 3000 && v <= 600000) photo_ms = v;
    }
    fclose(f);
}

static float ease(float x) { return x < 0 ? 0 : x > 1 ? 1 : x * x * (3 - 2 * x); }

static void draw_cover(SDL_Renderer *r, SDL_Texture *t, int sw, int sh, float alpha, SDL_BlendMode mode) {
    if (!t || alpha <= 0) return;
    int tw, th;
    SDL_QueryTexture(t, NULL, NULL, &tw, &th);
    float scale = (float)sw / tw > (float)sh / th ? (float)sw / tw : (float)sh / th;
    SDL_Rect dst = {(int)(sw - tw * scale) / 2, (int)(sh - th * scale) / 2, (int)(tw * scale), (int)(th * scale)};
    SDL_SetTextureBlendMode(t, mode);
    SDL_SetTextureAlphaMod(t, (Uint8)(alpha * 255));
    SDL_RenderCopy(r, t, NULL, &dst);
}

int main(int argc, char **argv) {
    started_at = SDL_GetTicks();
    setenv("EGL_PLATFORM", "wayland", 0);
    setenv("XDG_RUNTIME_DIR", "/tmp/xdg", 0);
    const char *window_type = argc > 1 && strstr(argv[1], "screensaver") ? "_WEBOS_WINDOW_TYPE_SCREENSAVER" : "_WEBOS_WINDOW_TYPE_OVERLAY";
    note("open: args %s", argc > 1 ? argv[1] : "(none)");

    SDL_SetHint("SDL_WEBOS_ACCESS_POLICY_KEYS_BACK", "true");
    SDL_SetHint("SDL_WEBOS_ACCESS_POLICY_KEYS_EXIT", "true");
    SDL_SetHint(SDL_HINT_RENDER_SCALE_QUALITY, "linear");
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS) != 0) { note("SDL_Init: %s", SDL_GetError()); return 1; }
    IMG_Init(IMG_INIT_JPG);
    TTF_Init();
    SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);

    SDL_DisplayMode mode = {0};
    if (SDL_GetCurrentDisplayMode(0, &mode) != 0 || !mode.w) { mode.w = 1920; mode.h = 1080; }
    SDL_Window *window = SDL_CreateWindow("Lumen Screensaver", 0, 0, mode.w, mode.h, SDL_WINDOW_FULLSCREEN | SDL_WINDOW_SHOWN);
    if (!window) { note("window: %s", SDL_GetError()); return 1; }
    claim_window(window, window_type);
    SDL_Renderer *r = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC);
    if (!r) { note("renderer: %s", SDL_GetError()); return 1; }
    SDL_RendererInfo ri;
    SDL_GetRendererInfo(r, &ri);
    int sw, sh;
    SDL_GetRendererOutputSize(r, &sw, &sh);
    note("renderer: %s, %dx%d, %u ms after start", ri.name, sw, sh, SDL_GetTicks() - started_at);

    SDL_BlendMode window_alpha_only = SDL_ComposeCustomBlendMode(
        SDL_BLENDFACTOR_ZERO, SDL_BLENDFACTOR_ONE, SDL_BLENDOPERATION_ADD,
        SDL_BLENDFACTOR_ONE, SDL_BLENDFACTOR_ZERO, SDL_BLENDOPERATION_ADD);
    SDL_BlendMode over_keeping_alpha = SDL_ComposeCustomBlendMode(
        SDL_BLENDFACTOR_SRC_ALPHA, SDL_BLENDFACTOR_ONE_MINUS_SRC_ALPHA, SDL_BLENDOPERATION_ADD,
        SDL_BLENDFACTOR_ZERO, SDL_BLENDFACTOR_ONE, SDL_BLENDOPERATION_ADD);
    SDL_SetRenderDrawBlendMode(r, SDL_BLENDMODE_NONE);
    SDL_SetRenderDrawColor(r, 0, 0, 0, 0);
    SDL_RenderClear(r);
    SDL_RenderPresent(r);

    static char photos[MAX_PHOTOS][256];
    int count = list_photos(photos);
    note("photos: %d in %s", count, PHOTO_DIR);
    int next = 0;
    SDL_Texture *shown = count ? load_photo(r, photos[next++ % count]) : NULL;
    SDL_Texture *incoming = NULL;

    TTF_Font *font = TTF_OpenFont(CLOCK_FONT, 120);
    if (!font) note("font: %s", TTF_GetError());
    int clock_w = 0, clock_h = 0, clock_minute = -1;
    SDL_Texture *clock = NULL;

    Uint32 shown_at = SDL_GetTicks();
    read_tuning();
    Uint32 swap_at = shown_at + photo_ms;
    Uint32 leaving_at = 0;
    Uint32 last_frame = shown_at;
    int first_frame_logged = 0, redraw = 1;
    struct { int frames, slow; Uint32 slowest; } fade = {0, 0, 0};
    note("first photo ready %u ms after start, RSS %ld kB", shown_at - started_at, status_kb("VmRSS"));

    for (;;) {
        SDL_Event e;
        while (SDL_PollEvent(&e)) {
            if (e.type == SDL_QUIT) { note("close: quit"); goto done; }
            if (e.type == SDL_KEYDOWN && !leaving_at) {
                note("key %d", e.key.keysym.sym);
                leaving_at = SDL_GetTicks();
            }
            if (e.type == SDL_WINDOWEVENT) redraw = 1;
        }
        Uint32 now = SDL_GetTicks();

        time_t wall = time(NULL);
        int minute = localtime(&wall)->tm_min;
        if (font && minute != clock_minute) {
            if (clock) SDL_DestroyTexture(clock);
            clock = render_clock(r, font, &clock_w, &clock_h);
            clock_minute = minute;
            redraw = 1;
        }

        if (!incoming && count > 1 && now + PRELOAD_MS >= swap_at) {
            read_tuning();
            incoming = load_photo(r, photos[next++ % count]);
            now = SDL_GetTicks();
            if (now > swap_at) swap_at = now;
        }
        int crossfading = incoming && now >= swap_at;
        float cross = crossfading ? ease((float)(now - swap_at) / crossfade_ms) : 0;
        if (crossfading && now >= swap_at + crossfade_ms) {
            note("crossfade: %u ms, %d frames, %d slower than %d ms, slowest %u ms", crossfade_ms, fade.frames, fade.slow, SLOW_FRAME_MS, fade.slowest);
            fade.frames = fade.slow = 0;
            fade.slowest = 0;
            SDL_DestroyTexture(shown);
            shown = incoming;
            incoming = NULL;
            swap_at = now + photo_ms;
            crossfading = 0;
            cross = 0;
            redraw = 1;
        }

        float scene = ease((float)(now - shown_at) / FADE_IN_MS);
        if (leaving_at) {
            scene *= 1 - ease((float)(now - leaving_at) / FADE_OUT_MS);
            if (now - leaving_at >= FADE_OUT_MS) { note("close: key"); goto done; }
        }
        int moving = crossfading || leaving_at || now - shown_at < FADE_IN_MS + 100;
        if (!moving && !redraw) {
            SDL_WaitEventTimeout(NULL, IDLE_WAIT_MS);
            last_frame = SDL_GetTicks();
            continue;
        }
        redraw = 0;

        if (scene < 1) {
            SDL_SetRenderDrawColor(r, 0, 0, 0, (Uint8)(scene * 255));
            SDL_RenderClear(r);
            draw_cover(r, shown, sw, sh, scene, SDL_BLENDMODE_BLEND);
            if (crossfading) draw_cover(r, incoming, sw, sh, scene * cross, SDL_BLENDMODE_BLEND);
            SDL_SetRenderDrawBlendMode(r, window_alpha_only);
            SDL_SetRenderDrawColor(r, 0, 0, 0, (Uint8)(scene * 255));
            SDL_RenderFillRect(r, NULL);
            SDL_SetRenderDrawBlendMode(r, SDL_BLENDMODE_NONE);
        } else {
            draw_cover(r, shown, sw, sh, 1, SDL_BLENDMODE_NONE);
            if (crossfading) draw_cover(r, incoming, sw, sh, cross, over_keeping_alpha);
        }
        if (clock) {
            SDL_Rect dst = {96, sh - 72 - clock_h, clock_w, clock_h};
            SDL_SetTextureAlphaMod(clock, (Uint8)(scene * 255));
            SDL_RenderCopy(r, clock, NULL, &dst);
        }
        SDL_RenderPresent(r);

        Uint32 after = SDL_GetTicks();
        if (!first_frame_logged) { note("first frame %u ms after start", after - started_at); first_frame_logged = 1; }
        if (crossfading) {
            Uint32 gap = after - last_frame;
            fade.frames++;
            if (gap > SLOW_FRAME_MS) fade.slow++;
            if (gap > fade.slowest) fade.slowest = gap;
        }
        last_frame = after;
    }

done:
    note("RSS %ld kB, peak %ld kB", status_kb("VmRSS"), status_kb("VmHWM"));
    if (shown) SDL_DestroyTexture(shown);
    if (incoming) SDL_DestroyTexture(incoming);
    if (clock) SDL_DestroyTexture(clock);
    if (font) TTF_CloseFont(font);
    SDL_DestroyRenderer(r);
    SDL_DestroyWindow(window);
    TTF_Quit();
    IMG_Quit();
    SDL_Quit();
    return 0;
}
