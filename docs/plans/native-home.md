# Native Lumen: the home screen test and the port

Started 2026-11-24, branch `native-saver`, after the native screensaver (`native-saver.md`) was seen working. This is the plan for
the test app that came before the rewrite, and what it found. The rewrite's spec is `native-rewrite.md`.

## Why

- **Blur and glass.** The web engine works out every blur again on each frame in which anything changes, about 2 ms each
  (`docs/research/frame-rate.md`), so the glass bar lost its live blur and Liquid glass is off. A native app blurs the wallpaper
  once into a small picture, and the bar reads from it: the blur costs nothing more per frame, and the bend at the bar's rim is one
  more lookup for the bar's own pixels.
- **Memory.** The web engine held 62 MB of graphics memory on 2026-11-24 with Lumen in front, and has peaked at 290 MB
  (`/sys/kernel/debug/pvr/proc_stats/live_pids_stats/process_stats`). Home measured about 86 MB, Settings 106 MB, and Lumen in the
  background about 63 MB (`docs/keep-alive.md`). The native screensaver rests at 20 MB of graphics memory and 8 MB of its own.
- **Start time.** The native screensaver has a window 0.5 s after launch. A cold web app takes 3 to 11 s.

What native does not give by itself:

- **Frame rate.** The screensaver's crossfade ran at 48 fps with a third of its frames late, and it only draws two full-screen
  pictures. The chip (PowerVR Rogue, `rgx_merlin7`) is slow at filling the screen. The test app measures this for the home screen.
- **The video wallpaper** is on a video layer the page cannot read. A native app probably cannot read it either, so glass over
  video likely stays a tint and rims.
- **TV calls.** A native app runs in a jail and cannot run `luna-send`. The screensaver has watch.sh do its calls. Lumen calls the
  TV all the time (apps, inputs, Live TV, settings, root commands), so this needs its own spike before the launcher is ported.
- **Text.** SDL_ttf 2.0.14 has no letter-spacing, kerning or shaping. Fine for a clock, more visible in Settings and the Guide.

The order of the port, its spikes and its budgets are in `native-rewrite.md`.

## The test app

`native/home/`, app `com.samrenfrew.lumen.ntest` ("Lumen Native Test"), a full-screen app of its own, so nothing in Lumen changes.
It draws with OpenGL ES 2 shaders (the TV's `libGLESv2`), not SDL's 2D renderer as the screensaver does, because blur and the bend
need shaders.

- **What it shows.** Lumen's first default wallpaper, the glass bar with six app tiles, and two grid rows under it, at the
  positions read from the live page on Sam's TV (tiles 240x144 with a 46px gap, the bar 1750x224 at 85,806, Down moves it up 770px).
- **Keys.** Left and Right move focus. Down and Up slide between the bar and the grid with today's timings and curves
  (`tokens.css`), and the backdrop fades between the sharp and the blurred picture. OK changes the glass: Liquid glass (blur, the
  bend at the rim and a colour fringe), Blur only, No blur (today's look). Back closes it.
- **How it draws.** At start it draws the wallpaper at screen size, then makes two blurred copies on the graphics chip: one for
  the glass (480x270) and one for the backdrop behind the grid (240x135, with its 26% dark tint in the pixels). A frame is the
  backdrop (one picture, or a mix of two during a slide), the tiles, and the bar. Each is one shader that works out its own rounded
  shape, shadow, sheen and rims, so no other pictures are held. It draws only while something moves.
- **The readout** on screen and the `ntest` lines in `/tmp/lumen-ntest-<uid>/log` give, for each move: frames, how long it took,
  frames later than 20 ms, the slowest, and the draw time per frame (the time the chip takes, read with `glFinish`).
- **Live tuning.** `/tmp/ntest-tune` is read at every key press, `name=value` per line: `down`, `up`, `veil_down`, `veil_up`,
  `focus` (ms); `glass_halvings`, `glass_passes`, `veil_halvings`, `veil_passes` (how strong each blur is); `bezel` (px), `bend`
  (px), `fringe` (0 to 1); `saturate`, `brighten`, `tint`; `mode` (0 to 2); `hud` and `finish` (0 or 1); `wallpaper=<path>`.

Build and install:

- `sh tools/build_native_test.sh` builds the package (in Docker on the NUC, as the screensaver's build does).
- `npm run deploy:ntest` builds and installs it. `LUMEN_LAUNCH=1` also opens it.
- `sh native/home/host-shot.sh <out.png> [row] [col] [mode] [slide]` draws one frame on the NUC with software graphics, in a
  container, to check the look with no TV. `LUMEN_SHOT_ZOOM=x,y,w,h` saves that part at three times the size.

## On the TV (2026-11-24)

- It starts: the shaders compile on the PowerVR BXE-4-32, and the first frame is ready 0.65 s after launch. Seen by Sam: "looking
  good". He found the first glass blur (480x270, two passes) too strong and the slide slow at today's timings, so `/tmp/ntest-tune`
  now has a lighter blur (`glass_halvings=0`, `glass_passes=1`) and `down=450`, `up=400`, `veil_down=400`, `veil_up=520` to try.
- **The screen refreshes every 20 ms (50 a second), not 60**, measured by timed buffer swaps. So 50 is the most any app can show
  on this TV as it is set now, and a frame is late only past about 30 ms.
- **The glass costs nothing extra.** Liquid glass, Blur only and No blur drew in the same time.
- **Working shapes out per pixel is what costs.** The first version worked out each rounded shape, shadow, sheen and rim in the
  shader on every frame: 21 ms a frame at rest and 29 ms mid slide, so Left and Right just missed 50 fps and Down and Up ran at
  about 37. A plain full-screen picture is only about 2.3 ms, and an empty frame 3 to 4 ms.
- **Fix: draw the shapes once.** The bar's bend, outline and shadow go into one picture and its sheen and rims into another, made
  at start (`bake_bar`); each resting tile is a picture too (`bake_tiles`). A frame then only looks pictures up. Only a tile that
  is changing focus is still worked out live. Draw time: 10 ms at rest, 12 ms mid slide, 9.5 ms down in the grid, and every frame
  lands on the 20 ms beat. That would also fit a 60 a second screen (16.7 ms).
- `touch /tmp/ntest-bench` before launch runs a timing test of each layer for about ten seconds and logs `bench` lines.
- Rule for the port: anything with a rounded shape, shadow or gradient is drawn once into a picture and reused; per-frame shaders
  only look pictures up and mix them.

## Open checks

- Back reaches the app as scancode 482 with key code 0. Both test apps close on it; seen on the Rust one (`docs/research/spikes.md`, Rust).
- **(verify)** Back closes the C test app, once it is reinstalled.
- **(verify)** With the shapes drawn once, real Down, Up, Left and Right moves show every frame (the readout's "frames of").
- **(verify)** Why the screen is at 50 a second, and whether Lumen's web page is too (`frame-rate.md` assumed 60).
- **(verify)** Graphics memory while it is up (`/proc/gpu/<pid>` or the driver's `process_stats`), against the web home's 86 MB.
- **(verify)** The look of the Liquid glass bar, to Sam's eye, and the strength of the two blurs.
