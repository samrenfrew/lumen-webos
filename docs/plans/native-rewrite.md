# Native Lumen: the rewrite

Written 2026-11-24, after the native screensaver (`native-saver.md`) and the home screen test app (`native-home.md`) were seen
working on the TV. This is the spec for rewriting Lumen and its side apps as native apps. Nothing here is built yet except those two.

## Decided (Sam, 2026-11-24)

- **A full rewrite, in this repo.** The web app in `app/` is the spec: the native one does what it does, ported, not redesigned.
  New work lives in `native/`. The TV scripts, tools, docs and release workflow carry over.
- **A new app id: Lumen Native, `com.samrenfrew.lumen.native`,** with `.guide`, `.quick` and `.saver` under it. The native apps
  never ship under the web apps' ids, so a Channel update cannot replace someone's working web launcher with one that will not
  start on their TV.
- **The web screensaver side app is gone for good**, from the frozen web version too: it caused the memory trouble
  (`PLAN.md`, the restarts item). Its id, `com.samrenfrew.lumen.saver`, is retired. Before `native-saver` reaches `main`: the
  native screensaver moves to `com.samrenfrew.lumen.native.saver` (it now sits in `native/saver/appinfo.json` under the old id,
  as 1.1.0), and the old id leaves the release workflow, the Channel listing, Settings > Side apps and the TV scripts. The
  screensaver inside the web Lumen stays, as the one the frozen version has.
- **Rust.** The launcher stays open for weeks, and a crash leaves the TV with no home screen. Spike 1 proves it on the TV first:
  the test app's scene in Rust with the same frame times, or we stay on C. The screensaver's C parts (`picker.c`, `weather.c`,
  `compose.c`) are linked in as they are and ported later.
- **The web version is frozen when native replaces it.** It stays downloadable (164 stars, 15 forks, 87 downloads of 2.2.1 when
  this was written) and gets no fixes. The README says which is which.
- **One TV.** The native build targets the LG 43UT91006LA on webOS 25 only: its PowerVR chip, its SDL 2.0.14 and its glibc 2.35.

- **Its title is "Home" (Sam, 2026-11-24)**, as the web Lumen's is, because Home Assistant automations go by the app's name.
  The id stays `com.samrenfrew.lumen.native`.
- **The web Lumen is off the TV (Sam, 2026-11-24).** Home, Back and power-on point at Lumen Native (`home-setup.sh`), and
  `com.samrenfrew.lumen` was uninstalled so it cannot come back to the front. It stays in the repo and the Channel.

## What does not come over (Sam, 2026-11-24)

The port starts as today's Lumen, one to one, less what Sam does not use. Anything left out can be added later if missed.

Left out:
- **The video wallpaper.** So glass over video needs no answer.
- **The light look.** Dark only.
- **Liquid glass, and glass as a setting.** The native Lumen has the one glass the web Lumen shows today. Liquid glass and Dock
  blur are not choices; Liquid glass may come back later.
  Back on 2026-11-24 as Settings > Appearance > Liquid glass, off by default: the app bar and every glass button bend the
  wallpaper at their rim, over a lighter blur (`bend`, `fringe`, `bezel`, `liquid_blur` in `look.rs`, tunable live with `nctl.mjs set`). Seen on the TV by Sam (2026-11-24); the values may be tuned later.
  **Glass lab (2026-11-24, dev builds):** Settings > Appearance > Glass lab, or launch with `{"view":"glass-lab"}`, opens the
  home screen with only the app bar and the three top buttons, neither usable, and a panel of sliders for every glass value in
  `look.rs`. New values added for it, each off or as before by default: `edge_shape` (2 is the round edge as before, higher is
  flatter in the middle and steeper at the rim), `magnify`, `specular` and `light_angle` (a glint along the rim facing the light),
  `rim_light`, `colour_rims` (the blue and pink rims on the bar), `button_colour_rims`, `sheen`, `frost` (white haze) and `glow`.
  The panel shows how long the last frame took to draw, and frames a second while "Move focus by itself" is on. What is set
  there is kept in the look file, as with `nctl.mjs set`; Reset puts back only the glass values. Sam tried it and set new defaults (2026-11-24): `liquid_blur` 0.1,
  `bezel` 27, `bend` 36, `rim_light` 0.5, `colour_rims` 0.35, `button_colour_rims` 0.1, `glow` 0.5. Added after:
  `side_width` and `side_bend`, which make the band of bend on the left and right edges narrower or weaker than on the top and
  bottom, blending round the corners, so a pill reads more like a drop of water than an even ring. **(verify)** their look.
  Liquid glass is on by default from 2026-11-24 (Sam).
