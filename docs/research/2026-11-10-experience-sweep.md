# Experience sweep: the code, and other launchers since 2026-11-01

Written 2026-11-10 as a report only; the notes under each item say what was built later that day. Two parts: a read of Lumen's own code for speed and ease of use, and a sweep of
similar projects for anything new since `prior-art.md` was last updated (2026-11-02). Nothing here was tried on the TV.

How to read the labels: **verified** means a second agent fetched the source and tried to refute the claim. **Single source** means one
agent read it once. **Claim** means it is only in a README or a comment.

## Part 1: Lumen's own code

The frame-rate and start-up work is thorough, and most of the usual advice is already done. These are the things still open, best first.

Items 1 to 3 were built the same day, for channel logos only (the app icons stay as they are, at about 170 KB). They are in the Verify list in `docs/plans/PLAN.md`.

### 1. OK waits for the button to come up, even where a hold does nothing

`useOkPress` (`app/src/hooks/useOkPress.ts`) acts on key up, so that a hold can lift an app. Every screen uses it, also the ones that
have no hold action:

- Lumen's Guide passes an empty `onLongPress` (`app/src/screens/Guide.tsx:95`), as does the Full guide
  (`app/src/screens/FullGuide.tsx:103`) and Lumen Guide (`app/src/guide/GuideApp.tsx:108`).
- On Home, only app tiles in the bar and grid have a hold (`Home.tsx:307-317`). The top buttons, the Inputs list, the hold-OK menu,
  folders, the Guide tile and the channel tiles do not, but they wait for key up too (`Home.tsx:286-304`).

`startup-perf.md` saw this as "the key press about 200-290 ms before the switch" when Settings opens, and left it. A press on a remote
lasts about 100 to 250 ms, so each of these OKs is that much later than it needs to be.

Fix: act on key down where the focused thing has no hold action. Where it has one (app tiles, the Settings grids), keep key up, and give
the tile a pressed look on key down so the press is seen at once. Cheap, and it is felt on every OK in the Guide.

### 2. Channel logos and app icons are kept as text in `localStorage`

- Logos are PNG files on the TV. Lumen reads each one through the root shell as base64, sharpens it on a canvas, and keeps the whole set
  as one JSON value (`app/src/lib/livetv.ts:343`, `:389-406`). The code's own comment puts it at 3 MB. App icons from the TV go the
  same way (`app/src/lib/apps.ts:181-214`).
- Lumen Guide is a second app with its own storage, so it holds a second copy. `guide-overlay.md` measured about 3 MB of its 7.8 MB JS
  heap as logos, and a 370 ms stall each time the cache was written.
- Each logo is a long `data:` address on an `<img>`. The browser decodes that text on the page's thread each time a row is made.
- Chromium loads all of an app's `localStorage` on the first read. The first read is the inline script in `index.html`, before the
  wallpaper is painted. Not measured here.

Fix: write the sharpened logos once as files inside the app's folder, as `copyFontIn` does for the font (`app/src/lib/root.ts:272`),
and point `<img>` at the files. The browser then loads and decodes them off the page's thread and keeps them in its picture cache.
`localStorage` keeps only the list of paths. lg-luma made the same move for icons on 2026-11 (commit `2bc5d0a`, single source).
A Channel update replaces the app folder, so the files must be written again after one, as the font is. `keep-alive.md` already lists
"the channel logos out of `localStorage`" as untried; this is the way to do it. Medium cost. **(verify)** that a page may load files
written into its folder after install (the font shows it can).

### 3. Work that runs the moment Lumen comes back

On each return Lumen reads the inputs at once (`app.tsx:309`) and the open apps at once and again 5 s later (`app.tsx:320-325`). The
open-apps read starts a root shell. `keep-alive.md` measured 215 to 330 ms of script before the first full frame of a return, and named
these reads. The Control Center read already waits 4 s for this reason (`app.tsx:365-374`).

Fix: hold both reads until about 1.5 s after the return. The open-app dots would update a moment later. Cheap. Measure with
`tools/tv/perf/return.mjs`.

### 4. The lower rows are still drawn in one go

