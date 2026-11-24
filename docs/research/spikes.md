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

## Overlay window from a developer app (2026-11-07)
Answer: yes. The app manager honours `"defaultWindowType": "overlay"` for a developer app on webOS 25.
Test app `com.samrenfrew.overlaytest` (one HTML file, a translucent pink bar at the bottom, closes itself after 20 s). Its appinfo.json
copies codefarm-eu's Lumen bar: `"transparent": true`, `"defaultWindowType": "overlay"`, `"handlesRelaunch": true`,
`"noSplashOnLaunch": true`, `"spinnerOnLaunch": false`, `"disableBackHistoryAPI": true`, `"visible": false`. No `trustLevel`.
Installed over SSH with the dev install service, then launched over YouTube (on its browse screen, no video playing):
- `running` listed YouTube as `card` and the test app as `overlay`. `getForegroundAppInfo` still gave YouTube.
- `luna://com.webos.service.capture/executeOneShot` (`"method":"DISPLAY"`) showed the bar drawn over YouTube, with YouTube visible through it.
  `com.webos.service.tv.capture` (the webOS 4 name) does not exist on webOS 25.
- Over DevTools, the overlay page had `document.hasFocus()` true, and YouTube had false but stayed `visible`. After the overlay closed
  itself (`window.close()`), it left `running` and YouTube had focus again.
- Over a playing YouTube video, Sam pressed Back on the remote. The overlay closed about 4 s after launch (its own timer is 20 s), so the
  key reached it, and YouTube had focus again. The video's `currentTime` rose about 1.1 s per second the whole time and it never paused.
- Over Live TV (2026-11-07, BBC One): the bar drew over the picture, `running` listed Live TV as `card` and the bar as `overlay`, and Live TV
  stayed in front. The TV's screenshot showed the picture under the bar.
Not checked yet: HDMI underneath.

## Lumen Guide spike (2026-11-07)
The four questions in `docs/plans/guide-overlay.md`, answered with the overlay test app (a dark 880px panel on the left that runs root
calls and writes each key it gets to `/tmp/ot-keys.log` through the Homebrew Channel).
1. **Root from a second app: yes.** `org.webosbrew.hbchannel.service/exec` ran `id -u` as `0` for `com.samrenfrew.overlaytest`. Over
   Live TV, the aerial channel list (323 KB) came back in about 1.0 s and Now and Next (`epgdb` `programPF`, 66 KB) in about 0.7 s.
2. **Open time: 0.7 to 2.5 s, cold**, from `NL_APP_LAUNCH_BEGIN` to the overlay's `NL_VSC` visible in `/var/log/messages`. The slowest
   was the first launch after an install. Most of it is before the page starts (navigation start came 2.2 s after the launch on the slow
   one; the page painted 0.25 s later). A launch while the app is already open is a relaunch and costs nothing visible. So keeping the
   guide running between uses is the lever to try first. Memory at the time: about 160 MB available, 450 MB of swap in use.
3. **LIST is code 787 for Input Hook** (`IR_KEY_PRLIST` = 795 in `/usr/share/X11/xkb/keycodes/lg`, minus 8). Input Hook 1.5.1 reads
   `/home/root/.config/lginputhook/keybinds.json`. `"787":{"action":"launch","id":"…"}` opened the test app over Live TV and LG's own list
   did not open. Each further LIST while it was up arrived as a relaunch (`webOSRelaunch`).
4. **The remote goes back to Live TV: yes.** While the panel was up it got every key (Up 38, Back 461) with focus. After Back closed it,
   Channel Up changed channel. The up arrow never changes channel on Live TV, open panel or not; testing with it first looked like a fault.
LIST was bound back to nothing after the spike (`keybinds.json.pre-list`). The test app is still installed (hidden).

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
- Changed 2026-11-04 (not yet seen working in Moonlight): a re-pair try used to count as working whenever the input device was there after
  it. If `unpair` leaves a connected controller up (not yet known), the old dead input device is still there, so the try would log
  "re-paired" and stop watching. Now a try first waits for the old input device to go, calling `hid/disconnect` if `unpair` did not drop
  it, and skips the try if it stays.
- The drop on 2026-11-03 22:01:53 was missed: `bt.log` shows a new input device (`9400.0002` to `.0003`), so the link dropped and came
  back inside one 2 s poll, dead, and `stadia.log` shows nothing until Stadia + Y at 22:04:36. No USB or chip error, `bt_history` still
  ends at 11-02, and the TV is wired only. The watcher now reads the input device's instance number and treats a change as a reconnect.
- An alert button cannot start a root command: `createAlert` with `onclick` set to the Homebrew Channel's `exec` did nothing (2026-11-04,
  no refusal in the log). Launching an app from the alert would take Moonlight's foreground, so the watcher instead reads the remote:
  `/dev/input/event1` ("LGE RCU") sends key 28 for OK and 412 for Back (16-byte events, 32-bit timeval); `event9` sent nothing. While the
  check alert is up and the controller is unproven, an OK press starts the repair.
