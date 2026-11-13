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

## Where things stand (2026-11-08)

- Lumen 2.2.0 is the latest release, numbered for Lumen Guide (2.1.6 had it first). 2.1.5 brought branch `live-tv-preview` to `main`: the Guide's rail and placeholder shapes, the
  live preview (dev builds), the brightness per input, the Home stand-in fix, the Stadia script fixes, and Lumen Guide, the overlay
  over Live TV (`docs/plans/guide-overlay.md`). From 2.2.0 the release workflow builds Lumen Guide too and lists it
  in the Homebrew Channel beside Lumen. `watch.sh` does nothing for it when it is not installed.
- Credits for the forks and projects this work drew on are in `README.md` (Credits) and `THIRD_PARTY_NOTICES.md`.
- Pick up next: the Verify items below, then Lumen Guide's own (verify) list in `docs/plans/guide-overlay.md` (Status).
- 2026-11-13, branch `live-tv-preview`: Lumen Screensaver is being moved into LG's screensaver's place (`docs/plans/side-apps.md`
  section 4), all four steps built, on the TV as a dev build. The trials over Plex and the plan are in that file.
- 2026-11-11, branch `live-tv-preview`: a third side app, Lumen Screensaver (the photos over the app you are in, so it resumes where you
  left it), Settings > Side apps (turn each side app on or off, and update them), and a version of its own for each side app, set by the
  release from whether its built files changed. Quick Settings and the screensaver join the release. Plan and status:
  `docs/plans/side-apps.md`.

---

## Verify on the TV (built, not yet seen working)

Add new items here, with the branch and what to check. Delete an item once Sam has seen it work.

- [ ] Lumen Screensaver in place of LG's (2026-11-13, branch `live-tv-preview`, dev build on the TV; `docs/plans/side-apps.md`
      section 4). All four steps built; Sam treats it as working and will feed back. Check: over a paused app the photos come up
      with no LG screensaver first and a key goes back to the app; after a reboot it is still Lumen's; Screensaver Disabled gives
      LG's back. Over Lumen's own screensaver it was seen ending itself unseen.

- [ ] Side apps (2026-11-11, branch `live-tv-preview`, not on the TV yet): the (verify) items in `docs/plans/side-apps.md` (Status).
      Install with `npm run deploy:home`, which now installs Lumen Screensaver too, and update the TV scripts (Settings > System >
      TV scripts) for the new `watch.sh`.

- [ ] A wake leaves an app you opened by hand (2026-11-10, branch `live-tv-preview`, watcher on the TV; `tools/tv/watch.sh`, `opened_by_hand`). The wake reads the TV's log for who launched the app in front. Check: switch the TV on, open an app from Lumen within 30 s, and it stays; `/tmp/homehook.log` has "was opened by hand, left in front". Then a TV that wakes into its last app (not opened by hand) still goes to Lumen. Not known: how the log marks the app the TV restores at a cold boot, and a launch from a remote's app button.
- [ ] OK acts on key down where a hold does nothing (2026-11-10, branch `live-tv-preview`, on the TV; `hooks/useOkPress.ts`). Check: OK
      in the Guide, the Full guide and Lumen Guide, on the top buttons, the Inputs list, a channel tile and Settings rows feels quicker,
      and nothing opens twice. A long OK on Control Center's button must not also press its first control. Hold OK on an app still lifts
      it, a second hold opens its menu, and hold OK in Settings > Apps still starts reordering. The first OK on a resting home screen only
      brings the focus up.
- [ ] Channel logos are files (2026-11-10, on the TV; `lib/livetv.ts`, `lib/root.ts` `syncLogos` and `writeLogos`). Seen remotely: 152
      files in `/var/lib/homehook/logos` and in Lumen's `logos/` folder, the old 3 MB value gone from `localStorage`, the row's logos
      loaded, `live-cache` 3 ms (was 38 to 99). Check: logos look as sharp as before on the Live TV row, the Guide and Settings >
      Channels. Lumen Guide shows logos on its first open after this install (it copies the files, and should not read them again).
      After the next install or Channel update the logos come back within a few seconds of launch.
- [ ] Reads on return wait 1.5 s (2026-11-10, on the TV; `RETURN_READ_MS` in `app.tsx`). Coming back from an app, the open-app dots
      and the inputs update about 1.5 s later than before. Not yet measured with `tools/tv/perf/return.mjs`.
- [ ] Lumen's Guide has the overlay's rail in place of the switch at the top right (branch `live-tv-preview`, on the TV). Left from
      Now and Next opens it, and Left past the earliest programme in the Full guide. Up/Down and OK pick a view, Right goes back. Up at
      the top of the list no longer leaves the list. The rail is 340px open and each item is as wide as its name (tried live; the
      build with it is not installed yet).
