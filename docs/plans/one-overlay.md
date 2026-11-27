# One overlay app: Guide, Quick Settings, Screensaver (and later the Dock) under one id

Proposed 2026-11-27. Steps 1 to 5 and the Dock built the same day, on branch `native`; what is left to see is under **Verify**. Sam asked whether the side apps are still needed. Items marked **(default)** were picked by Claude.

## Why there are two ids at all

webOS fixes an app's window type in its `appinfo.json`. Home is a normal full-screen card: opening it sends the app you were in to the
background, and the TV reports Lumen as the app in front. The side apps are `"transparent": true` overlays: the app under them stays
the card in front, keeps playing, and keeps its place (`docs/research/spikes.md`, "Overlay window from a developer app"). One id cannot
be both.

Making everything one overlay id was considered and rejected. Home would then sit over whatever app was last open:

- That app keeps running behind it, holding memory and playing sound behind an opaque home screen.
- `getForegroundAppInfo` names that app, not Lumen, and `watch.sh` (recent apps, the screensaver rules, LIST and GUIDE binds) and
  the keep-alive list all rely on it.
- The Home takeover mount and the power-on launch expect a full-screen app with nothing under it.

So home stays its own id, and the destination it is today.

## What changes

Today the one program is packaged four times (`tools/build_native.sh`): `com.samrenfrew.lumen.native` (home) and `.guide`, `.quick`,
`.saver`. The three overlays become one:

- **One overlay app**, `com.samrenfrew.lumen.native.overlay` **(default name)**, title "Lumen Overlay", hidden from LG's app list,
  `"transparent": true`, `"handlesRelaunch": true`.
- **The mode comes from the launch parameters**, not the program's file name: `{"mode":"guide"}` (with `"view":"full"` for the Full
  guide), `{"mode":"quick"}`, `{"mode":"saver"}`, later `{"mode":"dock"}`. `side_apps::side_of` and `saver_mode::runs_alone` read the
  parameter instead of `argv[0]`. No mode means the screensaver **(default)**, so LG's screensaver slot (below) needs no parameter.
- **One overlay at a time.** A launch while it is open switches mode: the gear while the Guide is up closes the Guide and opens Quick
  Settings. Back, the timeouts and the screensaver's any-key close the whole app, as each does now.
- **One process kept warm.** Today the Guide and Quick Settings are each on the keep-alive list. The merged app takes one slot, and
  the screensaver gets warm opens too. Each mode frees what it loaded on close (the screensaver's photos above all), so the warm
  process stays small. **(verify)** its resting memory against the Guide's today.
- **watch.sh and gear.sh** launch the one id with a mode. Binds still follow what is switched on.
- **LG's screensaver slot** (`home-setup.sh saver-mount`, `docs/plans/native-saver.md`) stages a copy of the overlay app instead of
  the saver app. It runs in LG's screensaver jail, as now.

## Settings > Side apps

- One row, "Lumen Overlay", installs, updates or removes the one app.
- Below it, a switch per feature, so each is easy to turn on and off **(Sam)**: Guide over Live TV, Quick Settings on the gear, Screensaver over apps, later the Dock. These only
  write the flags `watch.sh` reads; nothing is installed or removed.
- The old `.guide`, `.quick` and `.saver` ids are removed by Lumen once the overlay is installed, the same way as the Old Lumen row
  (`home-setup.sh retire-web`).

## Releases

The workflow builds two `.ipk` files instead of four and lists Lumen Overlay in the Channel. The three old native ids leave the
listing. The overlay carries Lumen Native's version, as the side apps do now.

## Steps (each leaves the TV working)

1. Mode from the launch parameter, with the program name still read as a fallback. Tests for the parsing.
   **Built 2026-11-27, seen working on the TV by Claude:** `side_apps::mode_of`. A `mode` in the parameters wins over the program
   name and app id; home's own id ignores it. Nothing launches with `mode` yet, so the TV behaves as before. On the TV, the Guide's
   id launched with `{"mode":"quick"}` drew Quick Settings in 555 ms and closed itself after 30 s; a plain Guide launch still opens
   the Guide. The first try showed the app must join the bus as the id the TV launched, not its mode's id (the TV refused it with
   "Invalid permissions"), so `side_apps::bus_name` takes the launched id.
   **(verify)** In LG's screensaver slot (`saver-mount`, off on this TV now) the screensaver now joins the bus as LG's screensaver
   id instead of `.saver`. That should be the right name there, but has not run.
