# Frame rate on the TV

Measured on 2026-10-22 on the LG 43UT91006LA (webOS 25), Live blur on, a still Immich wallpaper. The scripts are in `tools/tv/perf/`. They need the DevTools tunnel (`.claude/skills/tv/SKILL.md`).

## The scripts

- `walk.mjs <out.json> [dock,rows,cc,guide,settings]` walks the app with remote keys and records a trace. It relaunches Lumen before each part. It checks focus before each OK, so it does not open an app by mistake. `LIVECSS='…'` or `LIVEJS='…'` applies a change before each part, for A/B tests. `NORAF=1` turns off the in-page frame counter.
- `walk-report.mjs a.json [b.json]` prints a table for each part, with b beside a.
- `swaps-per-key.mjs <file> <part>` counts the frames that reach the screen in the 320 ms after each key. A 60 fps move gives about 19.
- `updown.mjs <out.json> [pairs]` presses Down and Up between the dock and the grid, many times after one relaunch, and records a trace. `updown-report.mjs a.json [b.json …]` prints medians per key: the first Down, later Downs, and Ups. `LIVECSS` and `LIVEJS` work as for `walk.mjs`. A relaunch does not reload Lumen when it is already running, so a `LIVEJS` change stays until `location.reload()`.
- `raster-tiles.mjs` counts the raster tiles per layer per key, from an `updown.mjs` capture made with `LMAP=1 CATS=cc,disabled-by-default-cc.debug`. `layer-paints.mjs` lists the layers that repaint. `key-timeline.mjs` prints one key's frames and long tasks.
- `return.mjs <out.json> <dock|row2> [runs]` opens Moonlight (from rest, or from row 2 with remote keys), waits, presses Home over SSH, and traces the return. `return-report.mjs` prints the frames shown, raster, decode and GPU time per return. `SHOTS=1` saves the frames the page drew. `LIVEJS` as for `walk.mjs`. It opens an app on screen, so ask Sam first.
- `probe.mjs ['<css>']` moves a small square for 2 s and prints the frame gap and the GPU draw time. It is the quickest check of what the page costs per frame.

The in-page frame counter forces a main-thread frame on each vsync. The swap counts without it were the same, so it does not change the result.

## What we found

Every focus move, on the glass bar or in the lower rows, shows about 7 frames in 320 ms. That is one frame every 40-47 ms, about 22-25 fps. The panel is 60 Hz and Chromium ticks at 60 Hz.

The cause is the GPU draw, not the app's script or layout. Each frame the GPU thread draws the whole screen (about 11 ms on the home screen), then the buffer swap blocks until the next vsync. If the draw does not fit before the vsync, the frame takes two or three vsyncs.

`probe.mjs` (a 40px square moving by transform):

