# Home button and power-on

Two TV settings that make Lumen the TV's home, set from Settings > Home and Power, which runs `tools/tv/home-setup.sh` as root. Both are written defensively and are
**(verify)** in `docs/plans/PLAN.md` until seen working on this TV. Where the ideas come from: `docs/research/prior-art.md` (Hearth 2.0,
upstream `docs/native-home.md`, Glasshouse).

## Native Home

The first try set `general.defaultApps.home` to Lumen with `setDefaultApp`. On this TV the setting took, but Exit and Back out of an app still opened LG Home, before and after a restart (tested 2026-11-01). So `native-home on` now uses the Home takeover from Glasshouse (`server/lib/home-takeover.js`, MIT) and lg-luma (`tv-helper/src/home.rs`, MIT):

1. Write a stand-in app to `/var/lib/homehook/home-app`. It has LG Home's id (`com.webos.app.home`), `appCategory: ["home"]`, and a blank page that opens Lumen when it starts and on each relaunch. Home pressed while Lumen is already in front (its screensaver included) used to leave a black screen: the stand-in asked for Lumen before its own window showed, so the window then covered Lumen. Seen twice on 2026-11-07, after Sam removed the Input Hook bind that sent Home straight to Lumen. Since then the page asks again every 500 ms, up to 6 times, while it is still on top. A launch of Home over Lumen on the TV showed the stand-in for about 0.4 s, then Lumen.
2. Bind-mount it over `/usr/palm/applications/com.webos.app.home`.
3. Make the app manager read it again: `blockSystemApp`, then `releaseBlockSystemApp` (with `forceBlockSystemApp: false` where the TV's schema asks for it).
4. Stop LG's Home process (`flutter-client -i com.webos.app.home`). The next Home, Exit or Back out of an app starts the stand-in, and the stand-in opens Lumen.

The mount does not outlive a restart. `/var/lib/homehook/home-mount` holds the Lumen id while it is on, and the boot guard mounts again about 20 s into a boot.

Seen on this TV (2026-11-01): Exit from YouTube opened Lumen about 1 s after the press, with nothing in front in between. Back out of YouTube opened Lumen. iPlayer did not close on Back at all. After a restart the guard mounted again by itself.

Known costs:
- The hand-over gap of about 1 s. lg-luma has none, because it puts its own app under LG Home's id. The same for Lumen would be Lumen's files under that id, which makes a second copy of Lumen with its own storage.
- While it is on, Lumen's LG Home button opens Lumen, since LG Home's id is the stand-in's.

## Power-on app

When the TV is switched on, `bootd` picks the first app from `firstAppByPowerOnReason` in `/etc/palm/bootd.json`. For the power
button (`remoteKey`), the TV's own button (`localKey`) and a cold start (`micomPower`) it has no app, so the TV opens its last input.
`bootd` reads `/var/luna/preferences/bootd.json` instead when that exists. `power-on on` writes that copy with those three reasons set
to Lumen, and so do Wake-on-LAN (`wakeOnLan`, `wakeOnWiFi`), so a Home Assistant switch-on that is a full boot opens Lumen too. Every
other reason (casting, HDMI-CEC, timers) stays LG's.

- It takes effect after a full restart (`reboot`), since `bootd` reads the file only when it starts.
- It refuses unless `bootd` names the copy's path, LG's file has the table, and `node` is there to write the JSON.
- The copy's md5 is kept in `/var/lib/homehook/bootd-override.md5`. A copy with another sum is someone else's and is never changed.
- A wake from standby does not go through `bootd`: see "Waking from standby" below.
- If this works, the TV input added on 2026-10-24 (`MVPD_IP-com.samrenfrew.lumen`) can go: `eim/deleteDevice` with the same appId.

## Waking from standby

With Quick Start+ on, switching on from standby is a resume, not a boot. `bootd` does nothing on a resume, so neither the power-on table
nor LG's Home Auto Launch applies. The TV brings back its last input. Found 2026-11-01, on Wake-on-LAN wakes from Live TV:

- `tvpower/power/getPowerState` (subscribed) reports `Prepare Resume`, then `LastInput Ready`, then `Screen On`, about 2 s apart in all.
  Nothing is in front at `Prepare Resume`. A subscription made before standby survives it.
- Switching off closes every app but Live TV, Lumen included. A launch at `Prepare Resume` is then a cold start: Live TV showed for
  about 6 to 8 s before Lumen.
- The app manager refuses launches once suspend starts ("cannot launch in suspend mode"). A power key sent through `/dev/input/event1`
  suspends at once; the real remote leaves about 40 s of Active Standby first (`docs/keep-alive.md`), where Lumen can be started hidden.
- The stand-in Home app was still running after switch-offs. Untested: whether opening it at `Prepare Resume` stays in front of Live TV.
- One switch-off during these tests ended in a reboot with reason `rebootByFault`.
- `tools/tv/resume.sh` (started by `60-homehook`) opens Home at `Prepare Resume`. With the takeover mounted, Home is the stand-in, so Lumen
  follows. First run, Wake-on-LAN from YouTube after an instant suspend (2026-11-01): nothing in front until 3 s after Screen On, the
  stand-in at 3 s, Lumen at 7 s. Live TV never came to the front. Not yet seen: what the screen shows in those first 7 s, and the real
  remote, whose standby window lets `watch.sh` start Lumen hidden, so the hand-over should be faster.
- 2026-11-02 (verify): `resume.sh` now opens Lumen itself, not the stand-in. Through the stand-in, Sam saw a "Home" loading screen
  for a few seconds before Lumen.
- Seen 2026-11-02, remote power key with the screensaver up: "Request Power Off" comes about 3.5 s before "Request Active Standby", and
  the TV is in Suspend about 6 s after the key, too soon for `watch.sh`'s standby preload at 10 s. Lumen is still open at "Prepare
  Suspend" but is closed by the next switch-on, so each switch-on is a cold start under LG's "Home loading..." cover (Lumen's title is
  "Home"). For about 2.5 s after Screen On the TV's resume cover (`com.webos.app.coverui`) shows a picture of the last screen, which
  was the screensaver. `resume.sh` now relaunches Lumen at "Request Power Off" when it is in front, so that picture is the home
  screen (verify; one run worked). Starting Lumen hidden at "Prepare Suspend" did not keep it warm.
