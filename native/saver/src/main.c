#define SDL_VIDEO_DRIVER_WAYLAND 1
#include <SDL.h>
#include <SDL_image.h>
#include <SDL_ttf.h>
#include <SDL_syswm.h>
#include <wayland-client-core.h>
#include <wayland-client-protocol.h>
#include <wayland-webos-shell-client-protocol.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include "compose.h"
#include "http.h"
#include "picker.h"
#include "settings.h"
#include "status.h"
#include "weather.h"
#include "util.h"

#define CLOCK_FONT "/usr/share/fonts/LG_Smart_UI-Light.ttf"
#define TUNING_PATH "/tmp/nsaver-tune"

enum {
    FADE_IN_MS = 600, FADE_OUT_MS = 500, SHOW_ANYWAY_MS = 8000, SLOW_FRAME_MS = 20, IDLE_WAIT_MS = 250
};

static Uint32 crossfade_ms = 2400;
static Uint32 photo_ms;
static Uint32 started_at;
static struct wl_webos_shell *webos_shell;
static volatile int quitting;

typedef struct {
    int count;
    SDL_Texture *tex[2];
    float x[2], y[2];
} Shown;

static struct {
    SDL_mutex *lock;
    SDL_cond *emptied;
    Picture pic;
    int full;
} slot;

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

static int lumen_own_saver_up(void) {
    FILE *flag = fopen("/tmp/lumen-ambient-on", "r");
    if (!flag) return 0;
    fclose(flag);
    FILE *p = popen("luna-send -n 1 -w 3000 luna://com.webos.applicationManager/getForegroundAppInfo '{}' 2>/dev/null", "r");
    if (!p) return 0;
    char out[512] = {0};
    size_t n = fread(out, 1, sizeof out - 1, p);
    out[n] = 0;
    pclose(p);
    return strstr(out, "\"com.samrenfrew.lumen\"") || strstr(out, "\"com.samrenfrew.lumen.dev\"");
}

static void end_tv_screensaver(void) {
    int rc = system("luna-send -n 1 -w 4000 luna://com.webos.service.tvpower/power/turnOnScreen '{}' >/dev/null 2>&1");
    note("turnOnScreen: %d", rc);
}

static int fetcher(void *unused) {
    (void)unused;
    int failures = 0;
    while (!quitting) {
        SDL_LockMutex(slot.lock);
        while (slot.full && !quitting) SDL_CondWait(slot.emptied, slot.lock);
        SDL_UnlockMutex(slot.lock);
        if (quitting) break;
        Picture pic;
        if (!picker_next(&pic)) {
            failures++;
            for (int i = 0; i < (failures > 5 ? 300 : 50) && !quitting; i++) SDL_Delay(100);
            continue;
        }
        failures = 0;
        SDL_LockMutex(slot.lock);
        slot.pic = pic;
        slot.full = 1;
        SDL_UnlockMutex(slot.lock);
    }
    return 0;
}

static int take_from_slot(SDL_Renderer *r, Shown *out) {
    if (SDL_TryLockMutex(slot.lock) != 0) return 0;
    if (!slot.full) { SDL_UnlockMutex(slot.lock); return 0; }
    Picture pic = slot.pic;
    picker_shown(&pic);
    slot.full = 0;
    SDL_CondSignal(slot.emptied);
    SDL_UnlockMutex(slot.lock);

    Uint32 t0 = SDL_GetTicks();
    memset(out, 0, sizeof *out);
    out->count = pic.count;
    for (int i = 0; i < pic.count; i++) {
        out->tex[i] = SDL_CreateTextureFromSurface(r, pic.img[i]);
        out->x[i] = pic.x[i];
        out->y[i] = pic.y[i];
    }
    picture_free(&pic);
    note("upload: %u ms", SDL_GetTicks() - t0);
    return out->count > 0 && out->tex[0];
}

