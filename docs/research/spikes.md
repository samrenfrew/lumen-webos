# Spikes: answers from the TV

LG 43UT91006LA, webOS 25. One entry per question in docs/plans/PLAN.md "Spikes", with the date and what was run.

## Background apps (2026-10-15)
Opened Lumen, then Plex; with Plex in front, `luna-send -n 1 luna://com.webos.applicationManager/running '{}'` listed lgchannels, LG Home,
YouTube (opened before Lumen), Live TV, HDMI 1 and PlxNative. Lumen was not running. Lumen has no code that closes itself, so the TV ended it,
while an app opened earlier (YouTube) stayed. So it is not "the TV kills every background app"; it kills Lumen. Unknown why (memory, a policy
for some app types, or a keep-alive allow list). Every Home press is a cold start either way.
Possible follow-up: `"keepAlive": true` in appinfo.json. In webOS OSE it makes the app manager suspend an app instead of ending it when
another app opens; LG's webOS TV docs don't list it, and LG's forum answer is that background apps can be ended for memory with no way to
stop it. `grep -l keepAlive /usr/palm/applications/*/appinfo.json /media/developer/apps/usr/palm/applications/*/appinfo.json` found
nothing: no LG app and not youtube-webos uses it, so whatever keeps YouTube alive is not this key. Tried on Lumen 2026-10-16 (dev build with
`"keepAlive": true`): no effect, Lumen still reloads from scratch after Plex. Removed again.

Update 2026-10-22: it is an allow list. The app manager (`/usr/sbin/sam`) reads `com.webos.applicationManager.keepAliveApps` and
`com.webos.applicationManager.restrictedKeepAliveApps` from the config service. As root,
`luna-send -n 1 -f luna://com.webos.service.config/getConfigs '{"configNames":["com.webos.applicationManager.restrictedKeepAliveApps"]}'`
gives `["youtube.leanback.v4","netflix"]`, which is why YouTube stays. `keepAliveApps` is LG Home, AirPlay, the voice apps, Alexa and
`com.webos.app.dvrpopup`. The same call's
`com.webos.applicationManager.lifeCycle` names who may close a background app: the memory manager (`memoryReclaim`) and the window manager
(`windowPolicy`, `userTimeout`). The sam binary has a `skip_to_close_to_keep_alive` path in its close handler. The config service has a
`setConfigs` method and override files in `/var/preferences/` (`configd_debug.json` among them). Not tried yet: add Lumen to either list,
then open Plex and check `running`. The Settings overlay is not kept alive either: it is not in `running` when closed. It only leaves the app
under it running because it opens as a transparent window, not a full-screen one.
The overlay comes from Settings' appinfo.json: `"transparent": true`, `"defaultWindowType": "overlay"`, `"trustLevel": "trusted"`, and also
`"usePrerendering": true`. Other values in use on the TV: `card`, `popup`, `home` (Senior Home), `screenSaver`, `minimal`, `system_ui`.
No store or developer app sets `defaultWindowType`, so it is unknown whether the app manager honours it for Lumen.

## Memory manager (2026-10-22)
Read only, with the TV in use. The memory manager is `/usr/sbin/memchute` (`com.webos.memorymanager`). Its policy is in the config service
too: `python3` over `/var/preferences/configd_db.json`, keys starting `com.webos.memorymanager.`. The ones that bear on app switching:
- `clearBackgroundApp`: `true`. The likely reason the app you were in closes when another app opens. Not proven: the log
  (`/tmp/var/log/legacy-log`) has no line for each close.
- `KeepOnLaunchEx`: LG Home, Senior Home, Google Assistant, Alexa, `com.webos.app.familycare`. Probably apps whose launch leaves the current
  app running, which would explain why LG Home does not end the app you were in.
- `HomeGroup`: LG Home, Senior Home. `KeepOnLaunch`, `FreezeTarget` (empty), `AppHibernationEnable` (`false`) are also there.
- `DefaultRequiredMemory` and `LaunchThreshold`: 120 (MB, by the look of it).

Memory at the time (BBC iPlayer, lgchannels, LG Home and AirPlay running): 1222 MB total, of which about 580 MB is reserved for video (CMA);
182 MB available; swap 900 MB on two flash partitions (`/proc/swaps`, not zram), 433 MB used. `luna-send -n 1 luna://com.webos.memorymanager/getCurrentMemState '{}'` (with `-tt`) gave
`{"currentLevel":"medium","usable":232}`. Other read methods (from `strings`): `getRecordInfo`, `getModeledRank`, `running`.
At boot, `memchute-ready.service` looks for `/home/root/memchute_debug.sh` and skips it when missing.