- First Moonlight test (2026-11-04): run 1 worked, 22 s from OK to working. Run 2's first try failed after 44 s with "not in pairing mode
  yet", and the second try (after Stadia + Y again) worked. Cause, a guess: the try starts on the OK press, before Stadia + Y is held, and
  paired after a fixed 4 s scan. Now a try scans for up to 30 s and pairs when the controller's address shows in `device/getStatus` with
  `"paired":false` (it does so only in pairing mode, also a guess), and logs that entry. `getStatus` also lists neighbours' devices and
  old scan results as `"paired":false`, so a stale entry for the controller would make this pair early again; the log line will show it.
- Second Moonlight test (2026-11-04 21:23): that stale entry happened. The scan matched "after 0s" with `"scanRecord":[]` and `"adapterAddress":""`
  (the paired entry has scan data), the script paired before Stadia + Y, an input device (`9400.000A`) appeared, and it toasted "Controller
  paired." while the controller was still orange and sent nothing. `tools/tv/stadia-repair-test.sh` in pairing mode fixed it (2112 bytes).
  Now a try needs a scan entry with data (a guess that a live one has it; after 20 s it pairs anyway and logs the entry), and a repair no
  longer counts as working until a report arrives: the check alert comes back after it, and B on the controller confirms.
- HCI snoop on (2026-11-06): `BtSnoopLogOutput=true` and `BtSnoopMaxLogFileSize=20000000` in
  `/mnt/lg/cmn_data/btsvc/fluoride-mtk/conf/bt_stack.conf` (original kept as `.orig`, survives a reboot), then
  `systemctl restart webos-bluetooth-service`. The log is `/var/log/bt_snoop_hci.cfa_MTK` (in RAM, `/tmp/var/log`), standard btsnoop, and
  `tshark` on the Mac decodes it down to ATT (`brew install wireshark`). A working link shows as a stream of Handle Value Notifications on
  handle 0x0040. The service restart drops the controller. The first capture missed a clean reconnect: `stadia.sh` unpaired the controller
  4.5 s after the drop, before the TV had reconnected. Capture a reconnect with `touch /var/lib/homehook/no-stadia` first.
- Reconnect captured (2026-11-06 21:37, watcher paused, power-cycle with no re-pair). This overturns "the controller stops sending" above:
  - The TV reconnects on its own, the controller asks for security, the TV turns on encryption with the stored key, and it works.
  - The controller then sends button reports as ATT notifications on handle 0x0040 (real D-pad and button values, about 230 in 10 s of
    presses). Over the same seconds `/dev/hidraw0` got 0 bytes. So the TV receives the input and drops it inside its own stack.
  - The kernel made the uhid input device at 21:38:01.205, before any GATT traffic, so the HID host set it up from its cache.
  - The controller sends a Service Changed indication (handle 0x0003) on every connection, fresh pairings too. On a fresh pairing the TV
    then re-reads every service and writes the HID report CCCD (0x0041). On the reconnect its re-read stops after Device Information
    (last request 0x0032..0x0037, "Attribute Not Found") and never reaches the HID service (0x0038 on) or writes 0x0041. Notifications flow
    anyway, since the controller kept its CCCD. Working guess: the Service Changed indication makes LG's HID host drop its handle map and
    wait for a re-discovery that never ends, so it throws away every report.
  - `bluetooth2/gatt/discoverServices {address}` on the dead link answered "GATT service discovery failed" (168) and sent nothing over the air.
  - LG's system log has only the kernel's uhid lines for this.
- Stack trace on (2026-11-06 21:41): `BtStackLog2File=true`, `BtStackMaxLogFileSize=20000000`, log at `/var/log/bt_stack.log_MTK` (in RAM).
  It shows the cause, in LG's Fluoride (`bta_hh` is the HID host, `bta_gattc` the GATT client):
  - On a reconnect, the HID host loads its saved report list ("Loaded 2 reports"), skips discovery ("discovery has been done"),
    registers for report ID 3 notifications, and opens the uhid device. All within 0.1 s of encryption.
  - About 1.7 s later the controller's Service Changed indication arrives for 0x0001..0xffff. `bta_gattc` throws away its cache and
    re-discovers the whole controller. The HID host gets the Service Changed (event 15) and discovery-done (event 8) callbacks and does
    nothing with them ("Unhandled event: 15"). It never registers again.
  - From then on each report reaches `gatt_client_handle_server_rsp opcode: 0x1b` and stops. Working: the same line, and the report reaches
    hidraw. Status 0x8d on those lines is only the link's encryption state ("encrypted, no MITM"), the same in both.
  - Working guess: the Service Changed clears the GATT client's notification registrations, as AOSP does, so the reports have no listener.
    On a fresh pairing (LG settings, 21:48) there was no cache, the HID host ran its own discovery and registered last, and it works.
  - So it is an ordering bug: a Service Changed that lands after the HID host registers kills input until something makes it register again.
    A re-pair does that. Other hosts with the same Stadia fault (Samsung, Android TV) are likely the same Fluoride code.
  - Next test: on a dead reconnect, subscribe to handle 0x0040 afresh through `bluetooth2/gatt/monitorCharacteristic` and see if values
    arrive. The 2026-10-21 try may have hit 0x0044, the other 2a4d (output) report. If they arrive, a TV-side bridge can feed them into a
    uhid device, and a reconnect needs nothing from Sam.
