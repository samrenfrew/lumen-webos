# Backlog

## Runtime icon pack from a URL
**Origin:** PR 4 (closed 2026-10-27) generated a matching tile for every app and baked it into the build. Sam declined it: a fixed app list means icon PRs to maintain and a bigger install. The icons the TV supplies are low quality, though.
**Idea:** One Settings field, "Icon pack URL", points at a folder on any web server with an `index.json` that maps app ids or titles to image files. Lumen fetches the index at launch, and `packIcon` checks it first. Unlisted apps and an unreachable server fall back to today's icons.
**Context for next agent:** The build-time pack already exists (`app/vite.config.ts:54`, `packIcon` in `app/src/lib/apps.ts:46`); this is its runtime version. Lumen ships the loader and the documented `index.json` format only, never icons. Tiles stay flat images; any gradient comes from Lumen's CSS, as the focus overlay already does. Decide: whether to cache the images or rely on the browser, and how to enter a URL without painful remote typing (a QR code or phone input could come later). Works on a non-rooted TV because it needs only the network.

## Video on Lumen's screensaver
**Origin:** Screensaver without Immich (2026-10-27). The simplest version shows wallpaper pictures or Immich photos only.
**Idea:** Add Video as a third choice under Settings > Screensaver > Show, so Lumen's screensaver plays the video wallpaper's places with the clock on top.
**Context for next agent:** `app/src/components/Ambient.tsx` crossfades still pictures; the video player lives in `app/src/lib/wallpaperVideo.ts`. Decide whether it reuses the home screen's video places or gets its own, and check the TV's weak graphics chip can play video under the status overlay. `watch.sh` already ends LG's screensaver while the video wallpaper plays in front.

## Separate pictures for the screensaver
**Origin:** Screensaver without Immich (2026-10-27). Lumen's screensaver reuses the pictures picked for the home screen.
**Idea:** Let the screensaver have its own picks, for example landscapes on the home screen and art on the screensaver.
**Context for next agent:** Today `Ambient` gets `config.wallpaperPicks`. A second store needs its own picker entry point without duplicating the Choose wallpaper page; decide where it lives in Settings > Screensaver.

## Switch off only LG Home's preload, not the whole preload manager
**Origin:** 2026-11-02. The preload manager kept opening LG Home (the stand-in) over iPlayer, so `watch.sh` now stops the preload manager outright.
**Idea:** Find what keeps setting `com.webos.app.home` back to `true` in `/var/preferences/webos-preload-manager-conf.json` (it was off at 08:21 and back on before 11:03), and stop that, so YouTube and Live TV can preload again.
**Context for next agent:** `setPreloadPolicy` answers true but changes nothing live. `/etc/palm/webos-preload-manager-conf.json` has Home `true` built in, and the binary also reads `/var/preferences/webos-preload-manager-debug-conf.json`, which might win. Notes: `docs/keep-alive.md` (preload paragraph). Check with `grep preloadmanager /var/log/messages` on the TV.