Still open in `PLAN.md`: rows 3 and on are drawn together 3.5 s after launch (`Home.tsx:158`, `:513`), with a 0.4 to 1 s frozen stretch
after. Lumen is now kept running and started hidden in standby, so a cold start is rare. This matters less than items 1 to 3.

### 5. Small things

- `useLiveTv` returns a new object on every render (`app/src/hooks/useLiveTv.ts:214-215`), and `liveTiles` depends on the whole of it
  (`app.tsx:487`). So the Live TV tiles and all Home rows are rebuilt on every render of the app, a toast included. Fixed the same
  day: `liveTiles` now depends on the five fields it reads.
- Every key on Home renders every row and tile again (`Home.tsx:160`, `:504`). The traces say the GPU is the limit, not script, so
  leave it unless a trace says otherwise.

## Part 2: other projects since 2026-11-01

No project has a technique that beats what Lumen already does. Most commits are translations, tested-TV tables and Home takeover on
other webOS versions.

### Worth a look: the deep dives

Done later the same day. Three agents read the full diffs, not the commit messages. The claims about Lumen's own code were then checked
in the files, and the claims about this TV were read from the TV over SSH (reads only, nothing was changed).

**1. Glasshouse's decode freeze: nothing to copy.** The whole fix in `cb2ab13` is `decoding="async"` on five 480x270 screenshots
(read in the diff). Before: one long task of 300 to 350 ms on a C2. After: a longest frame of 67 to 100 ms. Every `<img>` in Lumen
already has the attribute (checked: ten of ten in `app/src`). The pictures outside it are CSS backgrounds: the four minis on a folder
tile (`Tile.tsx:73`), the wallpaper thumbnails (`WallpaperPicker.tsx:88`) and the screensaver photos (`Ambient.tsx:172`, already loaded
first through `new Image()`). A CSS background cannot ask for async decode. If a trace ever shows a long task as a folder or the
wallpaper picker opens, change those to `<img decoding="async">`. Their frame tool is the same idea as `tools/tv/perf/`: a
`requestAnimationFrame` loop that counts gaps over 33 and 100 ms, plus a `longtask` observer.

**2. Glasshouse's stepped drift: not measured, and Lumen has no drift.** `f4860c8` changes one timing function, from `ease-in-out` to
`steps(360)` over 180 s on a `transform` (read in the diff). That is one new frame each 0.5 s, not 60 a second. The commit gives no
frame, GPU or power number. Keep it as a note for the day Lumen moves a still picture (a slow zoom on the screensaver photo, for one):
start with steps and measure on the TV.

**3. Upstream's boot and wake races: one is a real gap here.** All three diffs were read; the local verdicts are from `tools/tv/watch.sh`.

- *A wake-open over an app you opened* (`6159e4f`). Upstream's loop pushed Lumen over Netflix, opened 41 s into a boot. Its fix: the
  wake only replaces LG's own screens (`""`, `com.webos.*`, `com.lge.*`); any other app in front ends the wake. Lumen has the same
  gap: in the wake window, `watch.sh:264-271` opens Lumen over whatever is in front. The window is 90 s after a boot (`watch.sh:74`)
  and 30 s (`WAKE_SECS`) after a wake, and it applies only while Settings > Home and Power opens Lumen at power-on (`wake_wanted`,
  `watch.sh:164`). So an app opened by hand in the first 90 s of a boot is covered by Lumen. The cost of upstream's fix: a TV that
  wakes into a non-LG app (its last input was Netflix) would stay there, where today "every switch-on opens Lumen" (`watch.sh:7`).
  Fixed the same day, another way: the TV's log names who launched each app (`NL_APP_LAUNCH_BEGIN`, with `caller_id` and `mode`).
  The wake now leaves an app whose launch is newer than the wake and was not the TV's own `last_input` restore (`opened_by_hand` in
  `watch.sh`). An app the TV wakes into still goes to Lumen. In the Verify list in `PLAN.md`.
