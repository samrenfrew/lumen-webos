# Lumen roadmap

Target TV: LG 43UT91006LA, webOS 25 (Chromium 120), rooted, Homebrew Channel installed, Home button remapped with LG Input Hook.
Apps in daily use: PlxNative (Plex), youtube-webos, BBC iPlayer, Channel 4, Channel 5.

This is a personal fork. Nothing goes back to partridgeworks/webos-launcher. Features are tried against this TV first and kept only if they work here.

Code that needs a TV answer is written defensively (it does nothing harmful if the answer is no) and goes in **Verify on the TV** until
Sam has seen it work. The agent installs each change on the TV over SSH (`npm run deploy:home`), so "verify" means: Sam tries it on the TV
and reports back. Once Sam has seen it work, delete the item; git history keeps it.

The list was cleared on 2026-10-26. Older items that were never ticked are in git history (`git show 19db1f3:PLAN.md`); treat them as
seen unless something looks wrong.

---

## Where things stand (2026-11-02)

- Lumen 2.1.0 is released. It has the design audit (`docs/research/design-audit.md`), Home and Power, and Privacy. Branch
  `ccr-dba88192-ygl4fc` is three commits past it (Moonlight's tile, the Home preload guard, the screensaver's first photo).
- Built on 2026-11-02, not committed, not yet on the TV (it was off): Settings > Brightness, the screensaver going back to the app it came
  up over, the wallpaper fade without the stall, and app names without the store tagline. Each is in Verify below.
- Other forks were reviewed on 2026-11-02 (`docs/research/prior-art.md`, "Forks of Lumen"). Keep an eye on `codefarm-eu/lumen-webos`.
- Pick up next: `npm run deploy:home` when the TV is on, then Settings > System > TV scripts > Update (`watch.sh` changed), then the new
  Verify items. After that, the two Control Center items in Open on the TV.

---

## Verify on the TV (built, not yet seen working)

Add new items here, with the branch and what to check. Delete an item once Sam has seen it work.

- [ ] Preload guard in `watch.sh` (`docs/keep-alive.md`, item 4). After a reboot, `/var/preferences/webos-preload-manager-conf.json`
      still has `false` for Netflix, AirPlay and LG Channels, and `/tmp/homehook.log` shows a "preload: set back to off" line whenever
      the TV had flipped one back on. YouTube is left preloading on purpose.

- [ ] TV scripts, Home and Power, and Privacy in Settings (branch `ccr-dba88192-ygl4fc`; `docs/home-and-power-on.md`, `docs/privacy.md`).
      Live first over DevTools where it can be: the rows' text and layout. The root calls need a build with the scripts in it.
      1. Settings > System > TV scripts says Install or Update; OK installs them. Check `/tmp/homehook.log` and that `watch.sh` restarted.
         Also `sh /var/lib/homehook/home-setup.sh check` over SSH once, for the fullest report.
      Tested remotely 2026-11-01: step 1 works (Install, then Up to date, watchers restarted). Power-on works: after a restart from
      Live TV, bootd opened Lumen first and Live TV never showed. Home button opens: setDefaultApp did nothing for Exit, so it is now
      the Home takeover (`docs/home-and-power-on.md`); Exit and Back out of YouTube open Lumen after about 1 s, and the guard mounts it
      again after a restart. Still for Sam: how the 1 s hand-over looks, the real remote's Home and Exit, and Wake-on-LAN and remote
      power-on from standby.
      4. Privacy, one row at a time, with an app or two in daily use between each (Plex, iPlayer, YouTube): agreements, Limit ad
         tracking, Live Plus, Block LG ad servers, Stop LG tracking services. Note the values after a restart. Block TV software updates
         last.
         Tested remotely 2026-11-01: the agreements were already withdrawn and Live Plus already off. Limit ad tracking, Block LG ad
         servers (178 lines) and Stop LG tracking services are on; iPlayer and YouTube still open. The services row reports 2 still
         running (`service-logger` and `nudge` sat in "deactivating"). Plex not tried.
      5. Power-on works, so remove the TV input added on 2026-10-24 (`eim/deleteDevice`, appId `com.samrenfrew.lumen`) and the item
         "Cold start after a reboot" below.
- [ ] Screensaver without Immich (`design/system-cleanup`): Settings > Screensaver offers Enabled or Disabled, with no Off; Disabled hides the other rows. Enabled with
      Show set to Wallpaper pictures, LG's screensaver timer opens this app's, which crossfades the pictures ticked in Choose wallpaper.
      Immich photos still work. Check that `/var/lib/homehook/no-screensaver` is gone after a launch.
