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
- **Opens on LIST, and GUIDE opens the Full guide, only in Live TV.** `tools/tv/watch.sh` binds both keys with LG Input Hook while Live
  TV is in front, and starts the guide hidden. In other apps LIST is left to the app, GUIDE opens Lumen on its Guide, and the guide is
  closed. (The first plan bound LIST everywhere; it changed on 2026-11-07.)
- **A narrow panel on the left**, full height, just under half the screen wide (about 880px with the rail). The rest of the picture stays clear.
- **A rail at the left edge** with three icons: Now, Next, Full guide. Lumen's own Guide uses the same rail (`components/Rail.tsx`). Left from the list moves onto the rail and expands it to show the names
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
- Rail icons from Lumen's glyph set: a list for Now, a forward arrow for Next, a grid for Full guide. Collapsed about 96px wide, 340px open
  with each item as wide as its name, drawn over the list rather than pushing it.
- Animate only `transform` and `opacity` (CLAUDE.md). The panel slides in from the left.

## How it is built (defaults)

- A second Vite build from `app/` with its own entry and appinfo, into its own dist folder. The classic script and `chrome79` rules apply.
  A separate build, not a second input, since a classic script build takes one entry.
- **Data from the TV, not from Lumen.** The channel list and programmes come from the same root calls as Lumen's guide
  (`app/src/lib/root.ts`: `channelList`, `nowNext`, `eventList`, `currentChannel`, `openChannel`). Lumen Guide keeps its own cache (its own
  `localStorage`) so a second LIST draws at once, then refreshes.
- `npm run deploy:home` installs both apps. The Homebrew Channel release ships Lumen Guide as a second package later, once it works on dev builds.

## The spike (answered 2026-11-07)

1. Root commands work for a second app id: the guide reads channels and programmes with the same root calls as Lumen.
2. Open time: 1.6 to 2.6 s cold, 0.3 to 0.5 s when the guide is kept running hidden (Status, below).
3. LIST is code 787 and GUIDE is 362 for Input Hook. Sam opened the guide with LIST and tuned from it.
4. Whether Live TV gives the remote back cleanly (Channel Up after the guide hides) is still in the (verify) list below.

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
  launch (`NL_APP_LAUNCH_BEGIN` to `NL_VSC` visible), against 1.6 to 5.3 s cold. While hidden it draws nothing. Memory is in the
  "Memory" entry below. The guide is in `com.webos.memorymanager.KeepAlive` beside Lumen (set with `setConfigs`), which the memory
  manager reads only at boot, so it takes effect after the next reboot. Standby still closes it, as it closes Lumen (docs/keep-alive.md);
  the first LIST after a switch-on is then a cold open.
- **Memory (measured 2026-11-08):** memory private to the guide's renderer (what closing it frees) is 42 MB while shown. Once hidden it
  falls to 25 MB after 5 s and 16 MB after 25 s, with 7 MB more in swap. The JS heap is 7.8 MB shown and 3.8 MB hidden, and about 3 MB of
  that is the logo cache (149 logos as base64 text). Started hidden and never shown: 20 MB private. So the browser's floor is about 13 MB,
  and the guide's own data can save 3 to 4 MB at most. A native app would save about 10 MB, not worth a rewrite. The TV has 1.2 GB, with
  about 160 MB free and 440 MB in swap while Live TV, Lumen and LG's HbbTV app run.
- **Hidden start fixed (2026-11-08):** a hidden start used to call `webOSSystem.hide()` before the TV had ever put the window up. After
  that, LIST relaunched the page and it drew the panel, but the window stayed out of sight and keys stayed with Live TV. The window also
  waits for the page's first contentful paint and falls back to an 8 s timer, so a page that paints nothing waits 8 s on every show. Now a
  hidden start paints one invisible dot, waits for the window to get focus (10 s at most), then hides, and the page calls
  `webOSSystem.stageReady()` at the start and on each relaunch. Measured with a guide-only dev build: hidden 2.9 s after the launch, first
  show 0.54 s after LIST, the next 0.32 s. `webOSSystem.activate()` did not help.
- **Open speed (2026-11-08):** the panel mounted 140 ms after the relaunch, often before the TV showed the window, and the rows faded in
  390 ms later, once the TV said which channel is on. Holding the slide for that answer still showed the panel before the rows. Now the rows slide in with the panel at once as empty shapes, with blocks for the number, logo, title, times and progress. When the answer comes, the real rows fade in over them as one layer (one track) in 150 ms (`--dur-reveal`) while the shapes fade out, and the shapes are removed 400 ms later. A first try faded each row's content with a class added a render after the rows and removed 400 ms later; Sam saw a second flicker after the content loaded, some of the time.
  A CPU profile of one open showed 370 ms in `localStorage.setItem`: three logo files are missing on the TV, so each open asked for them,
  got nothing, and still rewrote the 3 MB logo cache. `readLogos` now saves only when it added a logo, and does not ask again for a logo
  it did not find while the page runs (Lumen's guide shares it). A showing now asks the TV again 1.2 s after it opens, not at once.
  After these, `setItem` is 11 ms, but the first 1.5 s of an open still has frames of 150 to 420 ms. They are the same with the panel
  not drawn at all (`.lguide { display: none }`), and with logos, shadows or `will-change` layers turned off. So they come from the TV
  showing the overlay window, not from the page. Once open, frames are 17 ms (60 fps); Up and Down were not measured.
- **Lighter tint (2026-11-08):** the panel is 65% black (`--lguide-tint-a`, was 80%), and the eased fade past its edge follows that value. The Full guide's tint is 75% (was 90%).
- **Closed while hidden (2026-11-08):** GUIDE then LIST often left nothing up and the next LIST was a cold start. `ls-monitor` showed
  the memory manager closing the hidden guide: `closeByAppId`, then `killApp` with reason `memoryReclaim`, about 10 s after it hid. The
  guide is in `com.webos.memorymanager.KeepAlive`, but the memory manager reads that only at boot, and the TV had not rebooted since it
  was added. The Full guide's schedule read now stops when the guide hides, so a hidden guide does not grow.
- **Shared rail (2026-11-08):** the rail is `components/Rail.tsx` with its styles in `theme/guide.css` (`.rail`), used by the overlay and by
  Lumen's Guide, which lost its switch at the top right. Each guide sets `--rail-top`.
- **GUIDE opens the Full guide (2026-11-08):** in Live TV, GUIDE (362) launches the guide with `{"view":"full"}` (watch.sh) and LIST
  (787) with no params. The key that matches the view that is up hides the guide, and the other key switches view: LIST on the Full guide goes to Now. Back from a Full guide that
  GUIDE opened closes the guide.
- **(verify)** On the TV: the open feels smooth with the row shapes first and the content fading in; the eased fade on the panel's right edge; GUIDE opens the Full guide and LIST opens Now.
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