## Sleep timer (2026-10-15)
`timer/sleep/getRemainTime` from the app: `Denied method call "getRemainTime" for category "/timer/sleep"`. As root, both `getRemainTime`
and `add` (`{"sleepTimeMinute":10}`, then `0`) return `returnValue:true`. Moved to `lib/root.ts`.

## ps on webOS 25 (2026-10-15)
Plain `ps` lists only processes on the current terminal, so a watcher started with `setsid` is invisible to it. Use `ps -ef`, or `/proc`.

## Settings keys (2026-10-15)
Read as root with `settingsservice/getSystemSettings`:
- `picture` / `backlight`: `"100"`. Per picture mode and input (`dimension: {pictureMode: "normal", input: "default"}`).
- `picture` / `eyeComfortMode`: `"off"`.
- `sound` / `soundOutput`: `"tv_external_speaker"` (`dimension: {audioOutput: "default"}`). The other values (soundbar over ARC or optical,
  Bluetooth) are still unknown: switch output in LG's settings and read it again.
Setting `backlight` as root works (80 then back to 100, visibly dimmed, `returnValue:true`). Eye Comfort `"on"`/`"off"` set the same way works (visibly warmer). Sound output: not needed (TV speakers only), so not tested.

## Clock after a reboot (2026-10-15)
`clockfix.sh` moved aside, then `reboot`. After boot `date` gave `Thu Oct 15 21:33:52 BST 2026`: right zone, right time. `clockfix.sh`
deleted. Only a software reboot was tried, not a power cut at the wall. The boot hook started `watch.sh` on its own.

## Live TV channel list and guide (2026-10-17)
Yes. As root: `iepg/getChannelList` with `"channelMode":"Terrestrial"` gives the 188 aerial channels with names, numbers and logo paths
(PNGs under `/tmp/gibbips/logo`); the guide is in `epgdb`, kind `com.webos.service.legacybroadcast.programSCH:4`, joined on
`signalChannelId`. Tuning: `luna://com.webos.service.apiadapter/tv/openChannel {"channelId":...}` opens Live TV on that channel, even
from Home (found from the SSAP gateway's `filtering-uris.js`; Home Assistant's `tv/openChannel` ends up there). Found by watching `ls-monitor` while
`com.webos.app.channeledit` loaded. The full trail and the calls are in `docs/research/live-tv.md`.

## Reboot reason (2026-10-21)
`tvpower/power/reboot` with `{"reason":"restart"}` returns `Invalid reason, invalid param`, so the Control Center's Restart did nothing.
The accepted reasons are in `/etc/configd/layers/base/com.webos.service.tvpower.json` (`rebootReasons`): `rebootByARRegular`,
`rebootByFault`, `rebootByOnRegular`, `rebootByTimeout`, `quickStart`. `lib/root.ts` now sends `rebootByOnRegular`. Over SSH it returned `returnValue:true` and the TV rebooted, but about a minute after it came
back the TV was off and off the network (verify: whether this reason ends in standby, or `rebootByARRegular` is the right one).

Retest 2026-10-24: every `tvpower` reboot comes back in Active Standby. The log line is `NL_POWER_STATE {"old":"Init","new":"Active
Standby","PowerOnReason":"rebootByOnRegular"}`, the same from a TV that was on and one in standby. `rebootByARRegular` does the same (it is the
scheduled overnight reboot). `turnOnScreen` does not wake it ("The current sub state must be 'screen off'"). The shell's `reboot` comes back on
(see Keeping apps alive, below), so `lib/root.ts` now runs that, detached so the exec call returns before the TV goes down (verify).

## Stadia controller over Bluetooth (2026-10-21)
Stadia controller in Bluetooth mode, as `StadiaXR77-xxxx` (vid 18d1, pid 9400). Watched `/proc/bus/input/devices`, `/dev/input/js*`,
`/dev/hidraw0`, the GATT layer (`bluetooth2/gatt/monitorCharacteristic` on service 1812, characteristic 2a4d) and the page's gamepad events.
- A fresh pairing works: presses reach the kernel, the page sees a standard gamepad, and the TV maps it to remote keys, so Lumen needs no
  gamepad code to navigate.
