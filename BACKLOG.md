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

## Stadia drop: release held buttons and pause the game
**Logged:** 2026-11-13
**Origin:** Stadia drops on 2026-11-13. A drop while a direction was held left Moonlight moving the character, and the drop cost a fight.
**Idea:** When `stadia.sh` sees the controller drop, send Moonlight a "nothing pressed" report so held buttons let go, then a Start press so the game pauses until the controller works again.
**Context for next agent:** Moonlight reads the controller from hidraw inside its jail (`/var/palm/jail/com.limelight.webos/dev`), and the dead device belongs to LG's HID host, so the reports must come from a gamepad of our own made through `/dev/uhid`. `tools/tv/stadia-bridge.py` already makes one (same IDs and report map, added to the jail); report ID 3, 10 bytes, rest is `[8,0,0,128,128,128,128,0,0,0]` (docs/research/spikes.md, "Bridge test"). Decide whether Moonlight treats it as the same player or a second controller, and which bit is Start.

## Native screensaver (from the 2026-11-24 review)

- The dark or light look (darkMode auto) is decided when the screensaver starts, so one that runs across 19:00 or 06:00 keeps the old look. Decide it per draw in `status_draw`.
- `localtime`, `gmtime` and `strtok` are shared between the threads; move to the `_r` forms.
- The log in `/tmp/lumen-saver-<uid>/log` is never trimmed (cleared at reboot).
- Cloud wallpaper pictures kept in `/tmp` have no cap (about 300 KB each, until reboot).
- Lumen Native: the Live TV row on home shows channel names on coloured tiles, while the Guide shows the real logos. Use the logos in the row too.
- Lumen Native: in the Full guide, the subtitles mark is drawn over the first letters of the programme name ("This is BBC Three").
- Lumen Native: the first Down into the Live TV row drops frames (23 of 30, one of 159 ms) while the guide loads.
- Lumen Native: now the frame rate allows it, try tiles that squash together a little as they slide off screen on Up and Down.
