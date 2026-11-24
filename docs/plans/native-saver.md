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
3. **The real one.** In the parts below. Part A was seen by Sam. Parts B to G were built on 2026-11-24 in one run and checked
   headless on the TV (see "Built, not yet seen"); none of them has been on the screen yet.

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

### Built, not yet seen (2026-11-24)

How it is put together now:

- **One finished frame per picture.** `src/compose.c` builds each picture at screen size on the background thread: placed as the
  CSS places it (cover, or a pair on black), scaled with area averaging, the bottom blur, the Print look and night dimming all in
  the pixels. The main loop only crossfades two opaque 1920x1080 textures, so a pair costs the same as a single photo. 200-290 ms
  a picture on the TV; white with the Print look matches the CSS to one level.
- **One status band.** `src/status.c` draws the shade, clock, date, weather line and forecast widget into one 1920x520 texture,
  once a minute (40-120 ms). Positions are worked out from the CSS; SF comes from `fonts/sf-<weight>.ttf` (made by
  `tools/make_sf_ttf.py`, private), else LG Smart UI. Icons and the strings for each language are made by
  `tools/gen_saver_look.mjs` and committed in `native/saver/assets/`.
- **Weather.** `src/weather.c` ports `lib/weather.ts`; cities from `assets/cities.tsv` (`tools/gen_saver_cities.mjs`).
- **Wallpaper pictures.** `local:<n>` is `photos/default-<n>.jpg` in the app's folder; `cloud:<id>` is downloaded once a boot from
  the URL in `assets/wallpapers.tsv` (`tools/gen_saver_wallpapers.mjs`).
- **No luna calls from the app.** `/usr/bin/luna-send` is root only, so the jail cannot run it (the `turnOnScreen` call in the
  spikes failed with 126 and a key still went back to the app). `watch.sh` does all of it. The app keeps
  `/tmp/lumen-saver-<uid>/showing` (its pid) from its first frame until a key; watch.sh dims the backlight when a live one
  appears and puts it back when it goes, so the backlight comes back up to a second after the key, not at once as on the web one.
- **Lumen's own screensaver.** In LG's place, when `/tmp/lumen-ambient-on` exists the app waits up to 3 s before it makes a
  window, and watch.sh ends it (`turnOnScreen`, then `closeByAppId`) when Lumen is in front. After 3 s it shows anyway.