- After the controller is switched off and on (a reconnect, no re-pair), the TV lists it as connected and makes the input device, but no
  reports arrive at any layer, not even GATT. The report notification descriptor (2902) reads `[1,0]`, and writing 0 then 1 to it does not
  bring reports back. So the controller stops sending, rather than the TV dropping them.
- Not fixed by restarting `webos-bluetooth-service` or `bthidmanager`, switching the adapter off and on, or `bluetooth2/hid/disconnect`
  then `hid/connect` (the connect fails: "Failed to connect with remote device"). Remove it in LG's settings and pair again (Stadia + Y
  for 2 s, orange light) to make it work. A TV reboot was done at the same time as a re-pair, so it is unknown whether a reboot alone helps.
- After the first re-pair, `bthidmanager` logs `type=not_support_gamepad` for the controller, and the TV shows a "may be incompatible"
  message. Presses still reached the kernel on that pairing, and the label was not logged for the pairing after the reboot.
- Before the reboot, the web engine (WebAppMgr) also kept a stale gamepad open: the page missed disconnects for up to 27 minutes and never
  saw the new controller. Not seen after the reboot.
- Re-pair from the command line, no LG settings (2026-10-21): with the controller in pairing mode, `adapter/unpair {address}`,
  `adapter/startDiscovery` (held open with `luna-send -i`), `adapter/pair {address, subscribe:true}` (answers `request:"endPairing"`), then
  `hid/connect {address}`. Reports arrived on `/dev/hidraw0` and the page saw live gamepad input. The TV did not turn it into remote keys,
  but `bthidmanager` had been restarted earlier that night and logged nothing after it, so retest on a fresh boot. Plan: a TV-side watcher
  next to `watch.sh` that runs this when the controller drops, so a mid-game drop needs only Stadia + Y. Order matters: run
  before pairing mode, `adapter/pair` "succeeded" at once and no input device came.
- Next test on a fresh boot: controller on, check it navigates; switch it off and on, check it breaks; run the re-pair, check navigation.
- Retest 2026-10-22 (TV up 1 h 40 min, `bthidmanager` not restarted, controller `StadiaXR77-b138`): after a reconnect, no reports on
  `/dev/hidraw0`. Protocol Mode (2a4e) already read 1 (report mode). Writing 1 to it and 1 (exit suspend) to the HID Control Point (2a4c)
  through `gatt/connect` then `gatt/writeCharacteristicValue {clientId, service, characteristic, value:{bytes}}` was accepted and changed
  nothing. Then `tools/tv/stadia-repair-test.sh` with the controller in pairing mode: 1584 bytes of reports, and the TV navigates again.
  So the command-line re-pair is a full fix, and the watcher plan stands. Repeated once (power-cycle to break, re-pair): works again.
- The TV has no `bluetoothctl`, `btmgmt`, `btmon`, `hcitool` or `/var/lib/bluetooth`, so BlueZ-level fixes (clearing the GATT cache or the
  bond file) are not open from the shell. LG's supported controller list does not include Stadia.