- [ ] Placeholder blocks while guide data loads (branch `live-tv-preview`, on the TV). In Lumen's Guide, a channel still waiting for
      its programmes shows blocks for the title, times, progress and Next, and the text fades in over them. In the Full guide (Lumen's
      and the overlay's) each row shows three blocks while the schedule loads, and the details at the top show blocks, not "No guide data".
- [ ] Lumen Guide: the (verify) items at the end of `docs/plans/guide-overlay.md` (Status).
- [ ] Lumen Quick Settings, the gear button's overlay (dev build, 2026-11-10): the (verify) items in `docs/plans/quick-settings.md`
      (Status). In Lumen the gear opens Control Center; in other apps it opens Quick Settings over the picture.
- [ ] The overlay guide survives GUIDE then LIST after the reboot (branch `live-tv-preview`). The keep list took effect at the
      2026-11-08 reboot; before it the memory manager closed the hidden guide (docs/plans/guide-overlay.md).
- [ ] Home over Lumen no longer goes black (branch `live-tv-preview`, `tools/tv/home-setup.sh` stand-in, already on the TV). Press Home
      on the remote from Lumen's screensaver and from Lumen's home screen: Lumen comes back after a short black flash at most.
- [ ] Stadia pairing popup (`tools/tv/stadia.sh`, rewritten 2026-11-08, on the TV). Each time Moonlight comes to the front, a popup asks
      to pair. OK starts it, Back skips it. Hold Stadia + Y at any time after OK; it retries for 5 minutes and restarts the TV's Bluetooth
      service after 3 failed tries. "Controller paired." shows once a button press arrives. To try again: Home, then Moonlight. First run
      paired on try 7, right after a Bluetooth restart. The bridge and its files are gone from the TV. The script also copies the
      controller's hidraw node into Moonlight's jail. Sam saw it work on 2026-11-08 (paired on try 1). Still to see: the Bluetooth
      restart after 3 failed tries, and Back on the popup.

- main, 2.1.3: switch-on follows Settings > Home and Power > When the TV turns on. With Lumen, a switch-on from standby still opens
  Lumen. With Last input, it leaves Live TV (or the last app) in front. Update the TV scripts from Settings first (issue 6).

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
      Change set to Over 30 minutes or more: from each start time the backlight moves from the last part's level to the new one, a
      little each minute, with a "brightness:" line per change in the log. Check that a 1-point step each minute is not visible.
      Each input keeps its own backlight (2026-11-06: the night level reached apps but Live TV stayed at the day's 80). Switch from an
      app to Live TV and back: within about 3 s the log shows "backlight <level> on <input>" for Live TV's input, and it matches.
      Seen remotely 2026-11-06: "backlight 80 on dtv" 3 s after Live TV opened. It was already 80, so a real change at 18:00 is still to see.
- [ ] Live preview in the Guide (2026-11-06, dev builds; `components/LivePreview.tsx`, branch `live-tv-preview`). The spike is
      done (`docs/research/live-tv.md`): it plays in 115 to 335 ms, under the page with no hole needed. Then with Settings > System > Live preview on: stop on a channel and the picture shows in about half a second;
      flick through and nothing tunes until you stop; Live TV's last channel is unchanged after; leaving the Guide lets the tuner go.
      Round 2 (2026-11-06, not yet on the TV): Settings > Live TV (dev builds) has Preview in the Guide, Preview on the Live TV row and
      Sound. Check: the logo holds until the picture shows, with no black gap;
      with Sound on, the preview plays the channel's sound; on Home, the focused channel tile on the Live TV row fills with the picture,
      and the lift animation does not misplace it; with Live TV in front, Lumen holds no preview. Also: the Guide button from Live TV
      opens the Guide on the channel being watched (`root.ts` `currentChannel`). Seen 2026-11-06: it does not, because
      `apiadapter/tv/getCurrentChannel` gives the last channel Lumen opened, not one changed to inside Live TV.
      `legacybroadcast.broadcast/getCurrentChannel` needs a `broadcastId` that only Live TV holds. Still to find: a source for Live
      TV's real channel. The Guide button now reopens the Guide afresh each press. The Guide's first preview waits for two frames and
      800 ms (`previewTiming.first`), so it does not slow the Guide's opening.
      Round 3 (2026-11-06): the dwell before tuning is 600 ms, after a 250 ms try made scrolling lag (`docs/research/frame-rate.md`). Sam saw a long black gap after the logo went: the TV keeps the
      picture muted about 2.5 s past `playing` (`docs/research/live-tv.md`). Installed 2026-11-06: the logo holds until the TV's
      `listMuteStatus` says the picture is unmuted (4 s fallback), then fades after 200 ms (Sam's pick, live). A spinner sits where the
      Live tag goes (20 px in on the row tiles) until the picture shows. All seen right live; check the build matches: no black between
      the logo and the picture, the spinner goes as the picture comes, and scrolling the Guide does not lag. `lumen.preview` on DevTools shows
      the waits (change them live) and `last` gives the newest tune's ms to `playing` and to first frames.
      Idea (Sam, 2026-11-06), spike first: when the Guide opens, start Live TV hidden (`applicationManager/launch` with
      `"preload":"full"`, as `watch.sh` does for Lumen) so OK on a channel opens it faster. Check it does not take the preview's tuner,
      play sound, or come to the front, and time OK to picture with and without it. Live TV is often still running already.
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
- [ ] Draw the lower rows one row at a time, not all at once 3.5 s after launch (`docs/research/startup-perf.md`, Still open).

### Spikes (yes/no answers, noted in `docs/research/spikes.md`)
- [ ] Recents button: rebind it with Input Hook to launch Lumen with `{"view":"recent"}`.
- [ ] Colour buttons: which does this remote have, and which key codes reach the app? (Gates the colour-button idea in Later.)
- [ ] `https://sylvan.apple.com/` videos play directly (then the plain HTTP fallback in `lib/wallpaperVideo.ts` goes).
- [ ] `hosts-extra`: capture what this TV calls (LG ads and tracking) and fill it in. It is empty now.

---

## Next: ideas from other launchers (2026-10-24)

From `docs/research/prior-art.md` and upstream (ammar027/lumen-webos). Build without the TV; try each one on the TV later.

- [ ] Frigate front door card: over the home screen or the screensaver, a card with the door camera's live view when Frigate sees a person or
      motion. Lumen listens to Home Assistant over its WebSocket API. Test which stream the TV plays (HLS, or JPEG frames from go2rtc).
- [ ] Home Assistant buttons in Control Center: a short row of chosen entities (for example the lounge lights).

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