- Bridge test (2026-11-06 21:55, passed). LG's GATT API delivers the reports on a dead reconnect:
  - Calls: `gatt/discoverServices {address}`, then `gatt/connect {address}` (gives `clientId`, "009"), then
    `gatt/monitorCharacteristic {clientId, service:"00001812-…", characteristic:"00002a4d-…", subscribe:true}` with `luna-send -i`.
    The `address` form fails the schema, and `instanceId` gives error 285. `getServices` lists one 2a4d with `instanceId:"064"` (handle
    0x0040, the input report), so the plain call picks the right one. Each report comes as `changed.value.bytes`, 10 bytes without the
    report ID (ID 3), for example `[8,0,0,128,128,128,128,0,0,0]` (byte 0 is the D-pad, 8 at rest; then buttons and four stick axes).
  - Working link: 1355 reports in 25 s. Dead link, after a power-cycle: 102 reports in 20 s of D-pad presses, while hidraw got 0 bytes.
  - `discoverServices` fails ("GATT service discovery failed") while the stack's own re-scan after Service Changed is still running; 4 s
    after the reconnect it failed, 50 s after it worked. A bridge must retry it.
  - The TV has `/dev/uhid` (root only), `node` 16.20.2 and `python3`. The report map (182 bytes) can be read from the dead device's
    `/sys/class/hidraw/hidrawN/device/report_descriptor`. Open question: the dead uhid device stays, so a bridge device is a second
    Stadia; whether Moonlight (it reads `/dev/hidraw0`) picks the live one is unknown.
- Bridge built (2026-11-06 22:00 on, `tools/tv/stadia-bridge.py`, started by `60-homehook`; not yet reliable):
  - It worked twice on a live dead link (22:06 and 22:09): it made its gamepad, and Moonlight used it once the node was in its jail.
    Moonlight's jail (`/var/palm/jail/com.limelight.webos/dev`) has a fixed copy of `hidraw0` only; with `hidraw1` added by `mknod`,
    Moonlight opened it within 3 s. The bridge now adds its node to each jail that has hidraw nodes. Sam confirmed input in Moonlight.
  - `gatt/discoverServices` fails until `gatt/connect` has run for the address, so the order is connect, discover, monitor.
  - It failed on fresh power-cycles (22:12, 22:18). A sign-up at 0 s comes before the controller's Service Changed. With a 4 s wait and
    renewals every 5 s, still nothing. The stack trace shows why: in the sign-up that worked (21:57) LG's service called
    `btif_gattc_reg_for_notification_impl` and then `notify_cb` per report. In the failing ones it only called `get_gatt_db`, never
    register. On disconnect the stack drops LG's client (`GATT_Deregister gatt_if=9`); LG re-registers it on `gatt/connect`, but its
    `monitorCharacteristic` then does not register for notifications. Guess: LG keeps its own "already monitoring" state per client.
  - `gatt/disconnect {clientId}` closed the client in the stack, but `gatt/connect` then returned the same "009" without opening it,
    and discovery failed from then on. A Bluetooth service restart resets LG's state, but also drops the controller.
  - LG's service is webOS OSE's `com.webos.service.bluetooth2` (`src/bluetoothgattprofileservice.cpp` on GitHub, `webosose`).
    `monitorCharacteristic` keeps a list of watched characteristics, each marked registered once the lower layer (LG's closed "SIL",
    `changeCharacteristicWatchStatus`) says yes, and skips any watch already marked. A watch is dropped, with an unregister call, only
    when its last subscriber goes. Nothing in the service clears the mark on a disconnect.
  - But no subscriber was left running (checked in `/proc`), so the service's list should have been empty at each new sign-up, and the
    stack log shows no unregister when a sign-up was cancelled either. So the stale state is probably below the service, in the SIL,
    which keeps its own record and does not register again. Not proven.
  - Next (needs the controller switched on and connected, no presses): one sign-up, then read `bt_stack.log_MTK` for
    `reg_for_notification`. Try `monitorCharacteristics` (plural, its own code path), a sign-up with `instanceId` only (error 285 so far),
    and `gatt/disconnect` followed by a fresh `gatt/connect` after a long wait. A Bluetooth service restart (drops the controller) is the
    fallback reset of the SIL.
  - The bridge is left off (`no-stadia-bridge`) until this is solved, so `stadia.sh` behaves as before it.
