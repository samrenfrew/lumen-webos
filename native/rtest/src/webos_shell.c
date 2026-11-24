#define SDL_VIDEO_DRIVER_WAYLAND 1
#include <SDL.h>
#include <SDL_syswm.h>
#include <string.h>
#include <wayland-client-core.h>
#include <wayland-client-protocol.h>
#include <wayland-webos-shell-client-protocol.h>

enum { CLAIMED, NO_WAYLAND_INFO, NO_WEBOS_SHELL };

static struct wl_webos_shell *webos_shell;

static void shell_global(void *data, struct wl_registry *reg, uint32_t name, const char *iface, uint32_t version) {
    (void)data; (void)version;
    if (!strcmp(iface, "wl_webos_shell")) webos_shell = wl_registry_bind(reg, name, &wl_webos_shell_interface, 1);
}
static void shell_removed(void *data, struct wl_registry *reg, uint32_t name) { (void)data; (void)reg; (void)name; }

int lumen_claim_window(SDL_Window *window) {
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
    wl_display_flush(info.info.wl.display);
    return CLAIMED;
}
