# Keeping apps alive

Goal: switching apps does not end the app you were in. Press Home in iPlayer, then go back to iPlayer: it carries on where it was, with no
launch screen. Lumen itself opens at once. Other apps do not have to stay open for long: going back within a minute or so is the use.

## Where it stands (2026-10-22)

Sam's check at the TV, after the session:
- Lumen opens much faster and with no launch screen at all. But the home screen lags for a moment after it appears, as if it is still
  catching up, so getting back to the last app quickly is still a struggle.
- iPlayer and Plex stay open long enough to go back to them.
- Seen 2026-10-22: iPlayer carries on from the same point, and the running dot looks right. Not reported yet: whether the sound stops
  in the background.

On the TV now (each list is LG's list plus the ids named):
- Lumen is in `com.webos.memorymanager.KeepAlive`: the memory manager never closes it.
- Lumen Guide (`com.samrenfrew.lumen.guide`, the overlay) was added to `KeepAlive` on 2026-11-07 (docs/plans/guide-overlay.md, Status).
- Lumen, iPlayer, YouTube, Plex, Channel 4 and Channel 5 are in `com.webos.memorymanager.KeepOnLaunchEx`: opening another app does not
  close them, but the memory manager still can when it needs room.
- iPlayer's appinfo is patched to `"closeOnBackground": false` (original at `/var/lib/homehook/iplayer-appinfo.backup.json`).
- Lumen is a hand install, not a Channel build: version 1.0.9158 built from the working copy, with `"noSplashOnLaunch": true` and the running
  dot (a dot above each app that is open). Nothing of it is committed. A Channel release replaces it.
- Every other config key has its value from before the session. The whole config's backup is `/var/lib/homehook/configd_db.backup.json`.

The config changes survive a reboot. The memory manager reads them only when it starts, so a change needs a reboot to take effect.

## How the TV closes apps

Found with `ls-monitor` during app switches. The details are in `docs/research/spikes.md` (Keeping apps alive).

- The memory manager (`/usr/sbin/memchute`, `com.webos.memorymanager`) closes the app you were in the moment another app opens:
  `applicationManager/closeByAppId`, reason `memoryReclaim`. It does this with 250 MB free and when the new app asks for 0 MB. It is policy,
  not a lack of memory.
- `KeepOnLaunchEx` protects the app on the list from that close (a soft keep). `KeepAlive` also protects it when the memory manager reclaims
  memory (a hard keep). Opening YouTube makes it reclaim, and it often closes every soft-kept app at once.
- An app can ask to be closed when it leaves the screen: `"closeOnBackground": true` in its appinfo. The app manager then closes it
  (`killApp`, reason `undefined`) whatever the memory manager's lists say. Of the apps in daily use, only iPlayer does this.
- These have no effect: `clearBackgroundApp` (nothing reads it), `applicationManager.keepAliveApps` and `restrictedKeepAliveApps` (they do
  not override the memory manager or `closeOnBackground`), `bindAppProperties` (it only binds Live TV's icons).
- The preload manager starts apps in the background on its own: YouTube, LG Channels and AirPlay among them. It keeps its own policy and
  ignores the config's `supportAppList`.

## Results

Each run is `tools/tv/measure.sh`: reboot, wait 2 minutes, then `switchtest.sh` with `memwatch.sh 2`. Warm means the app was still open when it was opened again.
Memory is tight: 1222 MB in all, about 580 MB of it reserved for video. Usable memory never went below 150 MB in any run.

Standard sequence (Lumen, Plex, iPlayer, YouTube, Channel 4, Channel 5 with Home between, 13 opens):

| Run | Change | Warm | Lowest usable MB |
|---|---|---|---|
| 1 | Baseline | 0 | 168 |
| 2 | LG Channels and AirPlay off the lists | 1 | 202 |
| 3 | Plus preloads off (lost at reboot) | 1 | 194 |
| 4 | Plus Lumen in `KeepOnLaunchEx` | 3 | 174 |
| 5 | Plus Plex in `KeepOnLaunchEx` | 4 | 176 |
| 6 | Plus Lumen and Plex in `KeepAlive` | 6 | 160 |
| 7 | Plus `clearBackgroundApp` false | 6 | 158 |
| 8 | `clearBackgroundApp` back, no launch screen installed | 6 | 159 |
| 9 | Only the two lists for Lumen and Plex, all else put back | 6 | 187 |
| 10 | Lumen hard-kept; iPlayer, YouTube, Plex, Channel 4, Channel 5 soft-kept | 3 | 152 |

Sam's sequence (Lumen and iPlayer in turn, with Plex and YouTube between, 20 opens; `SWITCH_SEQ` in `switchtest.sh`):

| Run | Change | Warm | Lowest usable MB |
|---|---|---|---|
| 11 | As run 10 | 9 | 185 |
| 12 | Plus iPlayer in `applicationManager.keepAliveApps` | 9 | 150 |
| 13 | That undone; iPlayer's appinfo patched | 12 | 168 |

A warm open is 0.4 to 1.6 s to the front, against 3 to 11 s cold. Lumen's own process is about 35 MB in the background (its share of the
shared web engine is not counted).

## Lumen's memory

Measured 2026-10-22, two runs: Lumen in front 30 s, then iPlayer in front with Lumen in the background (3, 13 and 73 s), then Lumen
closed. Usable is the memory manager's figure, the one it closes apps by.

| State | Usable MB | MemAvailable MB | GPU MB (web engine) | Lumen renderer PSS+swap MB |
|---|---|---|---|---|
| Lumen in front | 298-308 | 213-222 | 69-75 | 41+8 (old), 64+0 (fresh) |
| Background 3 s | 269-271 | 221-223 | 111-113 | 33+8, 58+1 |
| Background 73 s | 328-331 | 236-249 | 57 | 33+8, 58+0 |
| Lumen closed | 393-394 | 300-305 | 23-24 | - |

- Lumen in the background costs about 63 MB of usable memory. About 33 MB of it is GPU memory, the rest is its renderer.
- The GPU memory of all web apps shows under the web engine's main process (`WebAppMgr`, GPU in process), in
  `/sys/kernel/debug/pvr/proc_stats/live_pids_stats/process_stats`. Process figures such as RSS and PSS do not include it.
- Lumen's GPU memory is let go slowly: 111 MB just after a switch, 57 MB after a minute, 24 MB with Lumen closed. The 33 MB still held
  is probably drawn layers and decoded pictures. Lumen draws 37 layers, 5.9 screens of pixels, and `.lowerveil` (full screen) and the rows
  (`will-change`) each keep a layer of their own.
- A web renderer with little in it uses about 26 MB (iPlayer's), so Lumen's own share of its renderer is about 10 to 30 MB.
- Inside the page: JS heap 10 MB, 268 elements, 20 images (0.8 megapixels), no canvas or video. `localStorage` holds 3.5 MB of text, of
  which the channel logos are 3.2 MB (`lumen-livetv-logos-v2`, base64 PNG) and the icon cache 0.17 MB. The browser keeps `localStorage`
  in the renderer's memory, so the logos cost there all the time, not only when drawn (estimate: 3 to 6 MB).
- Hidden layers cut 2026-10-22 (verify): the running dots that are off, the dock hint, the veil and the rows fully below the screen are now
  `visibility: hidden` until used, so they get no layer. The home screen went from 38 drawn layers (5.9 screens of pixels) to 13 (3.9).
  Nothing on screen changes; the veil and the third row onwards draw when Down is pressed. Second run of the table above: GPU memory with
  Lumen in front 57 to 60 MB (was 69 to 75). In the background Lumen still holds about 33 MB of GPU memory, so that part is the visible
  layers and pictures, not the hidden ones. Usable memory could not be compared: LG Home and LG Channels were open in the second run.
  Return lag with this build, measured 2026-10-22 by frame gaps in the page (`requestAnimationFrame` from `visibilitychange`, iPlayer
  in front 15 s, launches over SSH): the page catches up in 1.3 to 2.1 s (was 3.1 to 3.7 s by trace, a different method). One run just
  after a cold boot froze for about 7 s. Down pressed about 1 s after the return: one run had stutters of 100 to 320 ms as the rows came
  up, the other was smooth.
- How to measure again: per process, `/proc/<pid>/smaps_rollup`; GPU, the file above; the app's process id, `applicationManager/running`.

## Not tried yet

Most useful first:

1. Lumen's lag on return. Traced 2026-10-22 (three returns from Plex, 3 s and 20 s away): after the first frame, the page draws almost
   nothing for 3.1 to 3.7 s. The cause is the GPU drawing the whole page again (3.3 to 4.7 s of GPU thread work), because the browser drops
   a hidden page's drawn tiles and decoded images. Lumen has 37 drawing layers, 5.9 screens of pixels in all, four of them full screen.
   Not the cause: swap (Lumen had about 13 MB swapped) and Lumen's own 5 s wallpaper release (a 3 s return decoded the pictures again too).
   Smaller parts: Lumen's return handlers run 215 to 330 ms of script before the first full frame (inputs, running apps, time format),
   and memory pressure makes the browser decode the wallpaper a second time.
   Live tests the same day, one return each: hiding `.lowerveil` and the lower rows (kept painted at opacity 0 so that Down is a fade) cut
   the layers to 15 and 3.8 screens, GPU work to 2.2 s, and the freeze to about 2 s. Live blur off as well (its backdrop filter puts the
   tiles over it on layers of their own): 10 layers, 3.2 screens, GPU 1.6 s, freeze about 1.5 s. A smaller stutter at about 5 s is the
   second read of the open apps and the render after it.
   Decided 2026-10-22: no visible change on return. The hidden rows built (`html.away`, rows and veil hidden for 2 s) and Sam saw it at
   the TV: the screen is already drawn when Lumen returns, so rows that vanish and come back look broken, though the freeze is shorter.
   iPlayer flickers the same way as it redraws. The lag only shows if a button is pressed in the first 3.5 s, so an invisible lag is
   better than a visible change. Any fix here must cut the GPU work without changing what is on screen (fewer layers).
2. Trim Lumen's memory footprint. Measured 2026-10-22, see "Lumen's memory" below. The drawn layers are the biggest part, and cutting
   them also shortens the return lag (item 1) with nothing changed on screen. Then the channel logos out of `localStorage`.
3. Done 2026-10-22 (verify): Lumen at boot. When the TV booted into Live TV, nothing started Lumen, so the first Home press was a cold
   start (about 8 s). Now `watch.sh`, once, 60 s after it starts and with the screen on, starts Lumen hidden if no Lumen build is open:
   `applicationManager/launch {"id":...,"preload":"full"}`. Seen on the TV: Lumen runs and Live TV stays in front. The webosbrew boot
   hooks run late (the watcher started about 2.5 min after boot), so Lumen is ready about 3.5 min after boot. Switch off with
   `/var/lib/homehook/no-boot-preload`. Seen by Sam: the first Home press opens Lumen at once, but with a black screen first. The
   preloaded page had fully started (its boot line shows the wallpaper decoded), but it was never drawn on screen, so there is no old
   frame to show while the GPU draws it, as there is on a normal return. The second Home press (back to Live TV, then Home) was much
   quicker, so the black is a first-show cost only. Untried: Lumen's 5 s wallpaper release while hidden probably
   also runs on a page that has never been shown, so the wallpaper is decoded again on the first show.
   Found on the way: `getPowerState` answers `"state": "Active"` with a space, so the watcher's power state was always empty. Its wake
   check by power state never worked on this TV (the gap check did); fixed in the same change.
4. Done 2026-10-22 (verify): the preloads Sam does not use (YouTube, LG Channels, AirPlay) are off. `setPreloadPolicy` returns true but
   does not write the file and is lost at reboot. Instead, `allowedApps` in `/var/preferences/webos-preload-manager-conf.json` has
   `false` for `com.webos.app.lgchannels`, `airplay` and `youtube.leanback.v4`. That holds across a reboot: `getPreloadPolicy` shows
   `isAllowed: false` for the three, and none of them ran in the first 5 minutes after boot. Netflix is also `false`: it has a preload
   entry but is not installed, so the entry only matters if it comes back. LG Home (CRIU), Live TV, HDMI 1, HDMI 3, Live Menu and Voice
   are still allowed; LG Home stays because the power button opens it before `watch.sh` swaps in Lumen. This also stops YouTube getting a running dot before anyone opens it.
   Undone on the TV, seen 2026-10-31: the file was rewritten 2026-10-26 19:08 with `youtube.leanback.v4` and `netflix` back to `true`
   (AirPlay and LG Channels still `false`). Nothing in the repo or the session notes did it. The preload manager writes this file itself
   when its policy changes ("Save configuration"), so something called `setPreloadPolicy` that evening (unknown what: an LG setting, an
   app update, or LG's own services). Set back to `false` in the file 2026-10-31 (backup `/var/lib/homehook/preload-conf.backup-2026-10-31.json`);
   it takes effect at the next reboot. `setPreloadPolicy` with `"isAllowed":false` returns true but changes nothing live.
   Undone again, seen 2026-11-01: rewritten 18:58 (TV clock) with YouTube back to `true`, during the Home takeover tests. Since then
   `watch.sh` sets the apps in its `NO_PRELOAD` list (Netflix, AirPlay, LG Channels) back to `false` at start and after each wake (verify).
   YouTube is left out on purpose: Sam does not mind it preloading.
   LG Home off too, 2026-11-02 (verify): with the Home takeover on, each preload of `com.webos.app.home` shows the stand-in in front, and it
   opens Lumen over the app being watched (seen four times that morning over iPlayer, caller `com.webos.service.preloadmanager` in
   `/var/log/messages`). The stand-in's `document.hidden` check did not stop it. `com.webos.app.home` is now in `NO_PRELOAD`. Set to
   `false` in the file by hand, then `systemctl restart preload-manager.service` (no effect on the app in front).
   That did not hold: by 11:03 the TV had set Home back to `true`, and the preload manager opened Home over iPlayer again at 14:28.
   Since 2026-11-02 (verify) `watch.sh` stops the preload manager outright (`systemctl stop preload-manager.service`) at start, after
   each wake and every 30 ticks. Nothing is preloaded by LG any more, YouTube and Live TV included; `watch.sh` still preloads Lumen. Runs 2 and 3 in the table above show switching the preloads off
   changed little (lowest usable memory 202 and 194 MB, the same warm count). YouTube asks for 135 MB in `supportAppList`, and the
   memory manager closes it when another app needs the room.
   Found 2026-10-31: switching off with the remote (power state `Active Standby`) closes every app but Live TV, Lumen included, with no
   close line in `/var/log/messages`. `KeepAlive` does not protect it. So the first Home press after a switch-on was a cold start (Home
   pressed 5 s after switch-on, Lumen up 6 to 9 s later). About 40 s into standby the TV suspends (`Suspend`, reason `quickStart`). The
   watcher now starts Lumen hidden 10 s into each standby. Lumen loads fully there (its boot line is written) and is still open after the
   suspend: one switch-on later it came up 1 s after LG Home. Not yet measured: a Home press in the first seconds after switching on into
   Live TV. In the one test so far, Sam switched on only 6 s after the preload started; Home then took 9 s, so the TV may also be slow to
   answer while it starts Live TV.
5. Preload our own apps. Adding them to the config did not work, and `setPreloadPolicy` answers "Cannot find" for an app it does not know.
   Read from the binary 2026-10-31 (`/usr/bin/webos-preload-manager`): its candidates come from the config key
   `com.webos.service.preloadmanager.supportAppList` (each app also needs `true` in `allowedApps`), ranked by launch counts in the db8 kind
   `com.webos.statisticsManager.appLaunchHistory:1`, capped by `com.webos.service.preloadmanager.maxPreloadApp`. The conf file also has an
   `applications` key (untried). So Lumen in `supportAppList` was probably not enough on its own: it also needs an `allowedApps` entry, and
   enough launches to rank. Separately, the TV preloads the last input at boot with `/etc/palm/preload-last-input.sh`
   (`preload: partial`, hidden). Lumen is a TV input now, so when it is the last input, the TV already preloads it with no help from us.
6. Keep the iPlayer patch after an iPlayer update: reapply it at boot when the flag is back. Unknown whether the app manager reads the
   change without a reboot.
7. The Recents button: still off on the remote. Now that apps stay open, it is worth turning on again.
8. Freeze instead of close: `com.webos.memorymanager.FreezeTarget` (empty now). A frozen app uses no CPU and can go to swap without
   closing, which may survive YouTube's reclaim. memchute has an `updateFreezeTarget` function, which may mean the list can change without
   a reboot.
9. A cap such as "Lumen plus the last three apps". It needs a watcher that rewrites the lists and restarts the memory manager at each
   switch, since it reads them only at start. Heavy; only if soft keeping turns out too loose.
10. memchute's debug log, to see why it picks the apps it closes.
11. Hibernation (`AppHibernationEnable`, `false` now): saves an app to flash and restores it. Highest risk: it writes to flash, and the
    memory manager has daily flash-write limits (`EfsDailyWrSize`).
12. Lumen as an overlay window, like LG's Settings panel (`"defaultWindowType": "overlay"`, `"transparent": true`), and
    `"usePrerendering": true`. LG may honour them for its own apps only.

## Tools

- `sh tools/tv/switchtest.sh [dwell-seconds]`: opens apps in a fixed order (13 steps, 20 s each) and prints, per step, warm or cold,
  seconds to the front, usable MB and the apps still open. `SWITCH_SEQ="<ids>"` replaces the order. It opens apps on screen.
- `sh tools/tv/measure.sh <out-dir> <name>`: one full run. It reboots the TV, waits 150 s, saves the keep lists and the open apps,
  then runs `switchtest.sh` with `memwatch.sh 2`. Put `<out-dir>` in the scratchpad.
- `sh tools/tv/memwatch.sh [seconds] [count]`: read only. Per sample: time, app in front, the memory manager's level and usable MB,
  `MemAvailable`, swap used, then each open app as `id=RSS+swap` MB.
- `sh tools/tv/variant.sh <in.ipk> '<json>' <out-dir>`: a copy of a built `.ipk` with extra appinfo keys.
- `reboot` over SSH usually brings the TV back on. On 2026-10-22 it stayed off after the preload edit and reboot. Wake-on-LAN
  (`eth0`, `00:a1:59:e8:89:08`, a magic packet to the broadcast address) turned it on, with a cold boot.

## Undo

- A config key: `setConfigs` with its old value. The old values are in the backup above.
- All config: copy `/var/lib/homehook/configd_db.backup.json` back to `/var/preferences/configd_db.json` and reboot.
- Preloads: copy `/var/lib/homehook/preload-conf.backup.json` over `/var/preferences/webos-preload-manager-conf.json` and reboot.
- iPlayer: copy the backup over `/media/cryptofs/apps/usr/palm/applications/bbc.iplayer.3.0/appinfo.json` and reboot.
- Lumen: install a build with `npm run deploy:home`, or update it in the Homebrew Channel.

## LG ad and logging services (read 2026-10-24)

Read over SSH with iPlayer playing. Nothing was changed.

- Memory: MemAvailable 112 MB of 1.2 GB. CMA (kept for video) is 594 MB, with 5 MB free.
- Running apps: iPlayer and Lumen only. LG Home was not running.
- The eight services cleanyourtv-WebOS stops all run here: `remotelogger`, `service-logger`, `contentminer`, `nudge`, `admanager`, `sportsalarm`, `adoverlay-service` and `uploadd`. Each uses about 1.2 MB, about 10 MB together. `acr2` is installed but was not running.
- The large users are the apps. iPlayer's renderer used 119 MB. Lumen's renderer used 22 MB in the background. `ss.apiadapter.pub` used 23 MB, and its job is not known yet.
- Stopping the services would free about 10 MB. That is a small gain.