- Sign-up trigger found (2026-11-06 22:57). After `systemctl restart webos-bluetooth-service`, every sign-up registers
  (`btif_gattc_reg_for_notification_impl`) and each cancel unregisters (`BTA_GATTC_DeregisterForNotifications deregistered`); the
  client id changed from 009 to 004, so never hard-code it. Then a sign-up was held open while Sam power-cycled the controller: at the
  disconnect the stack logged "DeregisterForNotifications registration not found", and every sign-up after that answered "subscribed"
  without registering. So a disconnect while a sign-up is live leaves LG's lower layer believing it is still registered. Only a
  Bluetooth service restart has cleared it so far, and that drops the controller (it came back in 20 s, dead as usual) and the Magic Remote.
  - So a bridge that works needs either a restart of the Bluetooth service per dead reconnect (about 30 s without input, no Stadia + Y),
    or a way to clear that record without one. A bridge that holds its sign-up while bridging always crosses the next disconnect.
  - The HID host's own cache is the `HidReport` line (`4d2a03013c404d2a05023c44`: report IDs 3 and 5 and their handles) in the
    controller's section of `/mnt/lg/cmn_data/btsvc/fluoride-mtk/rec/bluedroid/bt_config.conf`. Without it the HID host runs its own
    discovery on connect, as on a fresh pairing. But the stack holds the config in memory and saves the line again after each discovery,
    so removing it needs a restart too. Not tried.
  - `stadia.sh` changes that go with it: a drop no longer starts a re-pair while the bridge runs (the re-pair forgot the controller, so it
    could not reconnect); with the bridge, the check alert waits `BRIDGE_SECS` (15 s); `inst()` matched only digits, so reconnects to
    instance `000A` to `000F` and the like were missed. That last one predates the bridge.
- The manual re-pair on 2026-11-06 went wrong: `adapter/unpair` returned true but left the bond, and the controller stuck "already
  connecting". LG settings remove and pair fixed it. Use `stadia.sh` for re-pairs, not hand-run luna calls.

- Rewrite (2026-11-08): the old watcher froze for good on an `hid/connect` call that never answered (`luna-send -n 1` has no time
  limit), and its "live scan entry" test matched every entry, because `manufacturerData` holds a second `scanRecord` with data. After
  that, `adapter/pair` started SMP (`WAIT_AUTH_COMPLETE`) on every try and never finished, with the controller in pairing mode. A
  `systemctl restart webos-bluetooth-service` fixed it: the next try paired and sent input within 14 s. `stadia.sh` is now a popup on each
  Moonlight open, OK to pair, every bus call under `timeout`, pair tries in a loop checked by a hidraw report, and a Bluetooth restart
  after 3 failed tries. The mid-game drop detection, the check alert, the bridge and the `bt.log` copy are gone.
- Same day, 16:17: a pairing worked (input on `hidraw2`) but Moonlight saw nothing. Each reconnect or pairing can take a new hidraw
  number while `bthidmanager` still holds the old ones, and Moonlight's jail has only a fixed `hidraw0`. `stadia.sh` now copies the
  controller's node into `/var/palm/jail/com.limelight.webos/dev` each second; Moonlight opened `hidraw2` as soon as it was there.
- Drop popup back (2026-11-13): the 2026-11-08 rewrite only asked when Moonlight came to the front, so a mid-game drop showed nothing.
  `stadia.sh` again reads the input device's instance each second and asks when it changes; a change while the popup is up restarts
  its 60 s clock. Caught the drops at 21:37, 21:59 and 22:01; re-pairs took 12 to 23 s.
- Cached report line removed (2026-11-13 21:51): with the Bluetooth service stopped, the controller's `HidReport` line was taken out of
  `bt_config.conf`, then the service started. The HID host ran its own discovery, registered for report ID 3 last, and input worked
  with no re-pair (4950 bytes in 15 s). But it saved the line again at once, so the next power-cycle was dead as usual. Before this, a
  service restart alone did not fix a dead link (2026-10-21). So "stop, remove the line, start" recovers a drop without Stadia + Y, at
  the cost of about 30 s with no controller or Magic Remote, and must run at every drop. Not built.
- Debug logs off (2026-11-13 22:05): the snoop and stack logs from 2026-11-06 were still on (3.4 MB of stack log in 9 minutes, load
  about 29), and drops came every one to two minutes that evening. `bt_stack.conf` is back to `.orig` (the debug copy is at
  `/tmp/bt_stack.conf.debug` until a reboot). Guess, not proven: the logging slowed the stack enough to drop links. Watch the drop rate.
- A drop while a direction is held leaves Moonlight holding it, since no release report arrives (BACKLOG.md, "release held buttons").
- Drops went on with the logs off. Way round it (2026-11-13 22:20): the controller is paired to the gaming PC (the room above), not the
  TV, so the game reads it on the PC and LG's Bluetooth is out of the path. Sam: "works a treat". On the TV the controller is unpaired
  and `stadia.sh` is off (`/var/lib/homehook/no-stadia`). To go back: remove that file and pair from the popup. Moonlight's stream
  menu (quit and the rest), which the controller shortcut used to open, opens from the remote: hold Back, or press red.

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
- 2026-11-10: **the gear button is code 139 for Input Hook** (`IR_KEY_MENU` = 147 in `/usr/share/X11/xkb/keycodes/lg`, minus 8). Bound to an `exec` that wrote a log line, five presses logged 139 and LG's gear menu did not open (Sam, with Lumen in front). `IR_KEY_QMENU` (801, so 793) was bound too and never fired. So an overlay app can take the key, as Lumen Guide takes LIST. Not tested: the key in Game mode, where it opens LG's Game Dashboard. The test binds were removed.
- The whole screen, the gear menu included, can be captured with `luna://com.webos.service.capture/executeOneShot` (`"method":"DISPLAY"`). Opening the gear menu over SSH is not solved: `com.webos.app.quicksettings` is not an app id.

