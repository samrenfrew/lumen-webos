# Lumen Guide: a guide drawn over Live TV

Agreed with Sam on 2026-11-07. Items marked **(default)** were picked by Claude when Sam asked for defaults; Sam reviews them on the TV and
changes what is wrong. The rest Sam decided.

## Why it can work

A developer app may open an overlay window on webOS 25: `"transparent": true` and `"defaultWindowType": "overlay"` in appinfo.json. Tested
2026-11-07 over YouTube (a playing video kept playing, Back reached the overlay, focus came back) and over Live TV (the picture stayed in
front). Details in `docs/research/spikes.md`, "Overlay window from a developer app". The idea comes from codefarm-eu's Lumen bar
(`docs/research/prior-art.md`).

## What it does

- **A separate app**, "Lumen Guide", because the window type is per app and Lumen's home screen must stay a full card. It lives in this repo
  and reuses Lumen's guide code. Id **(default)**: `com.samrenfrew.lumen.guide`, hidden from LG's app list (`"visible": false`).
- **Opens on LIST.** LG Input Hook binds LIST to a small script, the way the Home bind works. Over Live TV it opens Lumen Guide. Over any
  other app **(default)** it does the same, and picking a channel opens Live TV on it. While the bind is on, LG's own channel list is not
  reachable from LIST; turning the bind off brings it back.
- **A narrow panel on the left**, full height, just under half the screen wide (about 880px with the rail). The rest of the picture stays clear.
- **A rail at the left edge** with three icons: Now, Next, Full guide. Left from the list moves onto the rail and expands it to show the names
  over the list. Up and Down move on the rail. **OK picks a view**; moving alone does not switch. Right goes back to the list. The rail is the
  only switcher; the list header names the current view.
- **Now** (opens on this): one row per channel with the number, logo, title, start and end time, and a progress bar. No live preview box,
  since the TV picture is already behind it.
- **Next**: the same rows with the next programme's title and start time.
- **Full guide**: Lumen's time grid (`app/src/screens/FullGuide.tsx`), full screen. Back returns to the narrow panel **(default)**.
- **Opens on Now each time, with focus on the channel you are watching**, held on the fourth row as the Guide screen scrolls today.
- **OK on a channel** changes to it and closes the panel. In Next it does the same. In the Full guide, OK on a programme that is on now does
  the same; OK on a later one does nothing **(default)**.
- **Closes** on Back, on LIST again, and after 30 seconds without a key.

## How it looks (defaults)

- The panel is a dark tint with Lumen's glass rims and no blur. An overlay cannot see the video, so it cannot blur it. A soft fade on the
  right edge into the picture, so the edge does not read as a hard box.
- Rows reuse the Guide screen's row styles, cut down to one column. The focused row gets `lguide__row--focused`, so `lumen.where()` can find it.
- Rail icons from Lumen's glyph set: a list for Now, a forward arrow for Next, a grid for Full guide. Collapsed about 96px wide, expanded
  about 280px, drawn over the list rather than pushing it.
- Animate only `transform` and `opacity` (CLAUDE.md). The panel slides in from the left.

## How it is built (defaults)

- A second Vite build from `app/` with its own entry and appinfo, into its own dist folder. The classic script and `chrome79` rules apply.
  A separate build, not a second input, since a classic script build takes one entry.
- **Data from the TV, not from Lumen.** The channel list and programmes come from the same root calls as Lumen's guide
  (`app/src/lib/root.ts`: `channelList`, `nowNext`, `eventList`, `currentChannel`, `openChannel`). Lumen Guide keeps its own cache (its own
  `localStorage`) so a second LIST draws at once, then refreshes.
- `npm run deploy:home` installs both apps. The Homebrew Channel release ships Lumen Guide as a second package later, once it works on dev builds.

## First, a spike

Answer these on the TV before building the panel. Note the answers in `docs/research/spikes.md`.

1. Does the Homebrew Channel run root commands for a second app id? If not, Lumen writes the channel data to a file the guide can read.
2. How long from LIST to a drawn panel, cold? Target: under a second. If it is slow, try a smaller first paint (cached rows, logos later) before anything else.
3. LIST's key code for Input Hook, and that the bind swallows it so Live TV does not open its own list as well.
4. Does Live TV give the remote back cleanly when the panel closes (Up and Down change channel again)?

## Not doing

Recording, reminders, and the "On Now" picture grid from LG's list.

