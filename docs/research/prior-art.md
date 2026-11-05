# Prior art: other custom TV home screens (October 2026)

Research done on 2026-10-24. Star counts come from the GitHub API on that day. Feature claims come from each project's README. Nobody tested them on a TV.

## The wave

In October 2026 a project called cleanyourtv went viral. It removes ads from Android and Google TVs and adds an Apple TV-style home screen. Its author built it by driving Claude Code. It has about 28,800 stars. Many ports and copies followed, for Fire TV, LG webOS and Samsung Tizen.

- cleanyourtv: https://github.com/Gathin23/cleanyourtv
- Press coverage: https://decrypt.co/381456/vibe-coding-ads-out-smart-tvs-heres-how and https://www.androidauthority.com/android-tv-vibe-hacking-remove-ads-bloatware-3725338/

Samsung is a dead end for this kind of work. The cleanyourtv Samsung notes say a Tizen TV gives no shell and the Home button always opens Samsung Home: https://github.com/Gathin23/cleanyourtv/blob/main/docs/devices/samsung-tizen.md

## LG projects that matter to Lumen

| Project | Stars | Root | Notes |
|---|---|---|---|
| [lasharor/cleanyourtv-WebOS](https://github.com/lasharor/cleanyourtv-WebOS) | 80 | Yes | The closest twin. Tested on webOS 25. Live TV, notifications, voice. |
| [rudra-avatar/Hearth](https://github.com/rudra-avatar/Hearth) | 84 | Yes | Credits Claude for most of the code. TV input, LG logo switch. |
| [gprot42/webos-launch-home](https://github.com/gprot42/webos-launch-home) | 25 | Yes | Longest feature list. |
| [zzeppieri/webos-custom-home](https://github.com/zzeppieri/webos-custom-home) | 45 | Yes | Ambient clock, eARC guard. |
| [simbasjungles/free-my-tv](https://github.com/simbasjungles/free-my-tv) | 4,026 | No | Built with Claude Code. Google TV, LG and Tizen. |
| [ecklf/lg-luma](https://github.com/ecklf/lg-luma) | 62 | Yes | Native, not web. tvOS-inspired. |
| [mbsintegrations/LG-TV-HA-Home-Launcher](https://github.com/mbsintegrations/LG-TV-HA-Home-Launcher) | 35 | ? | Home Assistant tiles. |

## Features Lumen does not have yet

1. Settings backup and restore, to the TV and to USB. The backup survives a reinstall. (launch-home)
2. Number keys 1 to 9 open the dock tile at that position. (launch-home)
3. Hold OK on an app for a menu: move, hide, uninstall. (Hearth)
4. Picture notification cards. A local POST, for example from Home Assistant, shows a card with a photo over whatever is on screen. (cleanyourtv-WebOS)
5. Add the launcher as a TV input, so the TV starts on it as its last input. (Hearth)
6. A switch for LG's "Show LG logo when the TV starts" setting. (Hearth)
7. eARC audio guard. Keeps sound on the receiver when its eARC link comes up late at power-on. (webos-custom-home)
8. Clock, date and weather on the home screen, and an idle ambient clock. (Hearth, webos-custom-home, free-my-tv)
9. Voice on the Magic Remote Voice button, answered by a language model or Home Assistant Assist. (launch-home, cleanyourtv-WebOS)
10. A manager that checks each component and rolls it back if it fails to start, plus a clear "go back to LG Home" path. (cleanyourtv-WebOS, free-my-tv)
11. Apple aerial videos as a background or screensaver. (launch-home)
12. Home Assistant tiles: temperatures, switches, energy. (LG-TV-HA-Home-Launcher)

## Patterns across projects

- Reversibility is the selling point. The most popular projects lead with "one button to undo" and a restore script.
- Options that need root are greyed out with the reason shown, not hidden. (Hearth)
- The screensaver is the slot people fill with personal data: revenue, a 3D model of the home, SmartThings rooms, weather.
- A pasted prompt or AGENTS.md file is often the install method.

## From X and the issue trackers

Reddit could not be read: every route was blocked. X posts could not be opened, so these come from search snippets and the repos they link to.

- [OmkarK97/webos-vibehack](https://github.com/OmkarK97/webos-vibehack) (266 stars): the dock glows in the focused app's colour. Pressing Home twice within 3 seconds stays on LG Home, as an escape hatch.
- [NaveenDA/lg-webos-custom-home](https://github.com/NaveenDA/lg-webos-custom-home): built with Claude Code on webOS 9. Disables 18 ad and tracking packages by mounting empty folders over them, with a one-script undo.
- A Home Assistant community post shows a doorbell camera as an overlay on a rooted LG TV. It sends a series of JPEG frames from go2rtc, because older webOS browsers do not play MJPEG reliably: https://community.home-assistant.io/t/video-doorbell-snapshots-as-an-overlay-on-an-lg-webos-tv/1019201 (not read, snippet only)
- A Decrypt article tells how Claude told a user to turn on Developer Mode on a rooted TV. That wiped the Homebrew Channel, and root could not be restored: https://tech.yahoo.com/ai/claude/articles/vibe-coded-smart-tv-claude-175329213.html
- The [cleanyourtv-WebOS issues](https://github.com/lasharor/cleanyourtv-WebOS/issues) report it working on a 43UT7590 and a 65UT7000 with webOS 25.

## Upstream (ammar027/lumen-webos), since our fork point on 2026-10-11

- Native Home selection. Adding `"appCategory": ["home"]` to the app's manifest lets `setDefaultApp` make Lumen the TV's real Home app. The Home button then opens it with no watcher and no Input Hook. Tested on one 65UT8000 on webOS 25. Restore LG Home before removing Lumen. Steps: upstream `docs/native-home.md`.
- The rows slide with `transform`, not `top`. Ours already does this since the Down and Up work of 2026-10-23 (`app/src/theme/global.css:933`).
- A fix for webOS 25: a resting Magic Remote pointer made the rows jump back to the first row. The browser fires mouse-enter on whatever slides under a still pointer. The fix ignores a mouse-enter when the pointer has not moved. Upstream `tools/apply_pointerfix.py`.
- A startup gate: the page stays hidden behind a black splash until the app list, wallpaper and icons are ready, with an 8 second fallback.

## Open questions

- Reddit was not searched, and X replies were not read.
- Free My TV's LG build may have fewer features than its README lists.

## Hearth 2.0 (read 2026-11-01)

Hearth 2.0.0 and 2.0.1 came out on 2026-11-01. Read from the source at commit `ef81da3`, not tried on a TV. Hearth's reference TV is webOS 7.6; nothing below was tested by its authors on webOS 25 unless it says so.

### Home button without Input Hook

Hearth 2.0 does not have a new way to catch the Home key. It ships Input Hook's own files inside the app (`public/homehook/ezinject` and `libinputhookpp.so`, GPL-3.0, credited) and starts them itself from a boot link (`/var/lib/webosbrew/init.d/49-launcher-homekey` running `src/homehook/homehook.sh`). It injects the same library into the same processes (`lginput2`, `micomservice`, `tvservice` and others), uses the same marker (`/tmp/inputhook`) and the same key file (`/home/root/.config/lginputhook/keybinds.json`). If the Input Hook app has already injected in this boot, Hearth does nothing. *Map home button* listens for the Home key code (773, 774 or 125) and writes the mapping.

What it does better than the Input Hook app: at boot it polls for the input processes every 0.25 s for the first 10 s and injects each one as soon as it appears, with one retry if its log does not say `InputHook initialized` within 10 s. If Hearth is uninstalled, the boot link takes Hearth's entry out of `keybinds.json`, so Home never points at an app that is gone.

For Lumen: Sam already has Input Hook, and the hook is the same code, so bundling it gains nothing on this TV. It would only help someone installing Lumen fresh, and it puts a GPL-3.0 binary in an MIT release. The early-inject loop is the only part worth a look, and only if Home is found to do nothing for a while after a boot.

### Opening an app when the TV is switched on

This is the useful one. When webOS powers on, `bootd` picks the first app from `firstAppByPowerOnReason` in `/etc/palm/bootd.json`. That table is how the Netflix button switches the TV on into Netflix. For the power button (`remoteKey`), the TV's own button (`localKey`) and a cold start (`micomPower`) it has no app, so the TV opens the last input. `bootd` reads `/var/luna/preferences/bootd.json` instead of its own file when that exists. Hearth writes a copy of LG's file there with those three reasons set to Hearth (optionally also `wakeOnWiFi` and `wakeOnLan`). Code: `src/lib/poweroncmd.ts`, `src/poweron/guard.sh`, `src/poweron/README.md`.

- The app opens behind LG's start-up cover (`coverUI`), as Netflix does, so Live TV is not drawn first. Claimed in the README; the PR was tested on a webOS 4.5 TV.
- Nothing runs in the background. It takes effect after one full restart, since `bootd` reads the file only when it starts.
- Hearth checks support before offering it: `grep -q /var/luna/preferences/bootd.json /usr/sbin/bootd` and `grep -q firstAppByPowerOnReason /etc/palm/bootd.json`.
- A boot check (`45-launcher-poweron`) removes the copy if the app is uninstalled, or if a firmware update changed LG's file, so an old copy never hides a new one. The write is done aside, read back, then moved into place after `sync`.
- Undo: `rm -f /var/luna/preferences/bootd.json` and reboot.
- Hearth dropped *Add the launcher as a TV input* (`eim/addDevice`) in favour of this, and removes the old input once.

For Lumen: this goes after the "Cold start after a reboot" item in `PLAN.md`. Lumen is a TV input today and `watch.sh` opens Lumen after a wake, but both act after the TV has already picked Live TV, which is why Live TV showed for 10 to 15 s. If `bootd` on webOS 25 honours the copy, Lumen would be the first app the TV opens, and the input and the wake launch could go. Two checks on the TV first, both read-only: the two greps above, and `cat /etc/palm/bootd.json` to see which reasons this TV's table has. Also unknown: whether waking from standby (not a cold boot) goes through `bootd` at all on this TV. If it does not, the `watch.sh` wake launch stays for that case.

### Hearth as the TV's Home app

*Set Hearth as the TV's Home app* writes Hearth's id to the system setting `general.defaultApps.home` through the root shell (an app's own call is refused). Hearth's notes say it worked on webOS 4.5 and not on 7.6, but its compatibility table has an LG G5 on webOS 26 (11.2) where it worked, and Back out of an app went to Hearth. Upstream reported the same method working on a 65UT8000 on webOS 25. A boot check (`46-launcher-homeapp`) puts LG Home back if the app is uninstalled. Code: `src/homeapp/`, `src/lib/homeappcmd.ts`.

For Lumen: this is the Native Home item in `PLAN.md`, now with two more reports that it works on new firmware, and the G5 report says it covers Back out of an app. That is the Exit and app-close problem in "Open on the TV". Worth doing next. The uninstall guard is worth copying.

### Reassigning remote buttons without Input Hook

webOS runs every remote key through a chain of JavaScript key filters inside `surface-manager`, set in configd as `com.webos.surfacemanager.keyFilters`. LG's Netflix and Prime Video buttons are handled there. Hearth puts its own filter (`src/remap/keyfilter.js`) at the front of the chain with `setConfigs` and `"volatile": true`, so it lives in memory only and a restart undoes it. The filter reads its map from a configd key it subscribes to, so a change applies at once. It never takes Home, arrows, OK, Back, Exit, power, volume, channels or numbers. Tested on webOS 4.5 only. Code: `src/remap/`.

For Lumen: Input Hook can already map any key, so this is not needed for Sam. It does have a "press a button" capture mode that tells the launcher which key name a button sends, which would answer the colour-button spike quickly.

### Smaller things

- Keep Hearth ready: bind-mounts a changed copy of memchute's keep list and restarts `memchute`. Lumen already does this through configd (`docs/keep-alive.md`), which survives a reboot without a mount.
- LG preloads: uses configd `supportAppList` plus `setPreloadPolicy` at each boot. Lumen found `supportAppList` ignored on this TV and edits `allowedApps` in the preload manager's file instead.
- Screen saver: bind-mounts its own QML over the screen saver app's QML file. On webOS 10 and later the stock saver is a Flutter app, so it mounts a whole changed app folder and uses `blockSystemApp` and `releaseBlockSystemApp`; untested there. Lumen's watcher approach is fine for now.
- Favourites row (hold OK, *Add to favourites*), order by last opened, and the home row looping from the last app to the first.
- Every root script has an uninstall guard: at boot, if the app is gone, it undoes its own change and deletes its boot link. Lumen's `watch.sh` goes quiet when no Lumen build is installed, but `60-homehook` and the watcher stay.
- A compatibility report issue form that fills a tested-TVs table.

## Other projects since 2026-10-24 (read 2026-11-01)

Read from each repo's source by an agent; "claim" means it is only in a README or doc. Star counts are from 2026-11-01. Nothing here was tried on this TV.

No change: gprot42/webos-launch-home, zzeppieri/webos-custom-home, OmkarK97/webos-vibehack, NaveenDA/lg-webos-custom-home, and upstream ammar027/lumen-webos (last commit Release 1.1.4 on 2026-10-24: a wake tool and clearing LG's update banner).

### The webOS 10 Home takeover

Three rooted projects now replace LG Home on webOS 10 (webOS 25) the same way, and say Home, Back out of the last app and wake all open the launcher with no Input Hook:

1. Bind-mount a staged app over `/usr/palm/applications/com.webos.app.home`, with `appCategory: ["home"]` in its manifest. Glasshouse's code says that without `appCategory` the Home key does nothing on webOS 10 (G5, 10.3.1).
2. Make the app manager read it again with `applicationManager/blockSystemApp` then `releaseBlockSystemApp` (webOS 11 needs `forceBlockSystemApp`). No restart of the app manager.
3. Stop LG's own Home process (`flutter-client -i com.webos.app.home`).

- [rorygallagher2024/glasshouse](https://github.com/rorygallagher2024/glasshouse) (1,579 stars, MIT). New to us, and the source Hearth borrows from. Code: `server/lib/home-takeover.js`, `server/lib/samrescan.js`, boot hook `server/50-tvweb.sh` (sends `releaseBlockSystemApp` first at every boot, so a block cut short never leaves Home blocked). Its tested-TVs list has a **50UT81006LA on 10.2.1 (33.22.56)**, a sibling of this TV, marked "Partial: blocking works; some settings reported not to apply".
- [ecklf/lg-luma](https://github.com/ecklf/lg-luma) (276 stars): the same sequence in `tv/home.sh`, boot hook `64-luma-home`. Claim: confirmed on a B5, C2, C4, NANO77 and UT8050 on webOS 25. Its issue #145/#146: the app manager may not relaunch a Home app that is already in front, so the launcher handles Home itself in that state.
- [rawinek/webos-homebar](https://github.com/rawinek/webos-homebar) (42 stars): the same, with a QML app (claim, tested on a B4, 10.3).
- [lasharor/cleanyourtv-WebOS](https://github.com/lasharor/cleanyourtv-WebOS) (373 stars, v1.6.0 to v1.6.12): mounts over LG Home where it is a Flutter or web app (webOS 24 and newer), but restarts the app manager instead of block and release. Claims webOS 25 on an OLED65G26LA (10.2.2), a 43UT7590PUA and a 65UT7000PUA (10.3.2). One user's C3 still showed LG Home for a few seconds after a full restart.

For Lumen: this is a third way to make Home open Lumen, beside Input Hook and `setDefaultApp`. It covers Exit and Back out of an app, which Input Hook does not. It is also the most invasive (a mount over a system app, redone at every boot). Try `setDefaultApp` first, since it is one setting and easy to undo; use the mount only if that does not cover Exit on this TV.

### Wake and power-on

- Glasshouse (`server/lib/home.js`) sets LG's `homeAutoLaunch` setting off (keeping the old value), and after a wake waits 4.5 s for the TV to bring back its last input before opening Home, then opens it once more if the input comes back. Lumen's `watch.sh` retries every 0.5 s for 30 s instead.
- Glasshouse re-sends `setPreloadPolicy` at every switch-on, because on a C5 (10.3) the preload manager turned every app back on at each switch-on.
- [chiotas/lightkeeper](https://github.com/chiotas/lightkeeper) (79 stars) adds the launcher as a TV input with `eim/addDevice`, as Lumen does. Tested on webOS 6.5 only.

### Other ideas worth a look

- Live preview of the focused input or channel (Glasshouse #1104): HDMI plays as `ext://hdmi:N`, a channel through `service/webos-broadcast-preview` without changing the last channel. Tested on a G3.
- Fall back to LG Home when a wake or boot open shows nothing (Glasshouse #1044).
- Glasshouse #966: LG Input Hook stops surface-manager key filters from working. Matters if Lumen ever uses a key filter beside Input Hook.
- cleanyourtv-WebOS: Back on Home lists running apps with Close all; a full power off button.
- lg-luma #91: let LG's screen saver start; #119: files written while running as root Home keep the app's owner.
- free-my-tv (5,639 stars, no root on LG): a light mode on TVs with under 1 to 2 GB, from `navigator.deviceMemory`.

## Forks of Lumen (read 2026-11-02)

There are 32 forks across this repo and ammar027's. About ten have commits of their own. Most of those make Lumen run on an older TV, so their speed fixes do not apply to this one (Chromium 120): codefarm-eu (webOS 4, an LG C8), hmahesh007, xcv58 and nghoang1288 (webOS 5), and juanrusch (webOS 6). Nothing here was tried on this TV.

Taken into Lumen on 2026-11-02 (see Verify in `docs/plans/PLAN.md`):

- quangtdev: night dimming as a black sheet, not opacity, and a 150 ms head start before a wallpaper fade. They found that opacity on `#wpimage` made the GPU blend both pictures on every frame of a fade, and the fade stalled.
- juanrusch: app names without the store tagline, and a long dock name ends in "…".

Built our own way:

- quangtdev's night backlight in `watch.sh`: it reads the backlight as it is, so it can save the screensaver's dimmed level as the "day" level. Lumen's Settings > Brightness instead keeps the schedule in Lumen and holds the backlight back while the screensaver shows.
- codefarm-eu's screensaver over an app: a key goes back to the paused app. Lumen passes the app in the launch parameters. codefarm-eu later moved this into its overlay bar, which hints that some apps lose their place when they are opened again.

Worth watching:

- codefarm-eu/lumen-webos is the most active fork. It has the Lumen bar (a second app: a dock drawn over the app you are watching), home screen edit mode, a screensaver that moves the clock to stop burn-in, one watcher at a time (a lock in `/tmp`), and watchers started without inherited file descriptors (`60-homehook`). Its newest commits are many small fixes to the bar's icon copying, so the bar is not settled yet.
- Pierregvx (branch `films-series`): a films and series section, AI picks and a player inside Lumen, in one large commit.
- ayushcody (branch `pr/dev-mode-icons`): Lumen on a TV that is not rooted (Dev Mode), and Lucide icons for LG's own apps. Its PR here was closed.
- quangtdev: Vietnamese, and a boot hook that stubs LG's ad and log services (Lumen's `privacy.sh` does the same job differently).

## Check-in 2026-11-05 (commits since 2026-11-01)

Read from commit subjects and messages only, not source. Nothing here was tried on a TV.

No new commits: gprot42/webos-launch-home, zzeppieri/webos-custom-home, OmkarK97/webos-vibehack, free-my-tv, and upstream ammar027/lumen-webos. ecklf/lg-luma has one commit (a preview image).

### Worth trying

- **Live preview of the focused input or channel.** Shipped in Glasshouse 0.137.0 (#1104, with a TV tile in #1066). HDMI plays as `ext://hdmi:N`; a channel plays through `service/webos-broadcast-preview`. Tested on a G3 only. Needs a check that Live TV preview works on this TV (webOS 25). Try it live over DevTools before any build.
- **`spinnerOnLaunch: false`.** cleanyourtv-WebOS set it, with `noSplashOnLaunch`, on its manager app to stop LG's grey splash and spinner (#131, checked on a real TV). Lumen's `app/public/appinfo.json` has `noSplashOnLaunch` but no `spinnerOnLaunch`. One line. Also check any separate Lumen app for the same flash.

### Worth a look, not urgent

- **Homebar 0.19.5 to 0.19.7** (rawinek/webos-homebar): after a wake from standby it sends one HOME; a held BACK closes it as System HOME; Inputs page with USB drives. webOS 11 on a G5, so only loosely relevant, but it matches the Exit and wake behaviour still to verify.
- **cleanyourtv-WebOS add-ons** (v1.6.14, then add-on work on 2026-11-03): wallpaper photo sources and cards (Unsplash, Immich, weather forecast, prayer times) supplied as downloadable add-ons. Lumen already has Immich; the plug-in idea is the only part that might fit.
- **Ad blocker hosts** (Glasshouse #1111): `elastic.lgwebostv.com` (Content Store request logs) and `rdx2.nextlgsdp.com` (sdx report upload on webOS 10). Compare with `privacy.sh`'s list and add any that are missing.

### Skip

- Glasshouse wallpaper Darkening and Pure white theme (cosmetic), and Homebar's Inputs page (Lumen has inputs).
- New small repos since 2026-11-01 (a Chromium port, a no-root launcher, a Turkish widget launcher): nothing relevant.