- Other hosts (2026-10-22 research): the same fault (reconnects, no input until re-pair) is reported for Stadia on Samsung TVs and Android
  TV (forum snippets, unread). On Linux, switching the host adapter off and on fixed it (Lakka-LibreELEC #1824, Steam Deck posts); that
  did not help here. Google publishes no Stadia firmware notes, and no host setting is known to fix it. The likely cause is on the
  controller side: it keeps state from the first pairing and stops sending on a reconnect, which a new bond resets.
- Built 2026-10-22: `tools/tv/stadia.sh`, the watcher. Only while Moonlight (`com.limelight.webos`) is in front. `device/getStatus` also
  lists unpaired devices found by a scan, and a device's name and address are far apart in its object, so it splits per device first.
  It tells you what to do with an LG alert (`com.webos.notification/createAlert`, title, message, an OK button), which stays up until
  `closeAlert {alertId}`; the watcher closes it once the controller works. Toasts (`createToast`) show over apps too, and `<br>` breaks a
  line in them. Both need `sourceId` set to an installed app id (Lumen's).
- A factory reset of the controller (Stadia button 10 s, unplugged), done on 2026-10-21, did not stop the fault.
- To recheck: each "no reports" result above relies on buttons being pressed during the read. A clean round, with Sam pressing and watching
  the screen on every step, would rule out a missed press. Over Moonlight, an alert leaves Moonlight the foreground app,
  and Moonlight reads the controller straight from `/dev/hidraw0` (and the remote from `event9`), not through window focus, so the alert
  cannot take the controller's input away from the game (seen 2026-10-22).
  A virtual Start press to pause the game is not built: it needs a uinput helper (`/dev/uinput` exists), and Moonlight would see it as a
  second controller.
- Mid-game drops (2026-11-02): `stadia.log` shows three drops on 2026-10-31 (22:16, 22:35, 22:38), each fixed by Stadia + Y. LG's
  system log (`/var/log/legacy-log`) rotates about hourly, so their cause was already gone. One lead: Bluetooth and Wi-Fi share one
  MediaTek MT7668 chip on an internal USB link. On 2026-11-02 at 10:37 a wake from standby gave "xHC error in resume", the chip failed
  `HCI_RESET`, the Bluetooth service crashed and the chip was reset (`/mnt/lg/cmn_data/btsvc/bt_history` says `ERROR_HCI_TIMEOUT`).
  The TV also rebooted on a fault (`PowerOnReason: rebootByFault`) about 20:19 on 2026-11-01. Not yet tied to a drop.
- The stack is LG's Fluoride (Android's Bluetooth stack), not BlueZ. The wait before a dead link counts as dropped is the LE supervision
  timeout, agreed with the controller at connect time; no TV config sets it. A USB Bluetooth adapter will not help: the kernel has no
  generic `btusb`, only LG's module drivers (`btmtk_usb`, `btmt7921_usb`, `rtk_btusb`). `bt_stack.conf` has the stack's own log and HCI
  snoop switched off (`BtStackLog2File`, `BtSnoopLogOutput`); turning them on would record each disconnect's reason. Not tried.
- Built 2026-11-02: while Moonlight is in front, `stadia.sh` copies the Bluetooth, USB and HID lines of LG's system log to
  `/var/lib/homehook/bt.log` (UTC times; `stadia.log` is local time), trimmed to 20 000 lines past 4 MB.
- Changed 2026-11-04 (not yet seen on the TV): a re-pair try used to count as working whenever the input device was there after it. If
  `unpair` leaves a connected controller up (not yet known), the old dead input device is still there, so the try logged "re-paired",
  toasted "Controller paired." and stopped watching, and only a drop (Stadia + Y) started a real repair. Now a try first waits for the
  old input device to go, calling `hid/disconnect` if `unpair` did not drop it, and skips the try if it stays. The check alert has
  Dismiss and Reconnect; Reconnect's `onclick` runs `touch /tmp/stadia-reconnect` through the Homebrew Channel's exec service, and the
  watcher starts the repair at once. Whether the notification service may call that service is unknown.

## Keeping apps alive (2026-10-22)
The plan and the measure are in `docs/keep-alive.md`. Each run: reboot, wait 2 minutes, then `switchtest.sh` (13 opens) with `memwatch.sh 2`.
- Who closes the app: watched with `ls-monitor` during Lumen, Plex, Lumen. When an app launches, the app manager tells the memory manager
  (`setRequiredMemory`), and memchute at once calls `applicationManager/closeByAppId` on the app you were in. The close reason is
  `memoryReclaim`, even when the new app asks for 0 MB and 250 MB is usable. So it is policy, not a lack of memory.
- `clearBackgroundApp`: no effect when set to `false`. memchute and sam do not contain the string, so nothing seems to read it.
- `restrictedKeepAliveApps` (sam's list) does not protect from memchute: YouTube is on it and was closed in every run.
- `KeepOnLaunchEx` plus Lumen: Lumen stays running in the background and every Home open after the first is warm (0.5 to 1.4 s, against
  3 to 7 s cold). It protects the app on the list, not the app you leave: Plex was still closed. Lumen's own process is about 35 MB in the
  background (RSS plus swap; its share of WebAppMgr is not counted).
- `KeepOnLaunchEx` plus Plex: Home no longer closes Plex, but a heavy launch (YouTube, iPlayer) still closed Lumen or Plex.
- memchute's own `com.webos.memorymanager.KeepAlive` (LG's HbbTV apps before) plus Lumen and Plex: every open of either after its first was
  warm, 6 of 13, the most this sequence allows. Lowest usable 160 MB, level never past `low`.
- These config changes survive a reboot (checked with `getConfigs` after each boot) and take effect only after one: memchute reads its
  policy when it starts.
- Preloads: the preload manager ignores `isEnabled` in the config's `supportAppList`, and it did not add Lumen or Plex from it. It keeps
  its own policy (`preloadmanager/getPreloadPolicy`). `setPreloadPolicy {"application":{"id":"airplay","isEnabled":false}}` works, but it
  is lost at reboot. `/var/preferences/webos-preload-manager-conf.json` holds `allowedApps`. AirPlay is also started by its own service
  (`airplay.service`), not only by preloads.
- Removing LG Channels and AirPlay from the memory manager's lists and turning off preloads freed about 25 MB at the lowest point
  (168 to 194 MB usable) and changed no warm count.
- The built-in Chromecast is three small services (2 to 3 MB). Left alone.
- `reboot` from the shell brings the TV back on and it stays on (unlike `tvpower` `rebootByOnRegular`, above).
- Follow-up the same day: `KeepOnLaunchEx` alone is a soft keep (memchute still closes the app when it reclaims memory). BBC iPlayer has
  `"closeOnBackground": true` in its own appinfo, so the app manager closes it as soon as it leaves the screen, whatever the lists say;
  `applicationManager.keepAliveApps` does not override it. `bindAppProperties` only binds Live TV's icons and titles. Patching iPlayer's
  appinfo to `false` keeps it open (results in `docs/keep-alive.md`).

## Gear menu (Quick Settings) (2026-10-23)
- The gear menu is not a web app, so DevTools cannot reach it. `com.palm.app.settings` does not run when it opens.
- The compositor, `surface-manager`, draws it from plain QML source in `/usr/lib/qml/QuickSettings` (package `com.webos.app.quicksettings`, 165 QML files, nothing compiled). The main panel is `Containers/QuickSettingsMainView.qml`; its colours come from `Utils/Style.qml`.
- `com.webos.app.inputcommon` is a different native QML app (Live TV banners, popups, LG's screensaver), in `/usr/palm/applications/com.webos.app.inputcommon/qml`.
- A changed copy works: copy the file to `/tmp`, edit it, `mount --bind` it over the original, then `systemctl restart surface-manager-daemon.service`. The panel turned purple. The mount also shows in each app jail, and it is gone after a reboot.
- The compositor reads the file only at start, so each change needs that restart. The restart blanks the screen for a few seconds and closes Lumen. To make a change permanent, the boot hook would add the mount. The compositor probably starts before the hook runs, so the hook would also restart it once (not tested).
- 2026-10-26: Lumen's glass on the gear menu's buttons works (a sheen and rims from `LinearGradient` in `QtGraphicalEffects`, which the TV has, and a 1.08x grow on focus). Sam: looks good, needs tweaks. Five files changed; the main one is `Component/BaseControlPanelButton.qml`. LG's QML is not kept in this repo.
- The whole screen, the gear menu included, can be captured with `luna://com.webos.service.capture/executeOneShot` (`"method":"DISPLAY"`). Opening the gear menu over SSH is not solved: `com.webos.app.quicksettings` is not an app id.

## Power-on and the Local Control Panel (2026-10-24)
- The menu that came up after the compositor restart on 2026-10-23 is LG's Local Control Panel, `com.webos.app.tvhotkey` (native QML). It is not the service menu (`com.webos.app.factorywin`, EZ Adjust and InStart, which asks for a PIN). Launched with no `activateType`, it opens its main menu.
- Its toggles write ordinary settings. "Show LG logo when turning off the TV" is `other` / `lgLogoDisplay` (`"on"` or `"off"`); "Screen Saver" is another toggle in the same list (`Interfaces/CustomerSupportInterfaces/CustomerSupportInterfaces.qml`). Set as root with `settingsservice/setSystemSettings`. `lgLogoDisplay` was put back to `"on"`; it did not cause the Live TV problem below.
- Wake-on-LAN resumes the TV from Suspend straight into Live TV. LG Home never opens, so a wake rule that only replaced LG Home did nothing. `watch.sh` now opens Lumen over whatever is in front after any wake.
- Remote power-on with LG's "Home on power-on" (`general` / `homeAutoLaunch`) `"on"`: LG Home holds the screen for about 7 s, whatever is launched over it (about 9 s to Lumen). With it `"off"` (set 2026-10-24), nothing opens first and Lumen is in front about 5 s after the wake.
- `luna-send` over SSH prints nothing without `ssh -tt`, unless its stdin is redirected (`</dev/null`). It looks like a dead bus but is not.
