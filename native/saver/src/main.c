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

#define TUNING_PATH "/tmp/nsaver-tune"

enum {
    FADE_IN_MS = 600, FADE_OUT_MS = 500, SHOW_ANYWAY_MS = 8000, SLOW_FRAME_MS = 20, IDLE_WAIT_MS = 250,
    OWN_SAVER_WAIT_MS = 3000, FETCH_LEAD_MS = 10000, WEATHER_CHECK_MS = 300000
};

static Uint32 crossfade_ms = 2400;
static Uint32 photo_ms;
static Uint32 started_at;
static struct wl_webos_shell *webos_shell;
static volatile int quitting;
static volatile Uint32 fetch_next_after;

static Settings settings;
static int frame_w = 1920, frame_h = 1080;

static struct {
    SDL_mutex *lock;
    SDL_cond *emptied;
    SDL_Surface *frame;
} slot;

static struct {
    SDL_mutex *lock;
    Weather reading;
    int version;
} sky;

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

static void showing_path(char *out, size_t n) { snprintf(out, n, "%s/showing", state_dir()); }

static void tell_watcher_showing(int showing) {
    char path[96], pid[16];
    showing_path(path, sizeof path);
    if (!showing) { unlink(path); return; }
    int n = snprintf(pid, sizeof pid, "%d\n", (int)getpid());
    write_file(path, pid, (size_t)n);
}

static void wait_while_lumen_own_saver_may_be_up(void) {
    for (int i = 0; i < OWN_SAVER_WAIT_MS / 100 && access("/tmp/lumen-ambient-on", F_OK) == 0; i++) SDL_Delay(100);
}

static int fetcher(void *unused) {
    (void)unused;
    int failures = 0;
    while (!quitting) {
        SDL_LockMutex(slot.lock);
        while (slot.frame && !quitting) SDL_CondWait(slot.emptied, slot.lock);
        SDL_UnlockMutex(slot.lock);
        while (!quitting && (Sint32)(fetch_next_after - SDL_GetTicks()) > 0) SDL_Delay(100);
        if (quitting) break;
        Picture pic;
        SDL_Surface *frame = NULL;
        if (picker_next(&pic)) {
            if (quitting) { picture_free(&pic); break; }
            Uint32 t0 = SDL_GetTicks();
            frame = compose_frame(&pic, frame_w, frame_h, settings_night(&settings));
            note("compose: %u ms", SDL_GetTicks() - t0);
            picture_free(&pic);
        }
        if (!frame) {
            failures++;
            for (int i = 0; i < (failures > 5 ? 300 : 50) && !quitting; i++) SDL_Delay(100);
            continue;
        }
        failures = 0;
        SDL_LockMutex(slot.lock);
        slot.frame = frame;
        SDL_UnlockMutex(slot.lock);
    }
    return 0;
}

static void publish(const Weather *w) {
    SDL_LockMutex(sky.lock);
    sky.reading = *w;
    sky.version++;
    SDL_UnlockMutex(sky.lock);
}

static int forecaster(void *unused) {
    (void)unused;
    Weather w;
    if (weather_peek(&w)) publish(&w);
    while (!quitting) {
        if (weather_read(&w)) publish(&w);
        for (int i = 0; i < WEATHER_CHECK_MS / 100 && !quitting; i++) SDL_Delay(100);
    }
    return 0;
}

static SDL_Texture *take_from_slot(SDL_Renderer *r, Uint32 next_is_due) {
    if (SDL_TryLockMutex(slot.lock) != 0) return NULL;
    SDL_Surface *frame = slot.frame;
    if (!frame) { SDL_UnlockMutex(slot.lock); return NULL; }
    slot.frame = NULL;
    fetch_next_after = next_is_due - (photo_ms < FETCH_LEAD_MS ? photo_ms : FETCH_LEAD_MS);
    SDL_CondSignal(slot.emptied);
    SDL_UnlockMutex(slot.lock);

    Uint32 t0 = SDL_GetTicks();
    SDL_Texture *t = SDL_CreateTextureFromSurface(r, frame);
    SDL_FreeSurface(frame);
    if (!t) note("upload: %s", SDL_GetError());
    else note("upload: %u ms", SDL_GetTicks() - t0);
    return t;
}