| Page under the square | Frame gap | GPU draw |
|---|---|---|
| Home screen as it is | 39 ms | 11.2 ms |
| No live blur anywhere | 20 ms | 3.2 ms |
| Lower rows hidden (the glass bar's blur gone) | 21 ms | 8.9 ms |
| Top buttons' blur off, glass bar's on | 34 ms | 6.4 ms |
| Page empty | 20 ms | 2.0 ms |

Each `backdrop-filter` costs about 2 ms of draw on every frame in which anything on screen changes, whatever its size. Chromium does not keep a blur result from one frame to the next. The home screen has four (the glass bar and three top buttons), so a small focus move pays for all four. Removing only some of them does not help much, because the draw must get under about 5 ms before the frame fits in one vsync.

Control Center is worse: 21 live blurs (10 grid controls, 7 list choices, the pill, 3 top buttons), 31 ms draw per frame, about 16 fps on the list open. Without blur, the draw is 5 ms.

Smaller things, measured but not the main cause:

- Each hidden open-app dot (`.tile__dot::before`) runs a `transform` transition on every focus move. Chromium does not put an animation with no visible change on the compositor, so it runs on the main thread.
- The focused tile gets `will-change: transform` only while it is focused, so each move removes one layer and makes another. A permanent layer on every row tile cut raster in the lower rows by about 40% and the key-to-screen time from about 120 ms to 65 ms, but not the frame rate.
- The focused tile's larger shadow and its shine (`::after`) cost some raster (800 ms to 590 ms over the rows walk), not frame rate.

## Left and Right on the glass bar, 2026-10-26

Each Left and Right repainted the bar's whole glass plate (`.row:first-child::before`, about 0.7 megapixels a move). The plate's sheen comes from a gradient in a CSS variable (`--glass-bar-sheen`). When the plate's style is computed again, Chromium makes a new gradient image and repaints the plate, even if the value is the same. Two writes caused this on every move: the row's own `--sel-x` (for the sliding highlight) and Home's `--lower-shift` on `.rows`, written again with the same value. The highlight is now a child element (`.row__shine`) that holds `--sel-x`, and `--lower-shift` is written only when it changes. Raster on the dock walk went from 457 ms to 196 ms, and Left and Right from about 5 to 7 frames per move to about 9. `LayerTree.layerPainted` shows which layers repaint; the scratch script used Left and Right in place of Down and Up in `layer-paints.mjs`.

Rule: do not write a style property on an element above the glass bar on each move. Put a value that changes per move on the element that uses it.

With frames on time, Down and Up felt too quick, so they now take 600 ms each, and the backdrop fades over 800 ms on Up (`tokens.css`).

## Down and Up, 2026-10-23

The full-screen blurred backdrop (`.lowerveil`) was rastered again 2 to 3 times on every Down and Up, because the TV's memory pressure makes Chromium drop its tiles. It is now a canvas, which has no tiles, and the bar and first grid row tiles keep a layer each. Down and Up went from about 10 fps to about 40 fps. The full table is in `docs/research/down-up-lag-review.md`. Note: `walk.mjs vert` starts with focus on the top buttons, so its Down and Up move between the top buttons and the dock, not into the grid.

## The overall fix

Taken instead (2026-10-22): only the focused top button or Control Center control blurs live, and the glass bar has no live blur (Settings > Appearance > Dock blur puts it back). The glass keeps its sheen and rims, so it still reads as glass.

## The Guide (2026-10-23)

`tools/tv/perf/guide.mjs` walks the Guide with a trace: Now and Next down and up, the switch, the Full guide down and up, and back. `guide-report.mjs` gives medians per step. Do not add the `disabled-by-default-cc.debug` category: on 2026-10-23 it made the TV restart.

In Now and Next, each move rastered the focused row's drop shadow again, twice (the row that loses focus and the row that gets it). That was about half the raster per move, and it gave one stall of 100 to 160 ms. The white capsule is now a `::before` plate on its own layer that only changes opacity. Now and Next went from 9 to 12 frames per move to 12 to 17, and the worst stall to 40 to 75 ms. The look is the same.

Full guide moves were already at about 14 frames, so they keep their shadow. The switch between the two views still stalls for 200 to 300 ms, because it builds the other view from nothing.

## Control Center (2026-10-23)

`tools/tv/perf/cc.mjs` reloads the page, then opens Control Center twice, opens two lists, moves round, and opens Inputs twice, with a trace. Read it with `WIN=1400 node guide-report.mjs`.

The open and close fades animate each control's opacity, so Chromium made a layer for each control when a fade started and dropped it when the fade ended. Each drop drew the control again, shadow included. Each control now keeps its own layer (`will-change`). On the first open, raster went from 300 to 460 ms to 120 to 140 ms, and from 3 to 5 frames to 6 to 8. Closing a list went from 150 to 370 ms of raster to none. The look is the same.

The panel opened on default values (backlight 100, Eye Comfort off) and redrew when the TV's values arrived 250 ms later. It now opens on the values last read, and the launcher reads them once it has settled.

Opening a list is still the heaviest step: about 65 ms of JS in the key handler and 200 ms of raster for the new options and the pill.

### Lists again (2026-10-26)

After the design cleanup, lists felt laggy again: list close drew 215 ms of raster in 4 frames, and list open 274 ms in 3 frames. The live blur was not the cause: only 2 or 3 controls blur during a list open. `LayerTree.layerPainted` showed that each list open and close repainted all ten controls and all seven choices twice. Tests in the live page found two triggers. A class that sets `opacity` on a control repaints it, even when the value does not change and the control has its own layer. A change of `pointer-events` on a parent repaints all its children too. An opacity animation on the same control repaints nothing. The grid and the list were hidden by such classes (`cc__grid--leaving`, `cc__list--shown`).

The classes are gone. The fades that hide the grid now keep running at their end value while the list is open, and the opened tile is hidden by an animation too. Mouse handlers check the phase instead of `pointer-events`. With the change live: list close 11 ms of raster in 8 frames (worst gap 180 ms to 57 ms), list open 218 ms in 5 frames (worst gap 195 ms to 105 ms).

## Settings rows (2026-10-26)

Up and Down in a Settings list ran at about 16 to 18 fps. The rows shared one layer with the whole list, so each focus move rastered the full list (about 1000 by 840 px) for one row's zoom and shadow. Each row now has its own layer (`will-change: transform` on `.slist__row`), as the Guide rows do. In `walk.mjs settings`, raster time fell from about 1.1 s to 0.27 s and dropped frames by more than half. Sam: much smoother on the TV.

## Opening Settings, and keeping it built while closed (2026-11-01)

From an idea in Glasshouse: build the other screens while Home is still, so they open faster. A dev-only switch (`lumen.warm` in `app.tsx`) kept Settings built while closed, hidden (`mounted`) or drawn at 1% opacity so its tiles stay rastered (`drawn`). Each open was timed from the nav line: the key press, then the end of the first long frame gap, which is when Settings shows.

- Settings shows about 420 to 470 ms after OK once it has been opened before in the session. About 100 to 170 ms is the page's own work before the switch, and about 250 to 350 ms is the wait for the first frame. Keeping it built did not change this: `mounted` gave about 440 ms, the same as off.
- The first open after a page reload: about 525 ms with it off, about 480 ms built (three tries each, spread about 60 ms). The difference is inside the spread.
- The first open after an install took 870 ms. That is the browser's caches being empty, which warm-up does not cover.
- `drawn` made closing Settings worse: the return to Home gained gaps of 117 to 167 ms, where off had none.

Not worth it: building Settings is not where the time goes. The Guide was not tried; it fetches its data when it opens, so it is a different case. The time left is the app's render when the screen changes and the first frame's raster.