- *LG's Home opens by itself about 70 s after boot* (`27dbf33`). Upstream reads `/var/log/messages` for a Home launch with
  `"mode":"last_input"` and, if it is under 5 s old, goes back to the last app. This TV has the same handler: its log holds
  `NL_APP_LAUNCH_BEGIN {"app_id":"com.webos.app.home","caller_app_id":"last_input_handler","mode":"last_input"}` (one line, at
  14:56 on 2026-11-10, not at a boot). Lumen acts on LG Home only with `home-takeover` on (`watch.sh:287-290`), which is off, or
  inside the wake window (the case above). So the fix for the first race covers it. **(verify)** what happens 70 s after a real boot.
- *A boot counts as a wake* (`4045991`). Lumen already does this (`watch.sh:74`), and also skips it when the scripts restart on a TV
  that has been up over 5 minutes (`watch.sh:77`), which upstream does not.

**4. LG promotion settings: all six exist on this TV, and four more switches are on.** Read from the TV with `getSystemSettings`:

| Key | Category | On this TV now | What it is |
|---|---|---|---|
| `livePromotion` | `option` | on | Not documented. lgtv-menu says it lightens LG's Home (claim) |
| `webOSPromotionVideo` | `option` | on | Promotion videos on LG's Home |
| `aiNudge` | `general` | on | lgtv-menu calls it pop-up prompts from LG's AI (claim) |
| `adCookie` | `general` | on | The personal-advert cookie. lgtv-menu turns it off too |
| `homePromotion` | `general` | off | Sponsored tiles on LG's Home (Glasshouse's label) |
| `screenSaverAd` | `general` | off | Adverts on LG's screensaver (Glasshouse's label) |
| `contentRecommendation` | `other` | off | Recommendations on LG's Home (Glasshouse's label) |

The key names also appear in settings dumps from G3, G4 and G5 TVs (webOS 9 and 10, chros73/bscpylgtv), all "on" by default. Lumen
sets none of them (no match in `app/src` or `tools`). Each is one `setSystemSettings` call, which Lumen's Privacy page already makes
for other keys. Since Lumen replaces LG's Home and screensaver, the gain is small: less work in the background, and no AI prompts.
What `livePromotion` and `aiNudge` change on screen is unknown. lgtv-menu's "return" script takes a screenshot after a return; it
times nothing. Its `launchtime.sh` times a launch from the `"visible":true` line in `/var/log/messages`, which `return.mjs` already
does better from inside the page.

**5. The web debugger port: open on this TV, and the fix others use cannot work here.** codefarm-eu's `7c13da2` (ported from
Glasshouse) adds one firewall rule, `INPUT -p tcp --dport 9998 ! -i lo -j DROP`, again every 30 s from the watcher (read in the
diff). It has no allow list: tools reach the port through an SSH tunnel, as Lumen's already do (`cdp.mjs:6`, port 19998 on the Mac).
On this TV the port listens on every address (`0.0.0.0:9998`), with no login, so any device on the home network can run script in
any open web app. But this TV's kernel has no `ip_tables` module, so there is no firewall to add a rule to. Other routes, none
tried: turn the inspector off when not in use (it is tied to `/var/luna/preferences/devmode_enabled`, which root access may also
need), or find how WebAppMgr is given the port and bind it to localhost. The risk is the home network only. A spike, not a quick fix.

### Checked, and no use here

- **A compiled-code cache for the app's script.** Chromium keeps one only for http, https and schemes the embedder registers; `file://`
  is left out (verified in `code_cache_host_impl.cc`). Whether LG's build registers `file` is unknown. So a smaller script is the only
  lever on parse time, and with Lumen kept running it is not worth chasing.
- **appinfo.json switches** (`usePrerendering`, `enableBackgroundRun`, `suspendDOMTime`, a V8 snapshot): not in LG's reference. LG also
  says web apps do not render at 4K, so there is no 4K cost to remove.
  https://webostv.developer.lge.com/develop/references/appinfo-json
- **Preload policy retries** (cleanyourtv `89d8863`): Lumen stops the preload manager outright.
- Fixes for webOS 1 to 6 in free-my-tv, vibehack and the Lumen forks: old Chromium.
- Native launchers (lg-luma, luma-webos-launcher, plx-native): a ceiling for smoothness, not something to copy. Native Lumen stays
  parked.