2. Package `.overlay` beside the old three in `build_native.sh` and `deploy_native.sh`. Try each mode over SSH with Sam watching.
   **Built 2026-11-27, each mode seen working on the TV by Sam:** `appinfo-overlay.json`, program
   `lumen-overlay`, with the screensaver's assets. `LUMEN_OVERLAY=1 npm run deploy:native` installs it. Launched over home with
   `{"mode":"guide"}`, `{"mode":"guide","view":"full"}`, `{"mode":"quick"}` and `{}` (the screensaver), each joined the bus, drew
   in 0.6 to 0.9 s and closed on `closeByAppId`. Sam then tried the Guide over Live TV, the Full guide, Quick Settings and the screensaver, and each
   behaved as the old app does. Right after switch-on the Full guide showed only now and next for a minute, in the old Guide
   too: the TV's own listings were still filling (995, then 470, then 1,602 programmes from the same reads). The release leaves it out until step 5. Its screensaver has no `config/` yet
   (watch.sh copies Lumen's settings only into the `.saver` folders), so it runs on default settings until step 3.
3. `watch.sh`, `gear.sh` and `saver-mount` use `.overlay` when installed, the old ids otherwise.
   **Built 2026-11-27, seen working on the TV by Claude:** LIST and GUIDE over Live TV are bound to the overlay with `{"mode":"guide"}`
   (and `"view":"full"`), the gear runs `gear.sh`, which opens it with `{"mode":"quick"}`, and the screensaver over an app opens it with
   `{"mode":"saver","from":…}`. Its `config/` is kept by watch.sh as the old `.saver` folder's was. A relaunch in saver mode while
   another mode is up turns the app into the screensaver (seen over the Dock). `home-setup.sh saver-mount` stages the overlay
   (`lumen-overlay`) when installed: **(verify)**, as the mount is off on this TV.
4. Settings > Side apps: one row and the feature switches. Removes the old ids. Keep-alive list updated.
   **Built 2026-11-27, seen working on the TV by Claude:** one row, Lumen Overlay, and below it while installed four switches: Guide
   over Live TV, Quick Settings on the gear, Screensaver over apps, Dock over apps. Each runs `home-setup.sh overlay <feature> on|off`,
   which writes or removes `/var/lib/homehook/overlay-<feature>-off`; watch.sh reads them when the app in front next changes (so a
   switch made in Settings applies once you leave Lumen). Dock off and on again wrote and removed its flag. `home-setup.sh
   retire-side`, also run by the guard at each TV scripts restart, uninstalls `.guide`, `.quick` and `.saver` once the overlay is in;
   it ran on this TV. `KeepAlive` now lists `com.samrenfrew.lumen.native` and `.overlay` (the web Lumen's three ids, long uninstalled,
   came out); it takes effect at the next restart.
5. Release workflow and Channel listing. Old ids and the program name fallback go.
   **Built 2026-11-27, not released:** `build_native.sh` packages two apps, the release publishes both and lists Lumen Overlay in the
   Channel; the old side apps drop out of the listing at the next push to `main`. The program knows the overlay only by `lumen-overlay`
   or its id (no mode means the screensaver, which is what LG's screensaver slot launches); `lumen-guide`, `lumen-quick` and
   `lumen-saver` mean nothing now. `LUMEN_OVERLAY=1 npm run deploy:native` installs it with Lumen.

## Open

- A relaunch with a new mode while open reaches the app and switches it (`side_apps::switches_to`); the same mode or none still
  closes it, as a second LIST or gear press does. Seen by Sam 2026-11-27: Guide, then Quick Settings, swapped at once.
- LG's screensaver starting while another mode is up: watch.sh launches the overlay in saver mode, and the app becomes the
  screensaver (`side_apps::become_saver`). Seen by Claude over the Dock on 2026-11-27 with a launch over SSH. **(verify)** a key then
  goes back to the app, and it happens on the TV's own timer.

## Verify (built 2026-11-27, seen by Claude only)

- The Dock with the remote: Home over an app, Left and Right, OK, Down, Up, Back, a second Home, and 10 s with no key.
- Home over the screensaver (over an app) closes it and leaves the app, rather than opening Lumen. Seen by Claude with `home.sh`.
- Home over the Guide or Quick Settings switches to the Dock. Seen by Claude with `home.sh`.
- Settings > Side apps: the four switches, and the gear, LIST and Home following them after you leave Lumen.
- The Dock's bar is a plain dark tint, opaque, with the glass rims and gloss. Home's bar shows the blurred wallpaper through it; over an
  app there is nothing to blur. Whether it should be see-through is Sam's call.

## The Dock over any app (built 2026-11-27)

Built as below, on branch `native`: `dock.rs`, `Home::open_as_dock` and `Home::draw_dock`, and `tools/tv/home.sh`. The Home button
(Input Hook 773) runs `home.sh` while Lumen Overlay is installed and its Dock is on. `home.sh` closes the old Lumen Screensaver if
it is up, passes the press to the overlay if it is up (its screensaver closes, its Dock opens Lumen, its Guide or Quick Settings
switch to the Dock), opens Lumen when Lumen or LG Home is in front, and otherwise opens the Dock over the app, named in `"app"` so
the Dock starts on it. Down, Up and OK on a folder or on Lumen's own tiles (Settings, Inputs) launch Lumen with
`{"view":"dock","col":…,"exit":"down"|"up"|"open"|"bar"}`; Lumen opens with its bar focused on that column and then makes the
move, once it has drawn its first frame, so the slide is seen (29 of 33 frames on the first try; before the wait, 2).

Seen by Claude on 2026-11-27, keys sent over the dev control channel (the overlay listens on 9996, `nctl.mjs`): over YouTube the
Dock came up on YouTube with Live TV's open dot; Right moved to Plex; Down opened Lumen on the tile under Plex in row 2; Up opened
Lumen on Control Center; OK on Plex opened Plex; Back closed it; a second Home opened Lumen at the same tile.

### How fast it opens (measured 2026-11-27)

Measured with a script on the TV that runs `home.sh` over YouTube, closing the overlay between runs: the press, `NL_APP_LAUNCH_BEGIN`
and `NL_VSC visible` in `/var/log/messages`, the process start from `/proc/<pid>/stat`, and the boot line. Every open is a cold
start, as the overlay is not kept running between presses. The Guide and Quick Settings start the same way, so each change below
helps them too.

| Stage | Before | After |
| --- | --- | --- |
| `home.sh` until the launch | 90-130 ms (two TV queries) | 70-90 ms (one) |
| Launch to the process starting | under 15 ms | same |
| Process start to window | 100-230 ms (SDL and EGL) | same |
| Shaders | 375-460 ms, compiled at every start | 4-7 ms, read back from `shader-*.bin` in the store |
| Then the first frame | 40-100 ms | 100-260 ms (the first draw with read-back shaders is slower) |
| First frame to visible | 200-300 ms, the TV revealing the window | overlapped: an empty frame is shown as soon as the window exists |
| **Press to the Dock on screen** | **850-1,100 ms** (2.2 s on the first run after an install) | **about 450-580 ms** |

The shaders are kept by `gfx::Program::new` (`glGetProgramBinary`, keyed by the source and the chip's name), so home's cold start
gains the same 0.4 s. The rest is SDL and EGL starting, the first frame, and the script. Getting under about 0.4 s needs the overlay
kept running between presses: closing would hide its window instead of ending, and a press would be a relaunch. Not tried: it needs
the window to stop taking the remote's keys while hidden.

### The idea, as agreed

Home's dock over the app you are in, so you can switch apps without leaving it. It is a mode of the overlay app and needs it kept warm,
since it is only worth having if it shows the moment Home is pressed.

- **First Home press:** the dock slides up from the bottom over the app, with the same look and the open-app dots. Focus starts on the
  app you are in **(default)**.
- **Which apps:** home's dock apps only, no setting **(Sam)**. A recents choice was dropped, because Down opening full home only makes
  sense when the bar is the top of home. So: one of the dock apps is Home, then OK; any other app is Home, Down, and pick it on home.
- **Second Home press:** opens full home.
- **Leaving the dock row opens full home there (Sam):** Down opens home focused on row 2, Up on the top buttons. Home opens with
  its dock and rows in the same place as the overlay's dock, so the rows look as if they rose up under it. The rows are not drawn
  over the app: that would be home as an overlay again, with its memory and drawing cost over a playing app.
- **OK on an app:** opens it, and the dock drops away.
- **Back, or 10 s with no key** **(default)**: closes it, and the app is where you left it.
- **With home in front:** Home does what it does now, no dock.
