# Home button and power-on

Two TV settings that make Lumen the TV's home, set by `tools/tv/home-setup.sh` over SSH. Both are written defensively and are
**(verify)** in `docs/plans/PLAN.md` until seen working on this TV. Where the ideas come from: `docs/research/prior-art.md` (Hearth 2.0,
upstream `docs/native-home.md`, Glasshouse).

## Native Home

The TV's Home routes open the app named in the system setting `general.defaultApps.home`. `native-home on` sets it to Lumen with
`applicationManager/setDefaultApp`, after checking that both Lumen and LG Home have `"home"` in their `appCategory` (Lumen's
`appinfo.json` has it). The first time, it saves the old answer to `/var/lib/homehook/defaultApps.backup.json`.

What it should fix: Exit, and Back out of an app, open LG Home today, because Input Hook only rebinds the Home key. Reports elsewhere say
this setting covers those too (a G5 on webOS 11.2, a 65UT8000 on webOS 25). Input Hook's Home mapping can stay as it is.

If it does not cover Exit here, the next step is the webOS 10 Home takeover in `docs/research/prior-art.md` (a mount over LG Home's app).

## Power-on app

When the TV is switched on, `bootd` picks the first app from `firstAppByPowerOnReason` in `/etc/palm/bootd.json`. For the power
button (`remoteKey`), the TV's own button (`localKey`) and a cold start (`micomPower`) it has no app, so the TV opens its last input.
`bootd` reads `/var/luna/preferences/bootd.json` instead when that exists. `power-on on` writes that copy with those three reasons set
to Lumen. Every other reason (casting, Wake-on-LAN, HDMI-CEC, timers) stays LG's.

- It takes effect after a full restart (`reboot`), since `bootd` reads the file only when it starts.
- It refuses unless `bootd` names the copy's path, LG's file has the table, and `node` is there to write the JSON.
- The copy's md5 is kept in `/var/lib/homehook/bootd-override.md5`. A copy with another sum is someone else's and is never changed.
- Unknown on this TV: whether waking from standby goes through `bootd`. The `watch.sh` wake launch stays for that case either way.
- If this works, the TV input added on 2026-10-24 (`MVPD_IP-com.samrenfrew.lumen`) can go: `eim/deleteDevice` with the same appId.

## The boot guard

`60-homehook` runs `home-setup.sh guard` at each boot. If no Lumen build is installed (checked twice, 15 s apart), it removes the
power-on copy and points Home back at LG Home. If a firmware update changed LG's `bootd.json`, it removes the copy so an old copy never
hides the new file. Each step is logged to `/tmp/homehook.log`.

## Install and try

```sh
scp tools/tv/home-setup.sh tools/tv/60-homehook <tv>:/tmp/
ssh <tv> 'cp /tmp/home-setup.sh /var/lib/homehook/ && cp /tmp/60-homehook /var/lib/webosbrew/init.d/ && chmod +x /var/lib/webosbrew/init.d/60-homehook'
ssh <tv> 'sh /var/lib/homehook/home-setup.sh check'        # read only
ssh -t <tv> 'sh /var/lib/homehook/home-setup.sh native-home on'
ssh <tv> 'sh /var/lib/homehook/home-setup.sh power-on on' # then reboot
```

## Undo

- `sh /var/lib/homehook/home-setup.sh native-home off` and `power-on off`, then reboot.
- By hand: `rm -f /var/luna/preferences/bootd.json`, reboot, and
  `luna-send -n 1 luna://com.webos.service.applicationmanager/setDefaultApp '{"category":"home","appId":"com.webos.app.home"}'`.
- Set Home back to LG Home before uninstalling Lumen. The guard does it at the next boot if you forget.
