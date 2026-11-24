#define SDL_VIDEO_DRIVER_WAYLAND 1
#include <SDL.h>
#include <SDL_syswm.h>
#include <string.h>
#include <wayland-client-core.h>
#include <wayland-client-protocol.h>
#include <wayland-webos-shell-client-protocol.h>

enum { CLAIMED, NO_WAYLAND_INFO, NO_WEBOS_SHELL };

static struct wl_webos_shell *webos_shell;
static struct wl_webos_shell_surface *claimed_surface;
static struct wl_display *claimed_display;

static void shell_global(void *data, struct wl_registry *reg, uint32_t name, const char *iface, uint32_t version) {
    (void)data; (void)version;
    if (!strcmp(iface, "wl_webos_shell")) webos_shell = wl_registry_bind(reg, name, &wl_webos_shell_interface, 1);
}
static void shell_removed(void *data, struct wl_registry *reg, uint32_t name) { (void)data; (void)reg; (void)name; }

int lumen_claim_window(SDL_Window *window, int overlay) {
    SDL_SysWMinfo info;
    SDL_VERSION(&info.version);
    if (!SDL_GetWindowWMInfo(window, &info) || info.subsystem != SDL_SYSWM_WAYLAND) return NO_WAYLAND_INFO;
    struct wl_registry *registry = wl_display_get_registry(info.info.wl.display);
    static const struct wl_registry_listener listener = {shell_global, shell_removed};
    wl_registry_add_listener(registry, &listener, NULL);
    wl_display_roundtrip(info.info.wl.display);
    wl_registry_destroy(registry);
    if (!webos_shell) return NO_WEBOS_SHELL;
    struct wl_webos_shell_surface *shell_surface = wl_webos_shell_get_shell_surface(webos_shell, info.info.wl.surface);
    wl_webos_shell_surface_set_property(shell_surface, "_WEBOS_ACCESS_POLICY_KEYS_BACK", "true");
    if (overlay) {
        wl_webos_shell_surface_set_property(shell_surface, "_WEBOS_WINDOW_TYPE", "_WEBOS_WINDOW_TYPE_OVERLAY");
        wl_webos_shell_surface_set_property(shell_surface, "_WEBOS_ACCESS_POLICY_KEYS_EXIT", "true");
        wl_proxy_marshal((struct wl_proxy *)info.info.wl.surface, WL_SURFACE_SET_OPAQUE_REGION, NULL);
    }
    claimed_surface = shell_surface;
    claimed_display = info.info.wl.display;
    wl_display_flush(info.info.wl.display);
    return CLAIMED;
}

int lumen_shell_state(int state) {
    if (!claimed_surface) return 0;
    wl_webos_shell_surface_set_state(claimed_surface, (uint32_t)state);
    wl_display_flush(claimed_display);
    return 1;
}

#include <wayland-webos-foreign-client-protocol.h>

static struct wl_webos_foreign *webos_foreign;
static struct wl_compositor *compositor;
static struct wl_webos_exported *video_exported;
static struct wl_display *video_display;
static char video_window_id[64];

static void foreign_global(void *data, struct wl_registry *reg, uint32_t name, const char *iface, uint32_t version) {
    (void)data; (void)version;
    if (!strcmp(iface, "wl_webos_foreign")) webos_foreign = wl_registry_bind(reg, name, &wl_webos_foreign_interface, 1);
    if (!strcmp(iface, "wl_compositor")) compositor = wl_registry_bind(reg, name, &wl_compositor_interface, 1);
}

static void window_id_assigned(void *data, struct wl_webos_exported *exported, const char *window_id, uint32_t type) {
    (void)data; (void)exported; (void)type;
    strncpy(video_window_id, window_id, sizeof video_window_id - 1);
}

const char *lumen_export_video_window(SDL_Window *window) {
    if (video_window_id[0]) return video_window_id;
    SDL_SysWMinfo info;
    SDL_VERSION(&info.version);
    if (!SDL_GetWindowWMInfo(window, &info) || info.subsystem != SDL_SYSWM_WAYLAND) return "";
    video_display = info.info.wl.display;
    struct wl_registry *registry = wl_display_get_registry(video_display);
    static const struct wl_registry_listener listener = {foreign_global, shell_removed};
    wl_registry_add_listener(registry, &listener, NULL);
    wl_display_roundtrip(video_display);
    wl_registry_destroy(registry);
    if (!webos_foreign || !compositor) return "";
    video_exported = wl_webos_foreign_export_element(webos_foreign, info.info.wl.surface, WL_WEBOS_FOREIGN_WEBOS_EXPORTED_TYPE_VIDEO_OBJECT);
    static const struct wl_webos_exported_listener exported_listener = {window_id_assigned};
    wl_webos_exported_add_listener(video_exported, &exported_listener, NULL);
    for (int i = 0; i < 5 && !video_window_id[0]; i++) wl_display_roundtrip(video_display);
    return video_window_id;
}

int lumen_place_video(int source_w, int source_h, int x, int y, int w, int h) {
    if (!video_exported || !compositor) return 0;
    struct wl_region *source = wl_compositor_create_region(compositor);
    struct wl_region *destination = wl_compositor_create_region(compositor);
    wl_region_add(source, 0, 0, source_w, source_h);
    wl_region_add(destination, x, y, w, h);
    wl_webos_exported_set_exported_window(video_exported, source, destination);
    wl_region_destroy(source);
    wl_region_destroy(destination);
    wl_display_flush(video_display);
    return 1;
}
