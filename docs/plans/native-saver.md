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
3. **The real one.** In the parts below, each checked by Sam on the TV before the next. Done so far: part A.

## The rest of the plan

The web screensaver (`app/src/saver/main.tsx`, `components/Ambient.tsx`) is the spec: the native one does what it does, ported, not
redesigned. Each part is checked on the TV over SSH (`luna-send ... launch {"id":"com.webos.app.screensaver"}` puts it on screen
without waiting for LG's timer), and timings are tried live through `/tmp/nsaver-tune` before they go into the source.

### A. Your photos (done 2026-11-24, seen by Sam)

`src/picker.c` ports `lib/immich.ts`: the sources and their mix (`immich:recent`, `recent-people`, `memories`, `favorites`,
`album:<id>`, several joined with `|`), the recent-photo weighting (`TIERS`), camera photos only on the broad sources, stacks left
out, portrait pairs from different days, the pool of unshown pictures, faces placed as `place()` does, and the wallpaper pictures
when Immich is off or unreachable. `src/http.c` uses the TV's libcurl, so HTTPS works with the TV's own certificates. A
background thread picks, downloads and decodes the next picture while the current one shows. JSON is cJSON 1.7.18
(`vendor/`, MIT). State (`pool.json`, `frames.json`, `stacks.json`) and the log live in `/tmp/lumen-saver-<uid>/`, so a reboot
starts a fresh pool. The picture downloaded but not yet shown is kept there (`ready.json`) and opens the next run at once.

`lumen-saver --pick <app folder> [n]` runs only the picker and prints what it picked, for checks over SSH with no window.

### B. Settings that follow Lumen

The screensaver's jail cannot see `/var/lib/homehook`. It reads `config/lumen-config.json` and `config/immich.json` from its own
folder (`saver-app.native`), which the jail mounts.

- `watch.sh` copies both files into the mounted folder when either changes (compare mtimes each loop), mode 644. The Immich key stays
  inside the screensaver's own folder, not in the shared `/tmp`.
- `home-setup.sh saver-mount on` copies them when it stages the folder.
- Today they are a copy made by hand on 2026-11-24, so a settings change in Lumen does not reach the screensaver yet.

### C. The look

Match Lumen's screensaver (`theme/ambient.css`, `components/Status.tsx`, `components/Forecast.tsx`):

- **Font.** SF is private (Apple's licence): the TV has `/var/lib/homehook/fonts/sf.woff2`, which FreeType cannot read. Convert it to
  TTF on the NUC with fonttools, push it beside the woff2, and have `saver-mount` copy it into the folder. Fall back to LG Smart UI.
- **Clock** bottom left at 120 px, with the 12 or 24 hour setting and AM/PM (`clockFormat`, `showAmPm`); **date** and **weather line**
  bottom right at 32 px, in the language Lumen uses (`language`).
- **Forecast widget**, small and large (`saverForecast`, `saverForecastSize`): weather from open-meteo for the city (`weatherCity`,
  from `config/cities.ts`), fetched with libcurl and refreshed every 30 minutes as `lib/weather.ts` does. Icons as PNGs rendered
  from the SVGs at build time.
- **Shade** behind the status (the eased dark band), **night dimming** (`wallpaperDim`), the **Print look** (`ambientPrint`: a cream
  veil, a black veil and a radial fall-off; three full-screen layers, so draw them into one texture once per photo, not per frame),
  and **reduce motion** (no fades).
- Text is drawn once a minute into a texture, so it costs nothing per frame.

### D. Backlight

As `lib/tvcontrols.ts`: read the backlight, dim to the Brightness setting (Dim 30, Low 10) while it shows, and put back the level for
now (the per-input schedule) on a key. Through `luna-send` from the app, or `system` settings calls with the luna library. **(verify)**
the screensaver's jail may call `com.webos.settingsservice` (LG's own screensaver has `requiredPermissions: all`).

### E. Quicker start and less memory

- First picture 3.9 s after launch on 2026-11-24 (2.8 s pick and download, 0.9 s upload of a pair). The kept `ready.json` picture
  should bring later runs near the 1.3 s of the wallpaper test.
- Immich previews come at up to 2560x1440. Scale each to the size it is drawn at (1920x1080, or 956 wide for a pair) on the
  background thread: smaller uploads and about half the graphics memory.
- Measure graphics memory over paused Plex with Lumen, the Guide and Quick Settings running, against the 180 MB of the web one.

### F. The wallpaper pictures choice

`saverShow: wallpaper` shows the pictures picked in Choose wallpaper (`wallpaperPicks`: `local:<n>` from Lumen's folder, `cloud:<id>`
downloaded). The jail cannot read Lumen's folder, so `saver-mount` copies the local ones in, and cloud ones are fetched as Lumen
does (`lib/wallpaper.ts`). Today it shows Lumen's five default pictures.

### G. Shipping it

- Replace the web side app: `com.samrenfrew.lumen.saver` becomes the native package (same id, so Settings > Side apps and the release
  listing stay). Overlay launches (Quick Settings' Screensaver button) work as today.
- `home-setup.sh saver-mount`: stage the native folder (with LG's id, `"type": "native"`, the config, font and pictures), check the app
  manager reports `native`, and mount it at boot from the guard as today.
- `npm run deploy:home` builds it with the Docker command above. The release workflow builds it too: download the SDK in the job
  (cache it) and run `build.sh` in `debian:bookworm-slim`.
- Then delete the web screensaver page (`app/saver.html`, `app/src/saver/`).

### Open checks

- **(verify)** Hyperion (below).
- **(verify)** It ends itself over Lumen's own screensaver, as the web one does (`/tmp/lumen-ambient-on`).
- **(verify)** After a reboot, with G done, it is still the TV's screensaver.
- **(verify)** Frame rate of a pair crossfade (two textures each side) against a single photo.

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
- Native apps run in a jail as their own user. They can read Lumen's folder (as a developer app) and `/usr/share/fonts`, and write
  the real `/tmp`, but not `/var/lib/homehook`. The log is now `/tmp/lumen-saver-<uid>/log`.
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