- **Down and Up peel the rows (2026-11-24, (verify)).** The gap under the app bar now grows and shrinks over the same time and curve
  as the slide, not in 200 ms ahead of it. Each row below the bar also slides on its own, starting `peel_down_ms` (60) later than
  the row above on Down and `peel_up_ms` (35) on Up, and catches up by `peel_catch` (0.7 of its delay), so later rows move faster.
  On Down the rows use `peel_down_*`, a curve that overshoots a little (y2 1.12) so each row snaps up; on Up they use `peel_up_*`,
  with no overshoot, so each row closes onto the one below. All tunable live with `nctl.mjs set`.
- **The Print look** on the home screen.
- **Focus strength** as a setting. One strength.
- **Hide the dock** and its dim.
- **The cloud wallpaper library.** Sam uses the built-in Landscape and Art wallpapers; those stay.
- **Languages other than English.** Text shaping for all nine is proven (spike 6), so they can return as strings only.

Kept, asked and not dropped: the live TV preview (so the open half of spike 5 still matters), folders, the pointer, Immich
photos, the brightness schedule. Not answered, so kept: the Full guide, the Guide over Live TV, Quick Settings over any app,
typed channel numbers.

## What the measurements ask for

From `native-home.md` (On the TV) and `native-saver.md` (Results):

- A frame has 20 ms: the screen showed 50 frames a second when measured. An empty frame costs 3 to 4 ms and a full-screen picture
  about 2.3 ms more. **Budget: 12 ms of drawing per frame**, so it also fits a 60 a second screen.
- Working a shape out per pixel is what costs. **Rule: every rounded shape, shadow, gradient and line of text is drawn once into
  a picture and reused. A per-frame shader only looks pictures up and mixes them.**
- Blur is free once made: the wallpaper is blurred once at a small size, and any glass reads from that.
- Draw only while something moves. At rest the app draws nothing.
- **Memory budget:** 40 MB of graphics memory in front, 10 MB hidden (pictures let go, kept on disk), 30 MB of its own. The web
  home is about 86 MB in front and 63 MB hidden. The test app is at 62 MB and that is not yet explained (spike 7).
- **Start budget:** first frame within 1 s of a cold launch, and within 0.3 s of coming back from another app.

## Shape

**One program, several apps.** A single binary, `lumen`, packaged four times with a different `appinfo.json` and start mode:
home (a normal full-screen app), guide, quick and saver (overlays). Each loads only what its mode needs. One renderer, one text
engine and one set of TV calls serve all four, where today there are four web pages and a C program.

Layers, lowest first:

1. **Platform.** The SDL window and its webOS type, keys and the pointer, the app's life (launched, sent to the back, brought
   forward with launch parameters such as `{"view":"ambient"}`), and the clock.
2. **Drawing.** OpenGL ES 2, as `native/home/src/gfx.c` does: programs, pictures, off-screen targets, and a **picture cache** that
   knows what each picture costs, keeps to the budget, drops the least used first, and drops nearly all of it when the app is hidden.
3. **Text.** FreeType and HarfBuzz, both on the TV, into a cached picture per line. This gives letter-spacing, kerning and shaping
   (Hindi), which SDL_ttf 2.0.14 cannot. SF when the private font is present, LG Smart UI when not. Digits stay proportional,
   except the clock time.
4. **Screen parts.** A small tree of parts, each with a place, a picture and animations of position, scale and opacity only, with
   the web app's timings and curves (`theme/tokens.css`) as named values in one file. Focus moves through the tree. Each focused
   part reports itself, so the native `where` can tell an agent where Sam is, as `lumen.where()` does now.