static void shown_free(Shown *s) {
    for (int i = 0; i < 2; i++) if (s->tex[i]) SDL_DestroyTexture(s->tex[i]);
    memset(s, 0, sizeof *s);
}

static void draw_texture(SDL_Renderer *r, SDL_Texture *t, SDL_Rect box, int fit_width, float px, float py, float alpha, SDL_BlendMode mode) {
    int tw, th;
    SDL_QueryTexture(t, NULL, NULL, &tw, &th);
    float scale = fit_width ? (float)box.w / tw : ((float)box.w / tw > (float)box.h / th ? (float)box.w / tw : (float)box.h / th);
    int dw = (int)(tw * scale + 0.5f), dh = (int)(th * scale + 0.5f);
    SDL_Rect dst = {box.x + (int)((box.w - dw) * px), box.y + (int)((box.h - dh) * py), dw, dh};
    SDL_RenderSetClipRect(r, &box);
    SDL_SetTextureBlendMode(t, mode);
    SDL_SetTextureAlphaMod(t, (Uint8)(alpha * 255));
    SDL_RenderCopy(r, t, NULL, &dst);
    SDL_RenderSetClipRect(r, NULL);
}

static void draw_shown(SDL_Renderer *r, const Shown *s, int sw, int sh, float alpha, SDL_BlendMode mode) {
    if (!s->count || alpha <= 0) return;
    if (s->count == 1) {
        draw_texture(r, s->tex[0], (SDL_Rect){0, 0, sw, sh}, 0, s->x[0], s->y[0], alpha, mode);
        return;
    }
    SDL_SetRenderDrawBlendMode(r, mode);
    SDL_SetRenderDrawColor(r, 0, 0, 0, (Uint8)(alpha * 255));
    SDL_RenderFillRect(r, NULL);
    int w = PAIR_WIDTH * sw / 1920;
    draw_texture(r, s->tex[0], (SDL_Rect){0, 0, w, sh}, 1, 0, s->y[0], alpha, mode);
    draw_texture(r, s->tex[1], (SDL_Rect){sw - w, 0, w, sh}, 1, 0, s->y[1], alpha, mode);
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

static int pick_only(const char *app_dir, int count) {
    SDL_Init(0);
    IMG_Init(IMG_INIT_JPG);
    http_init(&quitting);
    SaverConfig config;
    picker_init(app_dir, &config);
    for (int i = 0; i < count; i++) {
        Picture pic;
        if (!picker_next(&pic)) { printf("pick %d: none\n", i); continue; }
        printf("pick %d: %s, %d photo(s), %dx%d, at %.2f %.2f / %.2f %.2f\n", i, pic.id, pic.count, pic.img[0]->w, pic.img[0]->h, pic.x[0], pic.y[0], pic.x[1], pic.y[1]);
        picker_shown(&pic);
        picture_free(&pic);
    }
    printf("RSS %ld kB, peak %ld kB, log in %s/log\n", status_kb("VmRSS"), status_kb("VmHWM"), state_dir());
    return 0;
}

static float ease(float x) { return x < 0 ? 0 : x > 1 ? 1 : x * x * (3 - 2 * x); }

int main(int argc, char **argv) {
    started_at = SDL_GetTicks();
    setenv("EGL_PLATFORM", "wayland", 0);
    setenv("XDG_RUNTIME_DIR", "/tmp/xdg", 0);
    util_init();
    srand((unsigned)time(NULL) ^ (unsigned)getpid());
    if (argc > 2 && !strcmp(argv[1], "--pick")) return pick_only(argv[2], argc > 3 ? atoi(argv[3]) : 3);
    if (argc > 1 && (!strcmp(argv[1], "--weather") || !strcmp(argv[1], "--status") || !strcmp(argv[1], "--compose"))) {
        SDL_Init(0);
        IMG_Init(IMG_INIT_JPG | IMG_INIT_PNG);
        TTF_Init();
        http_init(&quitting);
        return argv[1][2] == 'w' ? weather_check(argc - 2, argv + 2) : argv[1][2] == 's' ? status_check(argc - 2, argv + 2) : compose_check(argc - 2, argv + 2);
    }
    int in_lg_place = argc > 1 && strstr(argv[1], "com.webos.app.screensaver") != NULL;
    const char *window_type = in_lg_place ? "_WEBOS_WINDOW_TYPE_SCREENSAVER" : "_WEBOS_WINDOW_TYPE_OVERLAY";
    note("open: %s", in_lg_place ? "in LG's screensaver's place" : "as an overlay");
    if (in_lg_place && lumen_own_saver_up()) {
        note("open: Lumen's own screensaver is up, ending");
        end_tv_screensaver();
        return 0;
    }

    SDL_SetHint("SDL_WEBOS_ACCESS_POLICY_KEYS_BACK", "true");
    SDL_SetHint("SDL_WEBOS_ACCESS_POLICY_KEYS_EXIT", "true");
    SDL_SetHint(SDL_HINT_RENDER_SCALE_QUALITY, "linear");
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS) != 0) { note("SDL_Init: %s", SDL_GetError()); return 1; }
    IMG_Init(IMG_INIT_JPG);
    TTF_Init();
    http_init(&quitting);

    char *base = SDL_GetBasePath();
    char app_dir[256] = ".";
    if (base) { snprintf(app_dir, sizeof app_dir, "%s", base); SDL_free(base); }
    size_t n = strlen(app_dir);
    if (n > 1 && app_dir[n - 1] == '/') app_dir[n - 1] = 0;
    SaverConfig config;
    picker_init(app_dir, &config);
    photo_ms = (Uint32)config.seconds * 1000;
    read_tuning();

    slot.lock = SDL_CreateMutex();
    slot.emptied = SDL_CreateCond();
    SDL_Thread *worker = SDL_CreateThread(fetcher, "fetcher", NULL);

    SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);
    SDL_DisplayMode mode = {0};
    if (SDL_GetCurrentDisplayMode(0, &mode) != 0 || !mode.w) { mode.w = 1920; mode.h = 1080; }
    SDL_Window *window = SDL_CreateWindow("Lumen Screensaver", 0, 0, mode.w, mode.h, SDL_WINDOW_FULLSCREEN | SDL_WINDOW_SHOWN);
    if (!window) { note("window: %s", SDL_GetError()); return 1; }
    claim_window(window, window_type);
    SDL_Renderer *r = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC);
    if (!r) { note("renderer: %s", SDL_GetError()); return 1; }
    int sw, sh;
    SDL_GetRendererOutputSize(r, &sw, &sh);
    note("renderer: %dx%d, %u ms after start", sw, sh, SDL_GetTicks() - started_at);

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

    TTF_Font *font = TTF_OpenFont(CLOCK_FONT, 120);
    if (!font) note("font: %s", TTF_GetError());
    int clock_w = 0, clock_h = 0, clock_minute = -1;
    SDL_Texture *clock = NULL;

    Shown shown = {0}, incoming = {0};
    Uint32 shown_at = 0, swap_at = 0, leaving_at = 0, last_frame = 0;
    int redraw = 1;
    struct { int frames, slow; Uint32 slowest; } fade = {0, 0, 0};

    for (;;) {
        SDL_Event e;
        while (SDL_PollEvent(&e)) {
            if (e.type == SDL_QUIT) { note("close: quit"); goto done; }
            if (e.type == SDL_KEYDOWN && !leaving_at) {
                note("key %d", e.key.keysym.sym);
                leaving_at = SDL_GetTicks();
                if (!shown_at) { note("close: key before the first picture"); goto done; }
            }
            if (e.type == SDL_WINDOWEVENT) redraw = 1;
        }
        Uint32 now = SDL_GetTicks();

        if (!shown_at) {
            if (take_from_slot(r, &shown) || now - started_at >= SHOW_ANYWAY_MS) {
                shown_at = SDL_GetTicks();
                swap_at = shown_at + photo_ms;
                note("first picture %u ms after start%s", shown_at - started_at, shown.count ? "" : " (none ready, on black)");
            } else {
                SDL_Delay(20);
                continue;
            }
            now = shown_at;
        }

        time_t wall = time(NULL);
        int minute = localtime(&wall)->tm_min;
        if (font && minute != clock_minute) {
            if (clock) SDL_DestroyTexture(clock);
            clock = render_clock(r, font, &clock_w, &clock_h);
            clock_minute = minute;
            redraw = 1;
        }

        if (!incoming.count && !leaving_at && take_from_slot(r, &incoming)) {
            read_tuning();
            now = SDL_GetTicks();
            if (!shown.count) swap_at = now;
        }
        int crossfading = incoming.count && now >= swap_at;
        float cross = crossfading ? ease((float)(now - swap_at) / crossfade_ms) : 0;
        if (crossfading && now >= swap_at + crossfade_ms) {
            note("crossfade: %u ms, %d frames, %d slower than %d ms, slowest %u ms", crossfade_ms, fade.frames, fade.slow, SLOW_FRAME_MS, fade.slowest);
            fade.frames = fade.slow = 0;
            fade.slowest = 0;
            shown_free(&shown);
            shown = incoming;
            memset(&incoming, 0, sizeof incoming);
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

        SDL_SetRenderDrawBlendMode(r, SDL_BLENDMODE_NONE);
        SDL_SetRenderDrawColor(r, 0, 0, 0, (Uint8)(scene * 255));
        SDL_RenderClear(r);
        if (scene < 1) {
            draw_shown(r, &shown, sw, sh, scene, SDL_BLENDMODE_BLEND);
            if (crossfading) draw_shown(r, &incoming, sw, sh, scene * cross, SDL_BLENDMODE_BLEND);
            SDL_SetRenderDrawBlendMode(r, window_alpha_only);
            SDL_SetRenderDrawColor(r, 0, 0, 0, (Uint8)(scene * 255));
            SDL_RenderFillRect(r, NULL);
        } else {
            draw_shown(r, &shown, sw, sh, 1, SDL_BLENDMODE_NONE);
            if (crossfading) draw_shown(r, &incoming, sw, sh, cross, over_keeping_alpha);
        }
        SDL_SetRenderDrawBlendMode(r, SDL_BLENDMODE_NONE);
        if (clock) {
            SDL_Rect dst = {96, sh - 72 - clock_h, clock_w, clock_h};
            SDL_SetTextureBlendMode(clock, over_keeping_alpha);
            SDL_SetTextureAlphaMod(clock, (Uint8)(scene * 255));
            SDL_RenderCopy(r, clock, NULL, &dst);
        }
        SDL_RenderPresent(r);

        Uint32 after = SDL_GetTicks();
        if (crossfading) {
            Uint32 gap = after - last_frame;
            fade.frames++;
            if (gap > SLOW_FRAME_MS) fade.slow++;
            if (gap > fade.slowest) fade.slowest = gap;
        }
        last_frame = after;
    }

done:
    quitting = 1;
    if (in_lg_place) end_tv_screensaver();
    SDL_LockMutex(slot.lock);
    SDL_CondSignal(slot.emptied);
    SDL_UnlockMutex(slot.lock);
    SDL_WaitThread(worker, NULL);
    if (slot.full) picture_free(&slot.pic);
    note("RSS %ld kB, peak %ld kB", status_kb("VmRSS"), status_kb("VmHWM"));
    shown_free(&shown);
    shown_free(&incoming);
    if (clock) SDL_DestroyTexture(clock);
    if (font) TTF_CloseFont(font);
    SDL_DestroyRenderer(r);
    SDL_DestroyWindow(window);
    TTF_Quit();
    IMG_Quit();
    SDL_Quit();
    return 0;
}