- **Settings.** watch.sh copies `lumen-config.json` and `immich.json` into `config/` of the staged copy and of the installed side
  app when they change, and writes `config/tv.json` (the TV's language and clock format, for the `auto` settings).
- **Shipping.** `native/saver/appinfo.json` is `com.samrenfrew.lumen.saver` 1.1.0. `tools/build_native_saver.sh` builds (in Docker
  when the host's glibc is under 2.36) and packages; `deploy_saver.sh`, `build_release.py` and the release workflow use it. The
  workflow downloads the SDK (`openlgtv/buildroot-nc4` release `2026.08-webos`, checked by SHA-256) and caches it. The web page is
  removed in a commit of its own, so it can be reverted alone.

Not ported: letter-spacing and kerning (SDL_ttf 2.0.14 has neither for SF), and Hindi is unshaped. The fallback font is LG Smart
UI, where the web one fell back to Geist.

Before the new TV scripts run, by hand on the TV: `umount /usr/palm/applications/com.webos.app.screensaver` (the trial's mount of
`saver-app.native`, which the scripts do not know) and `rm -rf /var/lib/homehook/saver-app.native`.

### Seen on the TV (2026-11-24, installed with `deploy:home`, `deploy:saver` and `saver-mount on`)

- Seen by Sam over YouTube: "it looks great". A key ended it and went back to YouTube.
- Crossfade of a pair: 116 frames in 2.4 s (48 fps), 12 over 20 ms, slowest 27 ms. One earlier run had a single 131 ms frame.
- Backlight: 52 to 30 one second after it came up, back to 53 (the schedule's level) one second after the key.
- Over Lumen's own screensaver, watch.sh ended it and the TV closed it without `closeByAppId`.
- Graphics memory: 28 MB of its own, the same as the spike. `GPUTotal - GPUCached` rose by about 59 MB (126 to 185), but half of
  that is the measure, not the app: the driver keeps a pool of freed pages (`GPUCached`, 35 MB before the run) and hands it back
  to the system when the app starts, so the subtraction counts the pool's loss as use. The driver's own figures are the ones to
  read: `/sys/kernel/debug/pvr/driver_stats` (`MemoryUsageAllocGPUMemUMA` went 105.8 to 134.1 MB and back) and, per process,
  `/sys/kernel/debug/pvr/proc_stats/live_pids_stats/process_stats` (lumen-saver: 27.5 MB, 31.4 MB peak) or `/proc/gpu/<pid>`
  in 4 kB pages (6935 at rest, 7087 in a crossfade). The 180 MB of the web one was measured the old way, so it may be up to a
  pool's worth (about 35 MB) too high.
- The Print look is gone from the screensaver, in Lumen's own too (Sam, 2026-11-24: it is made for art, not photos). Art
  wallpapers on the home screen keep theirs. Night dimming is now a plain dim of each picture.
- The next photo is now fetched 10 s before it is due (`FETCH_LEAD_MS`), not straight after the last one. Graphics memory is
  ordinary memory on this TV, so holding it anywhere cost the same 8 MB. At rest: 19.6 MB of graphics memory and 8 MB of its own
  (RSS); 27.7 MB for the 12 s around a change. The cost: a run that ends mid-wait leaves no kept picture (`ready.json`) for the
  next start, which then waits on the download (first picture 3.3 s, of which 2 s is the pick and download).
- The last photo downloaded stays on disk (`ready.json` and its JPEGs are no longer removed once shown), and only the first pick
  of a run uses it. A start over Live TV with it: first picture 0.9 s after launch (renderer 0.5 s, building the pair 0.4 s).
  The 3 s seen over Lumen was the wait for Lumen's own screensaver, which was up at the time.
- A restage must keep the staged folder itself: the screensaver's jail holds it bound, so `rm -rf` of it fails with "busy".
  `saver_stage` now replaces what is in it, and the jail sees the new files.
- First picture 3.8 s after launch; building a pair with the Print look takes 300-550 ms of it.
- Hyperion after the run: Piccap `connected` and `videoRunning` true, `hyperiond` on 19400.

### Open checks

- **(verify)** Built 2026-11-24, not installed: with the side app installed, Lumen's idle timer and Control Center's Art mode
  open it (`showScreensaver` in `app.tsx`), and Lumen's own built-in screensaver is only the fallback. When the TV's timer then
  starts the copy in LG's place over it, that copy waits (it sees the other one's `showing` file) and watch.sh ends it (`nsover`).
  To see: Art mode opens the native one, the idle timer does, a key goes back to the home screen, and the TV's timer three
  minutes later does not put a second one on top.
- **(verify)** Graphics memory while up, against the 30 MB of spike A and the 180 MB of the web one (plan part E).
- **(verify)** The look against Lumen's own screensaver side by side: clock, date, weather line, both forecast sizes, Print look.
- **(verify)** Backlight dims and comes back, over Live TV (input `dtv`) and over an app (input `default`).
- **(verify)** `running` lists `com.webos.app.screensaver` while it is up in LG's place and `closeByAppId` closes it; the
  `turnOnScreen`, 1 s, close sequence lands inside the app's 3 s wait over Lumen's own screensaver.
- **(verify)** The TV leaves "Screen Saver" by itself within 2 s of a key (else watch.sh calls `turnOnScreen`).
- **(verify)** After a restage the jail sees the new folder, and the overlay's jail can read the root-owned files in `config/`.
- **(verify)** The overlay (Quick Settings' button): it has no `photos/`, so wallpaper picks fall back to Lumen's folder.
- **(verify)** The release workflow: the SDK download and cache, `relocate-sdk.sh` on the runner, and that the Homebrew Channel
  takes a `native` manifest. It runs only on a push to `main`.
- **(verify)** `noSplashOnLaunch` and `spinnerOnLaunch` are honoured for a native app.
- **(verify)** Hyperion (below).
- **(verify)** After a reboot it is still the TV's screensaver.

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