5. **Services.** TV calls, root commands, HTTP, JSON, settings and the log. All off the drawing thread: a slow TV call must
   never hold a frame.
6. **Features.** Home, Live TV, the Guide, Control Center, Settings, wallpapers, Immich, weather, the screensaver.

**Root commands** keep the rule from `CLAUDE.md`: one module, one fixed command per function, each value checked against a list or
pattern, as `app/src/lib/root.ts` does.

**Settings** stay one JSON file with the web app's keys (`lib/storage.ts`), so a setting changed in either version reaches the
other while both are installed, and the native one starts with Sam's settings as they are.

## Working on it

The web app's rule is "DevTools first, build last". The native app needs its own way to do the same, built in phase 1:

- **A control channel** on the TV (a socket the dev build listens on, reached over SSH): read `where`, set any named timing, size or
  glass value live, press keys, and save a picture of the screen. It replaces `cdp.mjs eval`, `css` and `shot`.
- **A build for the NUC** with software graphics (`native/home/host-shot.sh` is the start): the same code draws any screen to a PNG
  in a container. It is used to check a change before the TV sees it, and to compare a native screen with the web one.
- **Tests.** The web app has none. Parts with no TV in them (layout, the photo picker, guide times, languages, settings) get
  tests that run on the NUC, to the `testing-standards` skill. Screens get a kept picture each, compared on every build.
- `CLAUDE.md` is rewritten for the native app when it becomes the daily one (phase 7), not before.

What exists (phase 1), all in `native/lumen/` unless said:

- `test.sh` runs the tests in the `lumen-native` container. `tools/build_native.sh` makes the `.ipk`; `npm run deploy:native`
  installs it (`LUMEN_LAUNCH=1` also opens it).
- `host-shot.sh <out.png> [--at <ms>] [--against <kept.png>] [keys...]` draws a screen with software graphics on the NUC.
  `shots.sh` compares every screen in `shots/list` with its kept picture in `shots/`; `shots.sh --keep` takes them again.
  `LUMEN_PRIVATE=1` draws with the SF font from `app/private/fonts`, as the TV build does; the kept pictures are without it.
  `--as lumen-guide` or `--as lumen-quick` draws a side app.
- `tools/tv/nctl.mjs` talks to the control channel on `127.0.0.1:9997` over an SSH tunnel to port 19997: `where`, `stats`, `get`,
  `set <name> <value>`, `reset`, `key <left|right|up|down|ok|back>...`, `shot <out.png>`, `call <uri> <json>`, `quit`.
- The app's log is `/tmp/lumen-native-<uid>/log` on the TV; the boot line also goes to `lumen.log`.

## Spikes first (phase 0)

Each is small, answers one question on the TV, and is written up in `docs/research/spikes.md`. The rewrite does not start until
1 to 4 have answers.

1. **Rust.** The test app's scene in Rust: same frame times, start time and memory, built in the container and in CI.
   **Answered 2026-11-24: yes** (`docs/research/spikes.md`, Rust). Same frame times and memory as the C app on the TV, every
   key works, built in a container. Still open: a build in CI, and the float flag Rust is retiring (the Rust version is pinned).
2. **TV calls from the jail.** A native app has an entry in `/var/luna-service2-dev/client-permissions.d` but none in `roles.d`,
   and cannot run `luna-send`. Try, in order: the TV's `libluna-service2` or `libhelpers` from the app; a role file written by the
   TV scripts; a small root helper started by the TV scripts that takes fixed commands on a socket. The last always works, and
   may be the quickest of the three (today each root command starts a shell). Prove: list apps, launch one, read and set a
   picture setting, run one root command.
   **Answered 2026-11-24: the first route works.** The app calls the TV itself through `libluna-service2` on a nameless handle
   (LG's SDL holds the app's own name). No role file or helper is needed. A root command costs about 45 ms.
3. **A place to keep things.** The jail cannot see `/var/lib/homehook`. Find where the app may write files that last (settings,
   the icon and logo caches, the next wallpaper), and how it reads other apps' icons, which the web app gets as text through root.
   **Answered 2026-11-24:** a folder of its own under `/media/developer`. `/tmp` is shared with root. `/var/lib/homehook` and
   LG's app folders cannot be seen, so their icons and the TV scripts' files come through a root command.