- [ ] Switch-on leaves Live TV alone (`design/system-cleanup`, `tools/tv/watch.sh`, on the TV 2026-10-31): the watcher opens Lumen at switch-on only
      if Lumen was the last app in front, or the TV came up on LG Home. Check: watch Live TV, switch off, switch on, and Live TV stays with no
      Lumen. Then switch off from Lumen and on again, and Lumen opens. `/var/lib/homehook/last-front` should name the app you were in.
- [ ] Home soon after switch-on is warm (`tools/tv/watch.sh`, on the TV 2026-10-31): switching off closes Lumen, so the watcher starts it
      hidden 10 s into standby. Seen: it loads in standby and survives the TV's deeper sleep (Lumen up 1 s after LG Home, 26 s after a
      switch-on). Not yet seen: Home pressed within 5 s of switching on into Live TV, after at least a minute in standby.
- [ ] Settings > Brightness (2026-11-02; `lib/brightness.ts`, `tools/tv/watch.sh`). Needs the TV scripts updated. Turn off Home
      Assistant's brightness and Eye Comfort automations first. Check: `/var/lib/homehook/backlight-schedule` holds three lines; within a
      minute `/tmp/homehook.log` shows a "brightness:" line and the backlight matches. Set a start time a minute or two ahead and watch it
      change. A change by hand stays until the next part of the day. With Lumen's screensaver up across a start time, the screensaver stays
      dim, and a key press brings back the new level, not the old one. Eye Comfort follows the schedule unless set to As the TV is set.
- [ ] Screensaver goes back to the app (2026-11-02; `watch.sh` passes `from`, `leaveAmbient` in `app.tsx`). Pause a film in Plex and in
      YouTube, let the screensaver start, press OK: the paused film comes back where it was, not Lumen's home screen. Home over the
      screensaver still goes to Lumen's home screen. If an app restarts instead of resuming, note which (codefarm-eu moved to an overlay
      for this, which hints that some apps lose their place).
