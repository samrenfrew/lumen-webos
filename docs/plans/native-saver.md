# Native screensaver pilot

Started 2026-11-24, branch `native-saver` (from `live-tv-preview`). The pilot for a native (GPU) Lumen, as `PLAN.md` (Parked) said:
build the screensaver natively first and measure it before deciding on the launcher.

## Why the screensaver

- Lumen Screensaver, the web side app, was turned off on 2026-11-20 after the TV restarted several times (`PLAN.md`, the restarts
  item). Its graphics memory was about 180 MB while up, on a TV with 1.25 GB and about 200 MB usable while Lumen and Live TV run.
- It can only come back if it uses much less memory. A native app has no web engine, so the photos and the clock are most of its cost.
- It is small: full-screen photos, a crossfade, a clock, any key ends it. It shows quickly whether native is smoother on this TV.

## What the TV gives a native app

- The TV's own `/usr/lib` has SDL2 2.0.14 (LG's webOS build), SDL2_image, SDL2_ttf, libcurl, wayland-client and
  libwayland-webos-client. The app links only those, so the package is one small binary.
- The window type (card, overlay, screensaver) is a webOS shell surface property, `_WEBOS_WINDOW_TYPE`, set after SDL makes the
  window (the way webosbrew/home-chooser's native fixture does it). LG's SDL 2.0.14 has no hint for it.
- Userland is 32-bit ARM, soft-float (`/lib/ld-linux.so.3`), glibc 2.35. Kernel is aarch64.

## Building

- Toolchain: openlgtv/buildroot-nc4 `arm-webos-linux-gnueabi` SDK, unpacked in `~/toolchains/` on the NUC. Its host tools need
  glibc 2.36, newer than Ubuntu 22.04's, so `native/saver/build.sh` runs inside a `debian:bookworm-slim` container:
  `docker run --rm -v $HOME/toolchains:$HOME/toolchains:ro -v $PWD:/src -w /src/native/saver -e HOME=$HOME -u $(id -u):$(id -g) debian:bookworm-slim sh build.sh`
- Package: `cd app && npx ares-package ../native/saver/build/pkg --no-minify -o ../native/saver/build`.
- The binary links against the SDK's SDL 2.32 headers but loads the TV's 2.0.14, so use only calls SDL 2.0.14 has.

## Steps

1. **Spike A, as an overlay.** App `com.samrenfrew.lumen.nsaver`, hidden. Crossfades the wallpaper pictures in Lumen's folder every
   8 s with a clock, fades in, any key fades out and quits. Logs `nsaver` lines to `lumen.log`: start to first frame, photo decode
   and upload times, slow frames, RSS. Measure graphics memory (`GPUTotal - GPUCached`) before and while it is up, over Live TV
   or a paused app. **(verify)** It draws as a transparent overlay, keys reach it, and the app under it is left as it was.
2. **Spike B, in LG's screensaver's place.** The same mount as `side-apps.md` section 4, with a native `appinfo.json` and window type
   `_WEBOS_WINDOW_TYPE_SCREENSAVER`. **(verify)** The TV launches it on its timer, and a key goes back to the app.
3. **The real one.** Lumen's settings from `/var/lib/homehook/lumen-config.json`, Immich photos (the selection in `lib/immich.ts`),
   the SF font, the date and weather, backlight, and the Lumen's-own-screensaver check. Only if steps 1 and 2 pay off.

## Results

Spike A, 2026-11-24, over Live TV, five of Lumen's wallpaper pictures (1920x1080 JPEGs):

| | Web side app (2026-11-19) | Native spike |
|---|---|---|
| Graphics memory while up (`GPUTotal - GPUCached`) | about 180 MB | about 30 MB (158 to 189) |
| The app's own memory (RSS) | web engine renderer | 6-8 MB, 22 MB peak |
| Start to first frame | 1.3-1.5 s (in LG's place), 4.5 s (overlay) | 1.3-1.6 s (overlay) |
| Crossfade | not measured | about 50 fps, a third of frames 21-28 ms |

- Seen by Sam: the overlay draws over Live TV, a key fades it out and closes it, Live TV is left as it was.
- Sam found the 1.6 s crossfade a little fast and liked 2.4 s.
- Bug fixed: halfway through a crossfade Live TV showed through. The compositor reads the window's alpha, and SDL's blend left
  it below 1 while two photos mixed. The incoming photo now uses a blend that keeps the window's alpha as it is.
- A JPEG takes 80-190 ms to decode and 35-125 ms to upload. It loads 2 s before the crossfade, while nothing moves.
- Native apps run in a jail as their own user. They can read Lumen's folder and `/usr/share/fonts`, and write `/tmp` (the real
  one), but not `/var/lib/homehook`. The log is `/tmp/nsaver.log`.
- Spike B, 2026-11-24, by hand over SSH: a copy with LG's id and `"type": "native"` in `/var/lib/homehook/saver-app` (the web copy
  moved to `saver-app.web`), bind-mounted over `com.webos.app.screensaver`, then `blockSystemApp` and `releaseBlockSystemApp`. The
  app manager reported it as native. Seen by Sam over paused Plex, twice: the TV launched it on its own timer, the photos and
  clock came up, a key went back to Plex where it was paused. watch.sh saw the mount and only logged it.
- Under LG's id the app runs in LG's screensaver's jail, which has no `/media/developer`, so it cannot read Lumen's folder. It
  looks for photos in its own folder (`photos/`) first. The real one downloads from Immich, so this only matters for the
  wallpaper pictures choice.
- With no photo, the clock drew over the last frame (two times on top of each other). The frame is now cleared every time.
- `fs.protected_regular` stops one jail user appending to a `/tmp` file another created, so the log only works for the first
  user that writes it. Give each its own file in the real build.
- **(verify)** Hyperion. After the trials, Sam found the ambient lights on a solid colour. `hyperiond` (started at the 05:41 wake)
  was alive but listened on no port and its loader did not answer; after a restart of both, Piccap was connected but its video
  capture had stopped (`videoRunning: false`). A standby restart of the TV fixed both. Not known whether the screensaver caused it:
  LG's own screensaver puts the TV in the same power state. Kept on as the TV's screensaver; after its next run, check
  `luna://org.webosbrew.piccap.service/status` (`connected` and `videoRunning` true) and that `hyperiond` listens on 19400. If it
  breaks again, unmount it.
- The mount does not survive a reboot (no `saver-mount` flag, no guard), and the web copy is back in `saver-app`; the native copy is `saver-app.native` (the mount follows it there).
- `/tmp/nsaver-tune` (`crossfade=<ms>`, `photo=<ms>`) is read before each photo, so timings can be tried live without a build.