static void draw_frame(SDL_Renderer *r, SDL_Texture *t, float brightness, float alpha, SDL_BlendMode mode) {
    if (!t || alpha <= 0) return;
    Uint8 level = (Uint8)(brightness * 255);
    SDL_SetTextureBlendMode(t, mode);
    SDL_SetTextureColorMod(t, level, level, level);
    SDL_SetTextureAlphaMod(t, (Uint8)(alpha * 255));
    SDL_RenderCopy(r, t, NULL, NULL);
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
    tell_watcher_showing(0);
    if (in_lg_place) wait_while_lumen_own_saver_may_be_up();

    SDL_SetHint("SDL_WEBOS_ACCESS_POLICY_KEYS_BACK", "true");
    SDL_SetHint("SDL_WEBOS_ACCESS_POLICY_KEYS_EXIT", "true");
    SDL_SetHint(SDL_HINT_RENDER_SCALE_QUALITY, "linear");
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS) != 0) { note("SDL_Init: %s", SDL_GetError()); return 1; }
    IMG_Init(IMG_INIT_JPG | IMG_INIT_PNG);
    TTF_Init();
    http_init(&quitting);

    char *base = SDL_GetBasePath();
    char app_dir[256] = ".";
    if (base) { snprintf(app_dir, sizeof app_dir, "%s", base); SDL_free(base); }
    size_t n = strlen(app_dir);
    if (n > 1 && app_dir[n - 1] == '/') app_dir[n - 1] = 0;
    settings_load(app_dir, &settings);
    SaverConfig config;
    picker_init(app_dir, &config);
    photo_ms = (Uint32)config.seconds * 1000;
    Uint32 fade_in_ms = FADE_IN_MS, fade_out_ms = FADE_OUT_MS;
    if (settings.reduce_motion) fade_in_ms = fade_out_ms = crossfade_ms = 1;
    else read_tuning();

    SDL_DisplayMode mode = {0};
    if (SDL_GetCurrentDisplayMode(0, &mode) != 0 || !mode.w) { mode.w = 1920; mode.h = 1080; }
    frame_w = mode.w;
    frame_h = mode.h;

    slot.lock = SDL_CreateMutex();
    slot.emptied = SDL_CreateCond();
    sky.lock = SDL_CreateMutex();
    int wants_weather = weather_init(app_dir, &settings);
    int can_draw_status = status_init(app_dir, &settings);
    SDL_Thread *worker = SDL_CreateThread(fetcher, "fetcher", NULL);
    SDL_Thread *weatherman = wants_weather ? SDL_CreateThread(forecaster, "forecaster", NULL) : NULL;
    if (!can_draw_status) note("status: no font, so no clock");

    SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);
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
    SDL_BlendMode premultiplied_over_keeping_alpha = SDL_ComposeCustomBlendMode(
        SDL_BLENDFACTOR_ONE, SDL_BLENDFACTOR_ONE_MINUS_SRC_ALPHA, SDL_BLENDOPERATION_ADD,
        SDL_BLENDFACTOR_ZERO, SDL_BLENDFACTOR_ONE, SDL_BLENDOPERATION_ADD);
    SDL_SetRenderDrawBlendMode(r, SDL_BLENDMODE_NONE);
    SDL_SetRenderDrawColor(r, 0, 0, 0, 0);
    SDL_RenderClear(r);
    SDL_RenderPresent(r);

    SDL_Texture *band = NULL;
    int band_h = 0, band_minute = -1, sky_seen = 0, have_weather = 0;
    Weather weather;

    SDL_Texture *shown = NULL, *incoming = NULL;
    Uint32 shown_at = 0, swap_at = 0, leaving_at = 0, last_frame = 0;
    int redraw = 1;
    struct { int frames, slow; Uint32 slowest; } fade = {0, 0, 0};

    for (;;) {
        SDL_Event e;
        while (SDL_PollEvent(&e)) {
            if (e.type == SDL_QUIT) { note("close: quit"); goto done; }
            if ((e.type == SDL_KEYDOWN || e.type == SDL_MOUSEBUTTONDOWN) && !leaving_at) {
                note("key %d", e.type == SDL_KEYDOWN ? (int)e.key.keysym.sym : -1);
                leaving_at = SDL_GetTicks();
                tell_watcher_showing(0);
                if (!shown_at) { note("close: key before the first picture"); goto done; }
            }
            if (e.type == SDL_WINDOWEVENT) redraw = 1;
        }
        Uint32 now = SDL_GetTicks();

        if (!shown_at) {
            shown = take_from_slot(r, now + photo_ms);
            if (shown || now - started_at >= SHOW_ANYWAY_MS) {
                shown_at = SDL_GetTicks();
                swap_at = shown_at + photo_ms;
                tell_watcher_showing(1);
                note("first picture %u ms after start%s", shown_at - started_at, shown ? "" : " (none ready, on black)");
            } else {
                SDL_Delay(20);
                continue;
            }
            now = shown_at;
        }

        int fading_in = now - shown_at < fade_in_ms;
        if (!incoming && !leaving_at && !fading_in && (incoming = take_from_slot(r, (now > swap_at ? now : swap_at) + crossfade_ms + photo_ms))) {
            if (!settings.reduce_motion) read_tuning();
            now = SDL_GetTicks();
            if (!shown || now > swap_at) swap_at = now;
        }
        int crossfading = incoming && now >= swap_at;
        float cross = crossfading ? ease((float)(now - swap_at) / crossfade_ms) : 0;
        if (crossfading && now >= swap_at + crossfade_ms) {
            note("crossfade: %u ms, %d frames, %d slower than %d ms, slowest %u ms", crossfade_ms, fade.frames, fade.slow, SLOW_FRAME_MS, fade.slowest);
            fade.frames = fade.slow = 0;
            fade.slowest = 0;
            if (shown) SDL_DestroyTexture(shown);
            shown = incoming;
            incoming = NULL;
            swap_at = now + photo_ms;
            crossfading = 0;
            cross = 0;
            redraw = 1;
        }

        if (can_draw_status && !((crossfading || fading_in) && band)) {
            time_t wall = time(NULL);
            int minute = localtime(&wall)->tm_min;
            SDL_LockMutex(sky.lock);
            int sky_changed = sky.version != sky_seen;
            if (sky_changed) { weather = sky.reading; sky_seen = sky.version; have_weather = 1; }
            SDL_UnlockMutex(sky.lock);
            if (minute != band_minute || sky_changed) {
                Uint32 t0 = SDL_GetTicks();
                SDL_Surface *drawn = status_draw(sw, sh, have_weather ? &weather : NULL, wall);
                if (drawn) {
                    if (band) SDL_DestroyTexture(band);
                    band = SDL_CreateTextureFromSurface(r, drawn);
                    band_h = drawn->h;
                    SDL_FreeSurface(drawn);
                    if (band_minute < 0 || sky_changed) note("status: drawn in %u ms", SDL_GetTicks() - t0);
                }
                band_minute = minute;
                redraw = 1;
                now = SDL_GetTicks();
            }
        }

        float scene = ease((float)(now - shown_at) / fade_in_ms);
        if (leaving_at) {
            scene *= 1 - ease((float)(now - leaving_at) / fade_out_ms);
            if (now - leaving_at >= fade_out_ms) { note("close: key"); goto done; }
        }
        int moving = crossfading || leaving_at || now - shown_at < fade_in_ms + 100;
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
            draw_frame(r, shown, scene, 1, SDL_BLENDMODE_NONE);
            if (crossfading) draw_frame(r, incoming, scene, cross, over_keeping_alpha);
        } else {
            draw_frame(r, shown, 1, 1, SDL_BLENDMODE_NONE);
            if (crossfading) draw_frame(r, incoming, 1, cross, over_keeping_alpha);
        }
        if (band) {
            SDL_Rect dst = {0, sh - band_h, sw, band_h};
            Uint8 level = (Uint8)(scene * 255);
            SDL_SetTextureBlendMode(band, premultiplied_over_keeping_alpha);
            SDL_SetTextureColorMod(band, level, level, level);
            SDL_SetTextureAlphaMod(band, 255);
            SDL_RenderCopy(r, band, NULL, &dst);
        }
        if (scene < 1) {
            SDL_SetRenderDrawBlendMode(r, window_alpha_only);
            SDL_SetRenderDrawColor(r, 0, 0, 0, (Uint8)(scene * 255));
            SDL_RenderFillRect(r, NULL);
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
    tell_watcher_showing(0);
    SDL_HideWindow(window);
    SDL_LockMutex(slot.lock);
    SDL_CondSignal(slot.emptied);
    SDL_UnlockMutex(slot.lock);
    SDL_WaitThread(worker, NULL);
    if (weatherman) SDL_WaitThread(weatherman, NULL);
    if (slot.frame) SDL_FreeSurface(slot.frame);
    note("RSS %ld kB, peak %ld kB", status_kb("VmRSS"), status_kb("VmHWM"));
    if (shown) SDL_DestroyTexture(shown);
    if (incoming) SDL_DestroyTexture(incoming);
    if (band) SDL_DestroyTexture(band);
    status_quit();
    SDL_DestroyRenderer(r);
    SDL_DestroyWindow(window);
    TTF_Quit();
    IMG_Quit();
    SDL_Quit();
    return 0;
}