4. **Staying alive.** The app is sent to the back and brought forward without restarting, gets its launch parameters each time,
   the Home button (LG Input Hook) and power-on open it, and the TV's memory manager treats it no worse than the web one
   (`docs/keep-alive.md`).
   **Answered 2026-11-24: yes, on the TV's keep-alive lists**, as the web Lumen is. The app takes its bus name before SDL
   starts, hears each relaunch with its parameters, and asks the window manager for full screen to come forward: 0.2 to 0.4 s.
   Not measured: graphics memory in the back once pictures are let go.
5. **Video.** The live preview and the video wallpaper are `<video>` elements today. Natively they need the TV's media service
   with a hole cut in the window. Find out whether both work, and whether the picture under a native window can be read (it
   cannot from a web page). If not, both stay out and glass over video stays a tint.
   **Mostly answered 2026-11-24:** a video plays under the native window (seen by Sam), and the video wallpaper is not ported
   anyway. The live TV preview's calls work from the native app and the TV reports the picture on screen; Seen by Sam (2026-11-24).
6. **Text.** A paragraph in each of the nine languages with FreeType and HarfBuzz, beside the web page's.
   **Answered 2026-11-24: yes**, within 2% of the web page's line widths. The SDK has no HarfBuzz; the build links a stand-in.
7. **The test app's memory.** Why 62 MB, and what one screen-size target costs on this driver.
   **Answered 2026-11-24:** 21 MB is pictures. A depth buffer SDL asks for by default costs 16 MB, and a drawing target per
   tile 12 MB. Without both: 33 MB. A screen-size target is 8.3 MB of pixels.
8. **50 or 60.** Why the screen is at 50 a second, whether it changes with the input or the content, and what the web page gets.
   **Answered 2026-11-24:** the panel keeps the rate of the last video source (50 after Live TV, 60 after a 60 a second HDMI
   input). The web page always draws 60, so it drops one frame in six at 50. A native app is given the panel's beat.
9. **Shipping.** The release workflow builds a native package and the Homebrew Channel lists it under the new id beside the old.

## Phases

Each phase ends with Sam using it on the TV. The web Lumen stays installed throughout as the way back, and settings made in it
reach the native one, so Settings can be ported late.

1. **Foundation.** The layers above up to Services, the control channel, the NUC build, the first tests. Shows the test app's
   scene with Sam's real apps, wallpaper and clock, and OK opens an app. Done when the budgets hold.
   **Built and on the TV 2026-11-24.** Registers, takes its settings, app list, icons and a wallpaper, keys work, OK opens an
   app, and it stays alive in the back once on the keep-alive lists (`docs/keep-alive.md`). Measured, not yet tuned (Sam: build
   first, tune once the whole app is on the TV):
   - Drawing 10.4 to 11.5 ms a frame on moves with the GPU finishing each frame, worst 15.5, no frame late.
   - First frame 0.7 to 1.0 s on a quiet TV, 2.0 and 3.7 s while the TV's own services were busy.
   - Graphics memory in front 30 to 33 MB, but 41 MB on a first launch (peak 52 while the wallpaper swaps). Over budget.
   - Hidden: 22.8 MB against 10 MB. It keeps the icons and both backdrops (11 MB). Over budget.
   - Coming back: not measured, the log has whole seconds only.
   Choices made without Sam: the glass is "Blur only" with a light blur; `serde_json` is the one library; shared files and
   icons are copied by root into the app's folder; no shared tree of screen parts until a second screen needs it.
2. **Home, complete.** The status block and weather, top buttons, the bar and grid with scrolling, Recent, folders, open-app dots,
   re-order and the hold-OK menu, captions, toasts, the pointer, in English. Then the Home
   button points at it and Sam uses it daily.
