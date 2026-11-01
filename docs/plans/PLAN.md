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

## Where things stand (2026-10-26)

- Design audit done (branch `design/system-cleanup`, `docs/research/design-audit.md`): the stylesheet split, three glass materials, two focus
  styles, one type scale (18, 22, 26, 32, 56, 80, 120), the wallpaper category bar with a sliding pill, and left-aligned Settings
  headers. Sam checked each change live on the TV on 2026-10-26, and the build is installed. Step 9's focus-zone helper was dropped
  (no shared logic to pull out). Not committed yet.
- Pick up next: the two Control Center items in Open on the TV (open timing against Inputs, and the sub-menu lag).

---

## Verify on the TV (built, not yet seen working)

Add new items here, with the branch and what to check. Delete an item once Sam has seen it work.

- [ ] TV scripts, Home and Power, and Privacy in Settings (branch `ccr-dba88192-ygl4fc`; `docs/home-and-power-on.md`, `docs/privacy.md`).
      Live first over DevTools where it can be: the rows' text and layout. The root calls need a build with the scripts in it.
      1. Settings > System > TV scripts says Install or Update; OK installs them. Check `/tmp/homehook.log` and that `watch.sh` restarted.
         Also `sh /var/lib/homehook/home-setup.sh check` over SSH once, for the fullest report.
      2. Home and Power > Home button opens: Lumen. Check: Home opens Lumen; Exit in an app opens Lumen; Back out of an app opens Lumen;
         Lumen's LG Home button still opens LG Home. Then switch it back and check LG Home returns.
      3. When the TV turns on: Lumen (only if the row is not "Not available"). Restart the TV now. Check: Lumen is the first thing drawn,
         with no Live TV first; the row no longer says "(after a restart)". Then power off and on with the remote, and with Wake-on-LAN.
      4. Privacy, one row at a time, with an app or two in daily use between each (Plex, iPlayer, YouTube): agreements, Limit ad
         tracking, Live Plus, Block LG ad servers, Stop LG tracking services. Note the values after a restart. Block TV software updates
         last.
      5. If step 2 or 3 works, remove the TV input added on 2026-10-24 (`eim/deleteDevice`, appId `com.samrenfrew.lumen`) and the item
         "Cold start after a reboot" below.

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
- [ ] Control Center timings (Sam, 2026-10-26): Control Center opens much faster than Inputs does, so the two panels feel different. Make
      them match: compare their open durations and easing in source, pick one, and try it live over DevTools with Sam before building.
- [ ] Control Center sub-menus are laggy again (Sam, 2026-10-26; about 16 fps open, 20 fps close, measured 2026-10-22): 21 live blurs
      change every frame, and the key press runs 60-70 ms of script before the first frame. Possible fix: the grid controls drop their live
      blur late in their fade (a switch at the start shows), and the list choices share one blur. Details in `docs/research/frame-rate.md`. Measure
      again first, since the design cleanup may have changed the cost.
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