## Part 3: replacing LG's built-in screens

Lumen Guide's method (a second app in an overlay window, kept running hidden, opened by a key bound with Input Hook) is now the common
one. Others use it on webOS 25:

- **cleanyourtv's input picker** (verified): a separate web app with `"defaultWindowType": "overlay"` and `"transparent": true`. Its
  docs say that on webOS 25 it "is started hidden after boot and the key only wakes it: instant". On other versions it starts on demand.
  https://github.com/lasharor/cleanyourtv-WebOS/blob/HEAD/docs/input-key.md
- **Glasshouse's input bar** (verified): the same window. On webOS 6.5, hiding the window left it holding the remote, so it closes after
  each use. Lumen Guide hides cleanly on this TV, so that limit does not seem to apply to webOS 25.
  https://github.com/rorygallagher2024/glasshouse/blob/HEAD/server/lib/inputbar.js
- **cleanyourtv's notification cards** (single source) use `"defaultWindowType": "topui"` on webOS 25. Untried here.

What this means for each built-in screen:

| LG screen | Route | State |
|---|---|---|
| Gear menu (quick settings) | An overlay app that shows Lumen's Control Center over any app, on the gear key | The best next one. See below |
| Input picker | The same overlay app, or a second view in it, on the Input key (code 241 for Input Hook, single source) | Lumen's Inputs list already exists in `Home.tsx` |
| Screensaver | `watch.sh` opened Lumen's, which leaves the app you were in and may lose its place. From 2026-11-11 an overlay app, Lumen Screensaver (`docs/plans/side-apps.md`) | cleanyourtv mounts over LG's saver app instead; the overlay avoids that mount |
| Volume and channel banners | None found. No project replaces them, and nobody documents which app draws them | Leave |
| Full Settings app | LG's is an overlay web app with trusted permissions (webOS OSE source) | Too large; Lumen's Control Center covers the daily part |
| Toasts | LG's `createToast` as root already works and is LG-styled | Leave |

### The gear menu

Built the same day as Lumen Quick Settings (`docs/plans/quick-settings.md`). Question 1 below is answered: the gear is code 139, and a bind stops LG's menu.

Two routes exist. The one tried on 2026-10-26 restyles LG's QML with a mount and a restart of the compositor at each boot
(`spikes.md`, "Gear menu"). Glasshouse dropped its own compositor mount for that reason: the restart tears down the screen, a file that
fails to parse takes the compositor down, and the boot hooks run after the compositor is up (single source,
`docs/IMPLEMENTATION.md`).

The overlay route has none of that. Lumen's Control Center (`app/src/components/ControlCenter.tsx`) already has picture mode, sound
mode, backlight, Eye Comfort, screen off, restart and the screensaver, all through `lib/root.ts`. Built as a second overlay app, as
Lumen Guide is, it would open over Plex or Live TV. From the Guide's numbers: about 0.3 to 0.5 s to show when kept hidden, and about
16 MB while hidden.

Open questions, all for the TV:

1. Which key code the gear button sends to Input Hook, and whether a bind stops LG's own menu. LIST did (`spikes.md`).
2. Whether the overlay should bind in every app, or only where LG's menu is not wanted. Game mode opens LG's Game Dashboard from the
   same key (Glasshouse, single source).
3. Picture mode and backlight are kept per input and per mode. The overlay must read them for the app underneath, not for Lumen.
4. Memory with a third app kept hidden. The TV has about 160 MB free with Live TV, Lumen and the guide running.

## Method

Three research agents (recent commits in twelve repos and new repos; overlay and system-app replacement; web performance on TV
Chromium), then one agent that tried to refute four claims from their sources. All cited links answered on 2026-11-10. The
web-performance agent found little: LG publishes no performance guide, and the rest was Samsung's memory guide, which Lumen already
follows.

Not read: Reddit, the Input Hook source, Glasshouse's `devtools.js`, the PR threads behind the two Glasshouse commits, and the newer repos beyond their file
lists (beeos, sonoma-tv, aurora-webos, CleanFrame-webos and others).