- 2026-11-03 (verify): `resume.sh` and `watch.sh` open Lumen at a switch-on only while Settings > Home and Power > When the TV turns
  on is Lumen, read from the `bootd-override.md5` file that `home-setup.sh power-on on` writes. Before this, they opened Lumen at
  every switch-on whatever the setting said (issue 6, on an LG C5).
- Glasshouse, Hearth, lg-luma and cleanyourtv-WebOS have no answer: Glasshouse opens its Home 4.5 s after the wake, over Live TV, and
  skips Wake-on-LAN wakes.

## The boot guard

`60-homehook` runs `home-setup.sh guard` at each boot. The first run after a boot also notes what `bootd` read
(`/tmp/lumen-poweron.loaded`), which is how Settings knows a restart is still needed. If no Lumen build is installed (checked twice, 15 s apart), it removes the
power-on copy and points Home back at LG Home. If a firmware update changed LG's `bootd.json`, it removes the copy so an old copy never
hides the new file. Each step is logged to `/tmp/homehook.log`.

## Install and try

From Lumen: Settings > System > TV scripts installs or updates the scripts from the app itself (they ship in the app's `tv/` folder; the
build copies them from `tools/tv/`). Then Settings > Home and Power:

- **Home button opens**: Lumen or LG Home. OK switches it. Shows "Not available" with the reason when the TV has no `blockSystemApp`.
- **When the TV turns on**: Lumen or Last input, with "(after a restart)" until the TV has restarted, and a **Restart the TV now** row
  meanwhile. Shows "Not available" with the reason when `bootd` here does not read the copy.

Each row reads the TV's real state when Settings opens (`lib/tvsetup.ts`, one root call), and again after each change.

Over SSH, the same script by hand:

```sh
ssh -t <tv> 'sh /var/lib/homehook/home-setup.sh check'        # read only, the fullest report
ssh <tv> 'sh /var/lib/homehook/home-setup.sh native-home on'
ssh <tv> 'sh /var/lib/homehook/home-setup.sh power-on on' # then reboot
```

Over SSH, `luna-send` prints nothing unless the session has a terminal, so use `ssh -t` for `check`. Lumen's own root calls read fine.

## Undo

- `sh /var/lib/homehook/home-setup.sh native-home off` and `power-on off`, then reboot for power-on.
- By hand: `rm -f /var/luna/preferences/bootd.json /var/lib/homehook/home-mount` and reboot.
- Set Home back to LG Home before uninstalling Lumen. The guard does it at the next boot if you forget.