3. **Control Center and Inputs**, in Lumen and as Quick Settings over any app. The brightness schedule.
   **Quick Settings built 2026-11-24, seen working by Sam:** the same program as `com.samrenfrew.lumen.native.quick` (`lumen-quick`,
   `src/side_apps.rs`), an overlay with the web Quick Settings' layout: Sound, Picture, the backlight and Eye Comfort, All settings,
   Screensaver (opens Lumen Screensaver), Screen off, and Restart, which asks for a second OK. A dark corner sits behind it, and the
   glass is a flat dark tint, since an overlay cannot see the app under it. It closes, not hides, on Back, the gear again and 30 s
   without a key: a native cold open should be fast enough that nothing is kept running. `gear.sh` opens it when installed.
   **Sleep timer (Sam, 2026-11-24)** takes the light/dark tile's place: it sets the TV's own `sleepTimer` (category `time`), which the TV takes and reads back. Seen working by Sam.
4. **Live TV.** The row, Now and Next, the Full guide, typed channel numbers, and the Guide over Live TV.
   **Data ported 2026-11-24, not yet drawn:** `native/lumen/src/livetv.rs` reads the channels, Now and Next, the guide
   service and the schedule through root (the app's bus role is public only, so it cannot call `iepg` or `epgdb` itself),
   keeps the last answer in the store's `livetv.json`, and copies the logos the web Lumen keeps in
   `/var/lib/homehook/logos` into the store's `logos/`. A logo new to both apps is copied unsharpened.
   **Screens built 2026-11-24, not yet on the TV:** the Live TV row under the grid (`home.rs`: Guide tile, channel tiles with
   logo or name card and progress, Now and Next under the focused one), the Guide and the Full guide (`guide.rs`, drawing only
   the rows in view and the programmes in its two hours), typed channel numbers, and `{"view":"guide"}`. Kept pictures:
   `home-live`, `guide`, `guide-typed`, `guide-full`, `guide-full-later`, from `fixtures/livetv.json`. Not built: the Guide over
   Live TV as its own app (the code runs when launched as `com.samrenfrew.lumen.native.guide`, but nothing packages it), and
   the live preview beside the Guide.
   **Live previews built 2026-11-24, seen by Sam:** `src/preview.rs` opens `utp.broadcast` on the app's own bus name after 0.6 s on a
   channel, places the picture with `wl_webos_exported_set_exported_window` (the TV reports it at the tile's place, doubled for the 4K
   panel), and cuts a hole once `avmutemanager` says the picture is unmuted. Screenshots show the hole black, so only Sam can see it work.
   **The Guide over Live TV packaged 2026-11-24, seen working by Sam:** `com.samrenfrew.lumen.native.guide` (`lumen-guide`), an
   overlay that opens on Now, or on the Full guide with `{"view":"full"}`. It closes on Back, OK on a channel, LIST again (any
   relaunch) and 30 s without a key, and is never started hidden. `watch.sh` binds LIST and GUIDE to it in Live TV when it is
   installed, in place of the web Lumen Guide. Its panels used to be drawn with no coverage of their own, so they would not have
   shown over Live TV; they now do.
5. **Settings.** All twelve pages, the wallpaper picker, Immich, Side apps, Home and Power, Privacy,
   TV scripts, backup.
   **Built 2026-11-24, not yet on the TV.** `settings_screen.rs` (pages and drawing), `settings_options.rs` (rows and what
   each writes, tested), `settings_tv.rs` (TV state, inputs, side app releases, Immich), `thumbs.rs` (picker thumbnails), and
   the new root commands in `root.rs`. Kept pictures `shots/settings*.png`. Gaps, each a `ponytail:` in the code: no Channels
   page until the Live TV channel list; Home and power-on can be turned off but not pointed at Lumen Native until
   `home-setup.sh` knows its id; home still shows only picked pictures (no plain, Immich or timed change yet).
   **Since then (2026-11-24):** Settings > Channels picks and orders the Live TV row (shown while the row is on and channels are
   known), and the Plain wallpaper draws the web Lumen's soft colour glows.
   **Gaps closed (2026-11-24, (verify) on the TV):** Settings points Home and switching on at Lumen Native
   (`home-setup.sh native-home on com.samrenfrew.lumen.native`, `power-on on ...`); the large weather widget draws on home with
   the screensaver's layout; the backlight fill has the web gradient and rims; channel logos stretched onto a tile are sharpened
   as they are drawn (the web Lumen's 3x copy is not needed, since all the native apps share one store). The five bundled
   wallpapers stay out: the native picker offers only Landscapes and Art, so a `local:N` pick can only come from old settings,
   and it is skipped.
   **Animations and glass (2026-11-24, (verify) on the TV):** Control Center and Inputs shrink back into their button as they
   close (`cc_close_ms`, `cc_close_to`, `cc_close_fade`), and the Guide's rows and cells have a light glass look baked into
   their cached shapes: a soft light along the top and a faint lower rim (`guide_glass`, 0 is the flat look). All four are
   tunable live with `nctl.mjs set`.
6. **The screensaver joins.** `native/saver` becomes a mode of the one program, Lumen's built-in one goes, and the wallpaper and
   the screensaver share the photo picker.
   **Done 2026-11-24.** The screensaver is a screen of Lumen and the same program packaged as Lumen Screensaver
   (`src/saver_mode.rs`). The C app in `native/saver/`, its build and deploy scripts and its place in the release workflow are
   removed; its C files are in the history. The test apps (`com.samrenfrew.lumen.ntest`, `.rtest`) are uninstalled from the
   TV; their source stays in `native/home/` and `native/rtest/` as the record of the spikes.
7. **Ship.** The new ids in the release workflow and the Channel, release notes, the README, `CLAUDE.md` and the `tv` skill
   rewritten for native, the web app tagged and frozen, `app/` kept in the repo as the record.

## What carries over, by part

| Part | Web source | Notes for the port |
|---|---|---|
| Home screen | `screens/Home.tsx`, `components/Row.tsx`, `Tile.tsx`, `Status.tsx`, `Forecast.tsx` | The test app has the bar, tiles, slide and backdrop. |
| App list, launch, recents, order | `lib/apps.ts`, `launch.ts`, `recent.ts`, `order.ts`, `tiles.ts` | Needs spike 2 and 3 (icons). |
| Control Center, Inputs | `components/ControlCenter.tsx`, `lib/tvcontrols.ts`, `inputs.ts`, `brightness.ts` | 21 glass controls: each a cached picture, all reading one blurred copy. |
| Live TV and the Guide | `lib/livetv.ts`, `hooks/useLiveTv.ts`, `screens/Guide.tsx`, `FullGuide.tsx`, `guide/` | The largest logic to port. The Full guide draws only the cells in view. |
| Settings | `screens/Settings.tsx`, `components/SettingsList.tsx`, `lib/storage.ts`, `tvsetup.ts`, `sideapps.ts`, `backup.ts` | Lists of rows on the blurred backdrop: simple to draw, many of them. |
| Wallpapers | `lib/wallpaper.ts`, `wpstore.ts`, `components/WallpaperPicker.tsx`, `config/wallpapers.ts` | Thumbnails go through the picture cache. |
| Immich, the screensaver | `lib/immich.ts`, `components/Ambient.tsx` | Ported into `native/lumen` (`picker.rs`, `compose.rs`, `saver.rs`). |
| Weather, clock, languages | `lib/weather.ts`, `clock.ts`, `i18n.ts`, `config/cities.ts` | Weather ported in `native/lumen/src/weather.rs`. Strings are generated from `i18n.ts`, as `tools/gen_saver_look.mjs` does. |
| Live preview | `components/LivePreview.tsx` | Spike 5. The video wallpaper (`lib/wallpaperVideo.ts`) is not ported. |
| Root commands | `lib/root.ts` | Same shape, behind whichever route spike 2 picks. |
| TV scripts | `tools/tv/watch.sh`, `home-setup.sh` | Kept. They learn the new ids, and lose the work a native app can do itself. |
| Log and boot timeline | `lib/log.ts`, `boot.ts` | Same lines in `lumen.log`, so `startup-perf.md` still reads them. |

## Done means

- Sam uses the native Lumen for two weeks with the web one untouched, and the TV does not restart on its own.
- Every move shows every frame (the readout's "frames of"), on home, in Control Center and in the Guide.
- The memory and start budgets above hold, measured the way `native-saver.md` describes (the driver's own figures).
- The side apps open in under a second.
