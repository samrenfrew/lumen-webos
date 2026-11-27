# One overlay app: Guide, Quick Settings, Screensaver (and later the Dock) under one id

Proposed 2026-11-27, not started. Sam asked whether the side apps are still needed. Items marked **(default)** were picked by Claude.

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
- Below it, a switch per feature: Guide over Live TV, Quick Settings on the gear, Screensaver over apps, later the Dock. These only
  write the flags `watch.sh` reads; nothing is installed or removed.
- The old `.guide`, `.quick` and `.saver` ids are removed by Lumen once the overlay is installed, the same way as the Old Lumen row
  (`home-setup.sh retire-web`).

## Releases

The workflow builds two `.ipk` files instead of four and lists Lumen Overlay in the Channel. The three old native ids leave the
listing. The overlay carries Lumen Native's version, as the side apps do now.

## Steps (each leaves the TV working)

1. Mode from the launch parameter, with the program name still read as a fallback. Tests for the parsing.
2. Package `.overlay` beside the old three in `build_native.sh` and `deploy_native.sh`. Try each mode over SSH with Sam watching.
3. `watch.sh`, `gear.sh` and `saver-mount` use `.overlay` when installed, the old ids otherwise.
4. Settings > Side apps: one row and the feature switches. Removes the old ids. Keep-alive list updated.
5. Release workflow and Channel listing. Old ids and the program name fallback go.

## Open

- **(verify)** A relaunch with a new mode while open reaches the app (`handlesRelaunch` for a native app, `nativeLifeCycleInterfaceVersion`
  2) and the switch is quick.
- **(verify)** LG's screensaver starting while another mode is up. The screensaver should take over, and a key go back to the app.