## Status

First build installed on 2026-11-07 (dev build only, not in the Homebrew Channel release).

- **Where it is:** `app/guide.html` and `app/src/guide/`, styles in `app/src/theme/lguide.css`, appinfo in `app/public-guide/`. `npm --prefix app run build:guide` builds it into `app/dist-guide/`. `npm run deploy:guide` installs it over SSH, and `npm run deploy:home` now installs it after Lumen.
- **LIST is bound:** `/home/root/.config/lginputhook/keybinds.json` has `"787"` launching the guide beside the `"773"` Home bind. The file before the change is `keybinds.json.pre-guide`. The spike app `com.samrenfrew.overlaytest` is uninstalled.
- **Seen working over Live TV:** the panel draws over the picture, with the channel on screen focused (ITV1, on row 3 because the list cannot scroll above channel 1). Sam opened it with LIST and tuned to another channel from it.
- **Open time: 1.6 to 2.6 s** from the launch (`NL_APP_LAUNCH_BEGIN`) to the panel's first frame, over four opens. Almost all of it is inside the page: the first frame came 1.4 to 2.5 s after the page started. That is slower than the spike's plain panel. Not yet measured where the time goes (script parse, the font, or reading the cache from `localStorage`). Keeping the guide running between uses was the lever (below).
- **Logos fill on the first open:** the guide has its own cache, so its first open showed channel names in the logo boxes. The logos are cached from then on (149 of them).
- **A second launch closes it:** any launch while the guide is open arrives as a relaunch, and the guide closes on a relaunch (that is how LIST closes it). A test launch over SSH while Sam had it open closed his panel.
- **Language and clock follow defaults:** the guide has its own `localStorage`, so it does not see Lumen's language or 12/24-hour setting.
- **Kept running between uses (2026-11-07):** Back, OK on a channel, the 30 s timeout and LIST while it is up now hide the guide with
  `webOSSystem.hide()` rather than closing it. The app stays in `running` as an overlay, the page is hidden, and Live TV has focus again.
  The next LIST is a relaunch: the TV shows the window again by itself, and the panel mounts afresh on Now. Shown 0.3 to 0.4 s after the
  launch (`NL_APP_LAUNCH_BEGIN` to `NL_VSC` visible), against 1.6 to 5.3 s cold. While hidden it draws nothing; its renderer holds about
  43 MB (PSS), close to Lumen's. The guide is in `com.webos.memorymanager.KeepAlive` beside Lumen (set with `setConfigs`), which the memory
  manager reads only at boot, so it takes effect after the next reboot. Standby still closes it, as it closes Lumen (docs/keep-alive.md);
  the first LIST after a switch-on is then a cold open.
- **Found for binding by app (2026-11-07):** GUIDE is code 362 for Input Hook (`IR_KEY_GUIDE` = 370, minus 8). Input Hook 1.5.1 has the
  actions `launch` (with `params`), `exec`, `replace` and `ignore`, and it reloads `keybinds.json` when the file changes ("Keybinds
  reloaded due to config file change" in its library). Starting the guide with LG's hidden start (`"preload":"full"`) did not help: it
  was not in `running` 6 s later, and the next launch took 5.6 s to show.
- **Only in Live TV (2026-11-07):** `tools/tv/watch.sh` now follows Live TV. When Live TV comes to the front it binds LIST and GUIDE to the
  guide and starts it with `{"hidden":true}`: the page hides before it draws, so the first LIST shows it in 0.3 s. When another app comes to
  the front it unbinds LIST (each app gets its own), binds GUIDE to an `exec` that opens Lumen on its Guide (`{"view":"guide"}`), and
  closes the guide to free its memory. Seen on the TV: the hidden start stayed hidden, LIST showed it and LIST again hid it, and a switch
  to Lumen changed the binds and closed the guide. Not seen: the switch back to Live TV (the TV went into standby mid-test).
  A wake straight into Live TV is not an app change for the watcher, so the guide is not started then and the first LIST is a cold open.
- **(verify)** On the TV: back into Live TV starts the guide hidden and binds LIST and GUIDE again; GUIDE in another app opens Lumen's Guide; the rail position after the fix (the icons sat at the very top in the first build), Left onto the rail and its names, OK on Next and Full guide, Back from the Full guide, the 30 s close, Channel Up working again after the guide hides, and the hidden guide surviving a switch to another app and back once the keep list is live.