## Power-on and the Local Control Panel (2026-10-24)
- The menu that came up after the compositor restart on 2026-10-23 is LG's Local Control Panel, `com.webos.app.tvhotkey` (native QML). It is not the service menu (`com.webos.app.factorywin`, EZ Adjust and InStart, which asks for a PIN). Launched with no `activateType`, it opens its main menu.
- Its toggles write ordinary settings. "Show LG logo when turning off the TV" is `other` / `lgLogoDisplay` (`"on"` or `"off"`); "Screen Saver" is another toggle in the same list (`Interfaces/CustomerSupportInterfaces/CustomerSupportInterfaces.qml`). Set as root with `settingsservice/setSystemSettings`. `lgLogoDisplay` was put back to `"on"`; it did not cause the Live TV problem below.
- Wake-on-LAN resumes the TV from Suspend straight into Live TV. LG Home never opens, so a wake rule that only replaced LG Home did nothing. `watch.sh` now opens Lumen over whatever is in front after any wake.
- Remote power-on with LG's "Home on power-on" (`general` / `homeAutoLaunch`) `"on"`: LG Home holds the screen for about 7 s, whatever is launched over it (about 9 s to Lumen). With it `"off"` (set 2026-10-24), nothing opens first and Lumen is in front about 5 s after the wake.
- `luna-send` over SSH prints nothing without `ssh -tt`, unless its stdin is redirected (`</dev/null`). It looks like a dead bus but is not.

## Rust (2026-11-24)
Spike 1 of `docs/plans/native-rewrite.md`: the home screen test app's scene (`native/home/`, C) written again in Rust, as an app of
its own, `native/rtest/` (`com.samrenfrew.lumen.rtest`, "Lumen Rust Test"). Same shaders, same keys, same readout, and it reads the
same `/tmp/ntest-tune` and `/tmp/ntest-bench`, so the two can be compared move for move. Its log is `/tmp/lumen-rtest-<uid>/log`.

On the NUC (done):
- **It builds in a container.** `sh tools/build_rust_test.sh` makes the package, `npm run deploy:rtest` installs it. Rust 1.100.0
  (pinned in `native/rtest/rust-toolchain.toml`), target `armv7-unknown-linux-gnueabi`, linked with the webOS SDK's gcc against
  the SDK's libraries. The image (`native/rtest/Dockerfile`) is Debian bookworm with rustup, built once.
- **It draws the same picture.** `sh native/rtest/host-shot.sh` and `sh native/home/host-shot.sh` drew five scenes each with
  software graphics (bar, bar with another tile and Blur only, both grid rows, No blur, and mid slide). All five pairs match in
  every pixel.
- **No Rust libraries.** SDL, SDL_image, SDL_ttf and OpenGL ES are called through declarations written by hand in `src/ffi.rs`
  (82 functions). The one build-time library is `cc`, which compiles the C that is linked in.
- **C links in.** `src/webos_shell.c` (the Back key claim, which needs Wayland header macros) is the C test app's code unchanged,
  compiled by `build.rs` and called from Rust. That is the route for `picker.c`, `weather.c` and `compose.c`.
- **LG's SDL is not stock SDL.** Its key event has an extra field (`inputSource`) before the key, so a key event declared from
  SDL's public headers reads the wrong bytes on the TV. `src/abi_check.c` is compiled with the real headers on every build and
  fails the build if a size, offset or constant in `ffi.rs` differs. It caught this one. Any Rust SDL library from crates.io
  would need the same care, which is a reason to keep our own declarations.
- **The SDK's C library is older than Rust needs.** The SDK links against glibc 2.12 so apps run on every webOS; Rust's standard
  library needs 2.17. Only one function was missing, `getauxval`, and `src/getauxval_missing_from_sdk_libc.c` supplies it.
- **Float maths needs a flag that Rust is retiring.** The TV's calling convention passes floats in ordinary registers (softfp).
  Rust's matching target also does all float maths in software. `build.sh` turns the chip's float unit on with
  `-C target-feature=+vfp3,+neon,-soft-float`, which works on 1.100.0 but prints "will become a hard error in a future
  release" (rust-lang/rust issue 162235). `LUMEN_RUSTFLAGS=` builds without it. If the flag goes, the choices are a pinned
  Rust, a nightly Rust with its own target file, or software float maths (the heavy maths is on the graphics chip anyway).
- **Size.** The program is 413 kB against the C one's 40 kB. Most of it is Rust's standard library.
- **Tests.** `cargo test` runs four on the NUC (the easing curves, an animation turned round mid way, the tuning file, log dates).

On the TV (2026-11-24, installed with `npm run deploy:rtest`, both apps with the same `/tmp/ntest-tune`):
- **It starts and draws.** The shaders compile and the scene is on screen (`NL_VSC` visible, `rtest` in front).
- **Frame times are the same.** The timing test (`/tmp/ntest-bench`), draw time per frame, Rust against the C app's run of 11:13
  the same day: all at rest 10.1 ms against 10.2, mid slide 11.8 against 12.3, down in the grid 9.7 against 9.5, wallpaper only
  6.2 against 6.2, at rest held to the screen's beat 12.2 against 12.2, and 20.0 ms a frame in both.
