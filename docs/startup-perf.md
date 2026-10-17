# Cold start: findings and plan

Written 2026-10-17, handing over from a cloud session to a local one. Sam reported that the first load got slower after the Live TV
work. This file is what we know so far, what is in the build to find out more, and what to do next.

## How to read a launch

Dev builds (the branch builds on the Homebrew Channel's dev feed, version 1.0.9xxx) write one "boot" line to the TV's log 20 s after
each launch (`lib/log.ts`, `bootMark` and `setRootTimer`). Over SSH:

```
grep ' boot ' /var/lib/homehook/lumen.log | tail -3
```

Every number is ms since the page started.
- **Stages:** `script`, `wp-start`, `frame`, `wallpaper`, `wp-blur-set` / `wp-bg-set` (each blurred copy applied), `apps-parsed <n>`,
  `apps-colours`, `apps`, `grid-start` / `grid` (the app grid drawn, 3.5 s after launch), `live-cache <channels> <logos> <ms>`,
  `live-drawn`, `live-read`.
- **Details:** `iconcache <KB> <ms>` (the saved app icons parsed), `sample <app id>` (an icon colour worked out again), `sampled` and
  `wp-loaded` / `wp-decoded` (see below).
- **Root calls:** `<method> start-end`, for example `listLaunchPoints 1459-1658` or `getChannelList 7441-7911`. `base64` is the
  logos or the icons, `curl` is Immich.
- **Frozen stretches:** `long start+duration`, every main-thread task over 50 ms. Timers and the screen wait behind these.

The first launch after an update is slow for reasons of its own (nothing cached: `script` was 1786 against 254 the next time), so open
Lumen once after updating before timing anything.

## What the logs showed so far

**Run on 1.0.9084, Immich wallpaper, first launch after the update** (not comparable, see above):
`script 1786, frame 2040, listLaunchPoints 2675-3061, wallpaper 6417, live-drawn 7261, apps 7420, getChannelList 9263-9701,
find 9716-9853, live-read 9926, curl 14423-14692`

**Run on 1.0.9085, bundled wallpaper, warm:**
`script 254, iconcache 270 153KB 2ms, long 189+201, long 393+64, wp-start 490, frame 1049, long 1042+255, listLaunchPoints 1459-1658,
iconcache 1666 153KB 2ms, wallpaper 1880, long 1849+85, long 1934+1678, long 3617+80, long 3697+65, long 3763+139, long 3906+99,
long 4005+211, long 4217+76, long 4339+680, live-drawn 5437, long 5124+679, long 5805+393, long 6205+227, long 6432+130, grid 6573,
apps 6641, getChannelList 7441-7911, find 7933-8111, live-read 8199, curl 9894-10181, long 10220+1106`

What that says:
1. **The page is frozen most of the time from 1.9 s to 6.6 s.** The worst stretch is 1.7 s, starting the moment the wallpaper is shown.
   Everything else waits behind it: the app list came back from the TV at 1.7 s but was only handled at 6.6 s, the app grid (due at
   3.5 s) was drawn at 6.6 s, Live TV's 600 ms timer after the wallpaper ran at 5.4 s instead of 2.5 s.
2. **Live TV's TV calls are not the cause.** They all come after the freeze (7.4 to 8.2 s), and each is quick: channel list about
   0.3-0.5 s, now/next 0.2 s, logos 0.3 s. Drawing the row from the cache falls inside a 0.7 s stretch at 5.1 s, but that stretch can't
   be split from other work yet.
3. **The saved app icons are small** (153 KB, parsed in 2 ms), so they are not it.
4. Not measured yet with marks: the Immich wallpaper on a warm start.

## Suspects (guesses until the next logs)

- **Live blur (backdrop-filter).** The 1.7 s stretch starts as the wallpaper is set and the blurred copies are applied as CSS variables
  on `<html>`, which restyles the whole page and repaints everything behind the live-blurred glass.
- **Icon colours re-sampled every launch.** `withIconColor` in `lib/apps.ts` reads a pixel off a canvas for each "plain" icon. A colour
  is only cached when sampling works; an icon that is a file the app ships (not a data: URI) may taint the canvas, so it is never cached
  and is sampled again on every launch. The browser profile did show `drawImage` and `getImageData` on a warm start. `sample <id>` marks
  will show it.
- **The app grid drawn all at once** at 3.5 s, decoding every icon.
- **Live TV's cache read** (`cachedLogos` parses about 700 KB of logos as one JSON value). `live-cache` will show its time.

## Runs on 1.0.9086 (commit bbf1335)

Live TV and live blur are both cleared. The big freeze stays when either is off:
- Immich, Live TV on, live blur on (20:19): `listLaunchPoints 3072-3508`, six `sample` marks 3516-3562, **long 3643+3871**, `wallpaper 8140`,
  `grid-start 8527`, `live-cache 164ch 152logos 85ms`.
- Bundled, Live TV off, live blur on (20:21): six `sample` marks 2233-2243, **long 2269+2079**, `wallpaper 4960`, `grid 5263`.
- Bundled, live blur off (20:22, Live TV row still off): six `sample` marks 2693-2709, **long 2816+2921**, `wallpaper 6246`, `grid 6440`.

What that says:
1. In every run the freeze starts 30-110 ms after the app list comes back and icon sampling starts. It ends before the wallpaper is shown,
   before any blurred copy is applied, and before the grid is drawn. So the blurred copies, the grid and the logo cache are not it.
2. The same six icons are sampled on every launch (`com.limelight.webos` and five `com.webos.app.*`): they never give a colour, so are never
   cached.
3. Left: the icon sampling, or the wallpaper decoded on the main thread (`img.decode()` may not run off-thread on the TV's Chromium).

## Next step: one run on the next dev build

That build caches "no colour" (`none`) for an icon that can't be sampled, and adds marks:
- `sampled <id> <ok|clear|taint|error|nocanvas> <width>px <ms>`: one icon done, its size, and the time spent drawing and reading it.
- `wp-loaded <width>px` and `wp-decoded`: the wallpaper image loaded, then its decode finished.

Open Lumen twice after updating. The first launch samples the six icons one last time (with `sampled` marks); the second should have no
`sample` marks. If the freeze is gone on the second launch, sampling was it. If it stays, look at where `wp-loaded` and `wp-decoded` fall
against it.

## Fixes ready to make once the logs point at them

- **Logos:** store each logo under its own localStorage key (or keep the row's logos with the Live TV cache), so drawing the row never
  parses the whole logo cache. Also remember logo paths that failed to read, so they are not asked for on every launch.
- **Wallpaper:** apply the two blurred copies in one style change, after the first frame settles; or skip the fade/blur work when the
  picture is the one index.html already painted.
- **Grid:** draw the grid rows one at a time over a few frames instead of all at 3.5 s.
- If Live blur is the cost: make the cheaper blurred copy the default on the TV and keep live blur as an option.

## Where the Live TV work stands

See `docs/live-tv-build.md` (Status). In short: the row (under the app grid, channels picked in Settings > Channels, scrolls so the focused
channel's now/next fits), the Guide (Now and Next, typed channel numbers), Settings and the Guide on the dock's backdrop. Not yet confirmed
on the TV: the number keys arriving as digit key codes in the Guide, and OK opening Live TV from the row. Phase 2 (full guide grid) not
started.