- [ ] Wallpaper fade without the stall (2026-11-02, from quangtdev's fork): night dimming is now a black sheet (`#wpimage::before`), not
      opacity, and a new picture is drawn 150 ms before its fade starts. Check at night: a wallpaper change fades smoothly, and the
      dimming looks the same as before.
- [ ] App names drop a store tagline (2026-11-02, from juanrusch's fork): "Spotify - Music and Podcasts" reads "Spotify", and a long name
      in the dock ends in "…" inside the focused glass.

---

## Open on the TV

- [ ] Cold start after a reboot: the TV showed Live TV for 10 to 15 s before Lumen (2026-10-24). Lumen is now a TV input (`eim/addDevice`,
      id `MVPD_IP-com.samrenfrew.lumen`, done over SSH), picked once as the last input; the next restart drew at 8.3 s (one sample). Next:
      more samples, check that Live TV no longer shows first, and why `/tmp/homehook.log` was missing after that boot. Removing the input
      is `eim/deleteDevice` with the same appId (untested).
      Idea from Hearth 2.0 (`docs/research/prior-art.md`, "Opening an app when the TV is switched on"): a copy of
      `/etc/palm/bootd.json` at `/var/luna/preferences/bootd.json` with `remoteKey`, `localKey` and `micomPower` set to Lumen makes
      the TV open Lumen first, behind LG's start-up cover. Check first, read-only: does `/usr/sbin/bootd` name that path, and what is in
      this TV's `firstAppByPowerOnReason` table.
- [ ] Keep apps alive (`docs/keep-alive.md`): Lumen, iPlayer, YouTube, Plex, Channel 4 and Channel 5 stay open in the background. Lumen's
      home screen lags for a moment on return. Next and untried: "Not tried yet" in that doc. Check Plex and iPlayer go quiet in the
      background and carry on from the same point.
- [ ] Control Center timings (Sam, 2026-10-26): Control Center opened much faster than Inputs, because its first frame takes 100 to
      200 ms and about half of the 260 ms grow had gone before it showed. Its grow is now 460 ms (Inputs stays 260); Sam liked it live on
      2026-10-27. Next: build it and confirm on the TV.
- [ ] Sound and Picture lists feel slower to open and close than Theme and Clock (Sam, 2026-10-27). Not a speed problem: 50 and 100 ms
      faster did not help. Their lists grow only downward, so the tile's "Sound" or "Picture" text is seen to slide and fade, while Theme
      and Clock grow both ways and their text barely moves. Next: Sam to come back with a direction.
- [ ] Exiting an app, or pressing Exit on the remote, opens LG Home, not Lumen (Sam, 2026-10-26). Only the Home key is rebound by Input
      Hook. The likely fix is the Native Home item under "Next" (`setDefaultApp`), so check whether it covers Exit and app close first.
- [ ] Draw the lower rows one row at a time, not all at once 3.5 s after launch (`docs/research/startup-perf.md`, Still open).

### Spikes (yes/no answers, noted in `docs/research/spikes.md`)
- [ ] Recents button: rebind it with Input Hook to launch Lumen with `{"view":"recent"}`.
- [ ] Guide button: rebind it with Input Hook to launch Lumen with `{"view":"guide"}`.
- [ ] Colour buttons: which does this remote have, and which key codes reach the app? (Gates the colour-button idea in Later.)
- [ ] `https://sylvan.apple.com/` videos play directly (then the plain HTTP fallback in `lib/wallpaperVideo.ts` goes).
- [ ] `hosts-extra`: capture what this TV calls (LG ads and tracking) and fill it in. It is empty now.

---

## Next: ideas from other launchers (2026-10-24)

From `docs/research/prior-art.md` and upstream (ammar027/lumen-webos). Build without the TV; try each one on the TV later.

- [ ] Frigate front door card: over the home screen or the screensaver, a card with the door camera's live view when Frigate sees a person or
      motion. Lumen listens to Home Assistant over its WebSocket API. Test which stream the TV plays (HLS, or JPEG frames from go2rtc).
- [ ] Home Assistant buttons in Control Center: a short row of chosen entities (for example the lounge lights).
- [x] Native Home: built as Settings > Home and Power > Home button opens (see Verify). `appCategory: ["home"]` is in appinfo.json. The TV step is `setDefaultApp` (upstream
      `docs/native-home.md`), so LG's own Home routes open Lumen, not only the remote's Home key that Input Hook rebinds. Restore LG Home
      before removing Lumen.
      More reports since (2026-11-01, `docs/research/prior-art.md`): Hearth's `general.defaultApps.home` worked on a G5 on webOS 11.2,
      with Back out of an app going to Hearth. If it does not cover Exit here, the fallback is the webOS 10 Home takeover (mount over
      `com.webos.app.home`, then `blockSystemApp` and `releaseBlockSystemApp`) that Glasshouse and lg-luma use.
- [x] Free memory by stopping LG services: built as Settings > Privacy > Stop LG tracking services (see Verify). `admanager` is left
      running, as HBO Max and Disney+ wait for it. `ss.apiadapter.pub` (23 MB) is still unknown.

## Later

In rough order of value:

1. Plex Continue Watching row from the Plex server's On Deck. Tiles open PlxNative. Opening a specific episode needs a launch parameter that
   PlxNative does not have; that is a request to its author, not something this repo can add.
2. Picture and sound mode per app (root; `watch.sh` already knows which app is in front).
3. YouTube subscriptions row (YouTube Data API, TV sign-in with a code), opening videos in youtube-webos with its `contentTarget` launch
   parameter (`{"contentTarget":"v=<id>"}` is documented; `target` is not).
4. Long-press shortcuts per app (for example YouTube: Subscriptions, Search).
5. Bedtime button: lower backlight, Eye Comfort and sleep timer together (all three work as root).
6. Colour buttons (if the spike finds any reach the app): Settings > Remote, an action per button (open an app, Control Center, screen off,
   sleep timer, picture mode, screensaver, nothing). Optionally Input Hook mappings so they work inside other apps too.
7. Auto-switch to an HDMI input when its device turns on; power off when idle.
8. Profiles, and a kids mode with a PIN and allowed hours.
9. Privacy page: LG ad and Live Plus settings, plus hosts-level blocking of LG ad and tracking hosts (check what the Homebrew Channel's own
   telemetry and ad blocking already covers before adding to `hosts-extra`).

## Parked

- Native (GPU) version of Lumen. Reference: PlxNative (Rust, SDL2, OpenGL ES, webosbrew native NDK; GPL-3.0-or-later, so read for approach
  only). If picked up again, start with a native Immich screensaver as a pilot rather than the whole launcher.
- `content-visibility: auto` on off-screen rows: the lower rows are kept painted at opacity 0 on purpose, so showing them is a fade, not a
  first paint. Revisit only with a trace.

## Not doing

- Live blur over large areas or full screen (too expensive on this GPU). Full-screen backdrops are a blurred copy of the wallpaper.
- Google Photos (since March 2025 third-party apps can only read photos they created or ones picked by hand).
- iPlayer, Channel 4 and Channel 5 content rows (no public APIs).
- Splitting the bundle: the app loads from `file://`, where ES module chunks do not load.
- Contributing any of this upstream.