- **Memory is the same.** Its own memory 13.6 to 13.8 MB against 13.7 (peak 33.2 against 33.1). Graphics memory 63.1 to 66.1 MB
  against 63.1 to 63.4 (`MemoryUsageAllocGPUMemUMA` in the driver's `process_stats`), three launches each.
- **Start time is the same to within what three launches can show.** The window opened 136 to 202 ms after start against 130 to
  164. The first frame was ready at 1253, 1263 and 1442 ms against 1145, 1150 and 1330 (both now include about 0.4 s spent
  measuring the screen's refresh). Rust was about 0.1 s later each time; not looked into.
- A C run started 4 s after the Rust app closed was slow and odd (blur 1145 ms, a frame every 40.8 ms) and is left out. With 6 s
  between runs both behaved.
- **Keys work, and every frame shows.** Sam pressed Left, Right, Down, Up and OK (12:29): 30 moves logged, none with a late
  frame, slowest frame 25 ms, draw 10.5 to 13.2 ms a frame. OK changed the glass. So the key event fix is right on the TV.
- **Back closes it.** The TV sends Back as scancode 482 with key code 0, which neither test app took as Back, so the C app never
  closed on Back either. Both now close on scancode 482. Seen by Sam on the Rust app (12:33); the C app is fixed in source but
  not reinstalled.

In CI (not run yet): this fork's workflows only run on `main`, so the Rust build has not run on a GitHub runner.

## TV calls from a native app (2026-11-24)
Spike 2 of `native-rewrite.md`. Answer: **the first route works. The app calls the TV itself, with the TV's own `libluna-service2`,
and needs no role file and no helper.** Tried from inside Lumen Rust Test with a probe (`native/rtest/src/probe.rs`): it reads
`/tmp/rtest-probe` whenever that file changes and runs each line (`open <name>`, `call <uri> <json>`, `sub <uri> <json>`, `ls`,
`read`, `write`, `id`), logging each answer as a `probe` line. So calls can be tried live, with no rebuild.

- A native app does get a role file at install, `/var/palm/ls2-dev/roles/prv/<id>.json`, which allows its own id as a bus name
  and the empty name, with every outbound call. The spec was wrong that it has none.
- **LG's SDL takes the app's own id on the bus** as soon as the window opens (it listens for the app manager's events). So
  `LSRegister("<id>")` fails with "already exists". **`LSRegister(NULL)`, a nameless handle, works**, and so do the
  `com.webos.vt.*` and `com.webos.chromium.media.*` names the role file allows. Use the nameless one.
- The glue is C (`native/rtest/src/luna.c`, 70 lines: open, call with a time limit, subscribe, pump), built against the SDK's
  headers and called from Rust. It runs the bus on its own GLib context, so it can live on a thread away from drawing.
- Proved, each from the jail, with the time the call took: the app list (`listLaunchPoints` 9 ms for 20 kB, `listApps` 23 ms
  for 174 kB), `getAppInfo` 1 ms, `running` 4 ms, the foreground app 1 ms, reading picture settings 14 ms and **writing one**
  (`setSystemSettings` backlight, 15 ms), volume 5 ms, inputs 4 ms, network 15 ms, the last channel 35 ms, the channel list
  12 ms, mute status, the sleep timer, the config service, **launching an app** (`applicationManager/launch` opened the C test
  app), and **a root command**: `org.webosbrew.hbchannel.service/exec` ran `id -u` as `0` in 43 to 59 ms.
- Subscriptions work: `getForegroundAppInfo` and `audio/getVolume` with `"subscribe":true` both sent their first answer.
- Its permissions are the web app's ("public" plus the Homebrew Channel's exec), so anything the web Lumen calls, this can.
- Not needed: a role file written by the TV scripts, or a root helper on a socket. A helper would still save the 45 ms a root
  command costs, if a screen ever needs many at once.

## A place to keep things (2026-11-24)
Spike 3. The jail's view, from the same probe (the app runs as its own user, uid 5725 here, group 5000):

- **Lasting and writable: `/media/developer`** (mode 777, the developer partition) and the app's own folder under
  `/media/developer/apps/usr/palm/applications/<id>/`. An install replaces the app's own folder, so settings and caches go in
  a folder of Lumen's own under `/media/developer`. (verify) that the app's uid is the same after a reinstall; if it is not,
  the folder must be made writable to all, or made by a root command.
- **`/tmp` is the TV's real `/tmp`**, shared with root: the log and the tuning files already use it. Lost at power off.
- **Not there at all: `/var/lib/homehook`, `/home`, `/var/preferences`, `/usr/palm/applications`, `/media/cryptofs`.**
  `/media/internal` is read-only. So the TV scripts' files (`lumen.log`, the settings they share) are reached with a root
  command, or the scripts move what the app needs into `/media/developer`.
- **Other apps' icons.** Sideloaded apps' folders can be read directly (`com.samrenfrew.lumen/appinfo.json`, its icons).
  LG's and store apps' folders cannot be seen, so their icons come through a root command once, as the web app does now,
  and are kept in the cache folder.
- The app can also write into other sideloaded apps' folders, the web Lumen's among them. Nothing should.
- Launch parameters arrive as the program's second argument, as JSON:
  `{"event":"launch","appId":"…","parameters":{"view":"ambient"},"interfaceVersion":2,…}`.

## 50 or 60 a second (2026-11-24)
Spike 8. Answer: **the panel runs at the rate of the last video source it showed, and stays there.** `vbe_outputFrameRate` in
`/proc/lgtv-driver/vbe/vbe-status` is the panel's real rate.

- After Live TV (UK broadcast, 50 a second) it is 50, and it stayed 50 in Lumen, in YouTube's browse screen and in the native
  test app (a frame every 18.5 to 20 ms).
- After HDMI 1 (the Chromecast, 59.9 a second) it is 60, and it stayed 60 when the native test app opened over it (a frame every
  15.6 ms measured over 20 swaps) and in Lumen afterwards.
- So a native app is given the panel's true beat by `SDL_GL_SwapWindow` and must fit both. The 12 ms drawing budget does.
- **The web page always draws 60 a second**, whatever the panel does: `requestAnimationFrame` came every 16.67 ms (median of
  120, min 16.6, max 16.8) while the panel was at 50. So with the panel at 50, which is its usual state on this TV, the web
  Lumen makes six frames for every five the panel shows, and one in six is dropped. `frame-rate.md` assumed 60 throughout.
  A native Lumen draws one frame per panel frame at either rate.
- Animations must be timed by the clock, not by counting frames, as the test apps already do.

## The test app's memory (2026-11-24)
Spike 7. Answer: **of the 62 MB of graphics memory, about 21 MB is pictures. 16 MB is a depth buffer nobody asked for, and
12 MB is the cost of giving each tile its own drawing target. Without those two it is 33 MB**, inside the 40 MB budget.

Measured with `touch /tmp/rtest-steps`: Lumen Rust Test then stops for 2.5 s after each start-up step (a `step` line in its
log), while `MemoryUsageAllocGPUMemUMA` for its pid is read from the driver's `process_stats` three times a second.

| After | Graphics memory | Added | The pixels alone |
|---|---|---|---|
| The window and context | 1.6 MB | | |
| Eleven shader programs | 1.6 MB | 0 | |
| The wallpaper at screen size, and its two blurred copies | 19.0 MB | 17.4 | 10.5 MB |
| 18 icons, 130x130 | 20.5 MB | 1.5 | 1.2 MB |
| 18 resting tiles, 296x200, each its own target | 38.3 MB | 17.8 | 4.3 MB |
| The bar's two pictures, 1890x364 | 44.7 MB | 6.4 | 5.5 MB |
| 24 empty frames swapped | 45.1 MB | 0.4 | |
| The scene drawn once to the screen | 61.5 MB | 16.4 | |

The window's own three buffers are counted apart (`MemoryUsageDmaBufImport`): 6.3 MB each, 19 MB once three are in use.
The program's own memory is 13 MB.

- **The 16.4 MB on the first real frame is a depth and stencil buffer** (`ZSBufferRequestsByApp` goes to 1). SDL asks for a
  16-bit depth buffer unless told not to. With `SDL_GL_DEPTH_SIZE` and `SDL_GL_STENCIL_SIZE` set to 0 (`no_depth=1` in
  `/tmp/ntest-tune`) it never appears: 44.9 MB. Lumen draws flat pictures back to front and needs neither.
- **A drawing target costs about 0.7 MB on top of its pixels.** 18 tile targets cost 13 MB more than their pixels. With one
  target shared by all 18, each tile's picture attached to it in turn (`one_fbo=1`), the tiles cost 5.8 MB: 49.8 MB in all.
  (verify) by eye that the tiles look the same drawn this way; the code path is the same but nobody looked.
- Both together: **33.1 MB**.
- The wallpaper step is 7 MB over its pixels, not explained. It makes five targets and frees two.
- Rules for the picture cache: ask for no depth or stencil buffer; pictures are plain textures; one drawing target (or a few)
  is reused to draw into them; never one target per picture.

## Text with FreeType and HarfBuzz (2026-11-24)
Spike 6. Answer: **yes. The TV's own FreeType (2.11) and HarfBuzz (4.0.1) shape and draw all nine languages, with kerning,
letter-spacing and joined-up Hindi, and the result sits within 2% of the web page's line widths.**

`text=1` in `/tmp/ntest-tune` (or `LUMEN_TEXT=1 sh native/rtest/host-shot.sh out.png` on the NUC) makes Lumen Rust Test show a
sheet of eleven lines: a sentence in each language at 34 px, a line of awkward pairs and digits at 64 px, and a line with
-0.02em letter-spacing. `native/rtest/src/text.c` (120 lines of C, called from Rust) splits a line into runs by script, shapes
each with HarfBuzz, and draws the glyphs with FreeType into one picture per line. The same lines were put into the live web
page over DevTools, in the page's own font stack at weight 500, measured and captured.

- **Line widths, native then web, in px:** English 1055 and 1076, German 1158 and 1184, Spanish 1108 and 1134, French 1117 and
  1140, Italian 1024 and 1049, Portuguese 1074 and 1096, Dutch 1094 and 1114, Turkish 1063 and 1084, Hindi 1003 and 977, the
  64 px line 819 and 823, the letter-spaced line 909 and 922.
- At 34 px the web's lines are 2% wider; at 64 px they match. The web page loads SF from `sf.woff2` and the native app from
  `sf-500.ttf`, so the likely cause is the font file, not the shaping. Not chased. If the port must match the web to the
  pixel, it loads the same file.
- **Hindi is right in both**: the conjuncts and the marks above the line join as they should. The native sheet uses
  `LG_Smart_UI_Devanagari.ttf` for the Hindi run, punctuation included, which is why its commas are heavier and its line wider.
- Kerning shows in "AV", "To", "Wa", "Ty" and "P."; digits are proportional (the 1s in "11°" sit close), as `CLAUDE.md` asks.
- **Time on the TV: 88 ms for the eleven lines, fonts loaded included** (17 ms on the NUC). So a line is drawn once into a
  cached picture, as the spec says, and a screen of new text is a frame or two of work on a worker thread.
- **The SDK has FreeType's headers but no HarfBuzz at all.** The build declares the nine HarfBuzz functions it uses by hand
  and links against a stand-in library with the TV's library name (`src/harfbuzz_missing_from_sdk.c`); the TV's real one
  loads at run time. The NUC build uses Debian's HarfBuzz.
- Not done: right-to-left text (no language needs it), line breaking, an ellipsis for text that is too long, and font
  fallback glyph by glyph (the splitter knows Latin and Devanagari only).

## Video under a native window (2026-11-24)
Spike 5. **A video wallpaper works. Glass that shows the video through it, and the live TV preview, are not proven.**

- **A video plays under the native app.** In see-through mode (`see_through=1` in `/tmp/ntest-tune`: the frame is cleared to
  alpha 0 and the wallpaper is not drawn) Lumen Rust Test showed the bar and tiles over a playing clip. Seen by Sam: the
  mountain video moves behind the bar. Steps, all from inside the jail:
  1. Export the window for video with the `wl_webos_foreign` Wayland protocol (`lumen_export_video_window` in
     `native/rtest/src/webos_shell.c`): the TV answers with an id such as `_Window_Id_66`.
  2. On a bus handle named `com.webos.media.client.<anything>` (the role file allows it), call `com.webos.media/load` with
     `{"uri":…,"type":"media","payload":{"option":{"appId":"<id>","windowId":"_Window_Id_66"}}}`. It answers with a `mediaId`.
  3. `com.webos.media/play` with that id. `subscribe` sends the time every 200 ms and `endOfStream`; `seek` to 0 replays it.
  The media service connects the decoder to the screen itself (`videooutput/getStatus`: MAIN connected, 1280x720 in, the
  whole panel out). Without the window id the clip decodes but never reaches the screen.
- **The clip must be unloaded.** The pipeline outlived the app: after the app closed it was still listed, holding a decoder,
  until `com.webos.media/unload`. The app must unload on every way out, and a TV script should sweep for strays.
- **The glass bar does not show the video**: it still read the still wallpaper's blurred copy, as Sam saw. To blur the
  video the app would have to read the video's picture. PicCap does read it on this TV (it holds a `VT_CAPTURE` pipeline, as
  root, for Hyperion), so it can be done; it was not tried from the jail, because a second capture client may take the
  capture away from Hyperion. Until that is tried, glass over video is a tint and rims, as the web app has it.
- **Screenshots cannot check video.** `capture/executeOneShot` shows black where the video is, in every mode, and the app is
  not allowed to call the capture service at all ("Not permitted"): the one TV service found so far that refuses it.
- **The live TV preview is not proven.** The web app plays `tv://<channel>` in a `<video>`; what the web engine calls for that
  was not captured. Next step: run `ls-monitor` on the TV while the web Lumen starts a preview, and repeat those calls from
  the probe with the exported window id.

## Staying alive, so far (2026-11-24)
Spike 4, the part that needs no reboot:

- **Out of the box a native app is closed the moment another app opens**, as the web Lumen was before it went on the TV's
  lists (`docs/keep-alive.md`). Lumen Rust Test launched the C test app with `applicationManager/launch`, and was gone with
  no "close" line in its log. Its next launch was a fresh start.
- **Launch parameters reach a fresh start** as the second program argument: `…"parameters":{"view":"ambient"}…`.
- LG's SDL registers the app with the app manager itself, so how a relaunch of a running app arrives (an SDL event, or
  nothing) is not known yet.
- When the two test apps were opened in turn a few seconds apart, the window manager relaunched the one just closed
  (`NL_APP_LAUNCH_BEGIN … "caller_id":"com.webos.surfacemanager","mode":"autoReload"`). Not understood.
- **(verify)** with `com.samrenfrew.lumen.rtest` added to `com.webos.memorymanager.KeepAlive` and `KeepOnLaunchEx` and the TV
  rebooted (the memory manager reads the lists only at start): it survives another app opening, it is told when it goes to
  the back and comes forward, a launch while it runs delivers the new parameters, and its memory in the back.
- The Home button and power-on are settings that name an app id (`keybinds.json`, `home-setup.sh`), the same for a native app.
