# Side apps: the screensaver as an overlay, Settings > Side apps, and versions of their own

Agreed with Sam on 2026-11-11. Sam asked for the first build on default assumptions and will try it on the TV. Items marked
**(default)** were picked by Claude; the rest Sam decided. Everything that needs a TV answer is in **Status** as (verify).

Lumen has two side apps: Lumen Guide (`guide-overlay.md`) and Lumen Quick Settings (`quick-settings.md`). This adds a third, Lumen
Screensaver, a way to turn each one on or off and update them from Lumen, and gives each its own version number.

## 1. Lumen Screensaver: the photos over the app you are in

### Why

Today watch.sh opens Lumen with `{"view":"ambient","from":"<app>"}` when LG's screensaver starts over another app. Lumen is a full card,
so the app goes to the background, and a key that ends the screensaver launches it again. Some apps take that relaunch as "go to my
start screen", and the memory manager may close an app in the background that is not on the keep list (`docs/research/spikes.md`,
"Background apps"). So the app does not resume where you left it. Quick Settings' Screensaver button is worse: it opens Lumen with no
`from`, so a key goes to Lumen's home screen.

An overlay window leaves the app as the card in front, untouched, as Lumen Guide does over Live TV and the overlay test did over a
playing YouTube video (`docs/research/spikes.md`, "Overlay window from a developer app"). codefarm-eu moved its own screensaver hand-back
into an overlay for the same reason (`docs/research/prior-art.md`). The 2026-11-10 sweep marked the screensaver "Done, no gain"; it did
not weigh the lost place, which is why this plan replaces that line.

cleanyourtv mounts its own files over LG's screensaver app instead. This plan first ruled that out as the same kind of mount as the
gear menu route Sam dropped (a bad file can take the compositor down). That was wrong for webOS 25: there the screensaver is an app
folder like LG Home, and the mount puts a web app in its place, as the Home takeover already does (`tools/tv/home-setup.sh`). Section 4
replaces watch.sh's hand-over with that mount, after the trials of 2026-11-13.

### What it does

- **A separate app**, "Lumen Screensaver", id `com.samrenfrew.lumen.saver`, hidden from LG's app list, an overlay window like the other
  two.
- **Lumen keeps its own screensaver.** Lumen's idle timer (1 minute by default) and Control Center's Art button open Lumen's own, with
  the clock sliding over from the home screen. When LG's screensaver starts while Lumen is in front (Lumen's own timer is off, or
  Lumen is on Settings or the Guide), watch.sh opens Lumen's own as today.
- **Over any other app**, when LG's screensaver starts and Settings > Screensaver is Enabled, watch.sh launches Lumen Screensaver
  instead of Lumen. Without the app installed, it opens Lumen as today.
- **Quick Settings' Screensaver button** launches Lumen Screensaver too, and falls back to Lumen when it is not installed.
- **Starts clear, fades in when the photo is ready.** The window is transparent, so the app stays in sight while the first photo
  downloads and decodes. Then the screensaver fades in over 600 ms (opacity only). If no photo has come after 8 s **(default)**, it
  fades in anyway on black, with the clock.
- **Any key ends it**, the same as Lumen's: a 500 ms fade out, then the app closes and the app under it has the remote again. A key
  pressed while it is still clear ends it too.
- **Not kept running.** It starts when it is needed and closes when it ends, so its photos take no memory the rest of the time
  **(default)**. A cold start is 1.6 to 4 s for the other overlays; nobody waits on it, since the app stays in sight meanwhile.
- **Lowers the backlight** as Lumen's does (Settings > Screensaver > Brightness), and puts it back before it closes.

### Where it gets Lumen's settings

Each app has its own `localStorage`, so the overlays cannot see Lumen's settings (that is why the guide does not follow the language or
the clock setting). Lumen already writes every setting to `/var/lib/homehook/lumen-config.json` 3 s after each change (`lib/backup.ts`),
and the Immich server to `/var/lib/homehook/immich.json`. The screensaver reads both at each start through `lib/root.ts`
(`readFile`), the same calls Lumen uses. No new file.

The file is `{"version": 1, "config": {...}}`. A side app reads it with `parseConfig`, which fills in defaults for anything missing and
drops what it does not know, so a newer Lumen with an older side app, or the reverse, still works. If Lumen ever changes what a
field means, `version` goes up and the side apps check it.

From the file it uses: the photo source (wallpaper pictures or Immich, and which), time on each photo, brightness, Print look,
night dimming, reduce motion, the clock's AM/PM, language, clock format, the weather city and the weather widget. Weather is fetched
by the screensaver itself, from the same service as Lumen.

Bundled and private wallpapers are copied into its own folder at build time, since a page may load only from its own folder. Cloud
wallpapers and Immich photos load from the network, as in Lumen.

### How it is built

- `app/saver.html` and `app/src/saver/main.tsx`, which mount `components/Ambient.tsx` unchanged apart from an `onShown` callback.
  Styles: `theme/home.css` (the clock), `forecast.css`, `ambient.css`, and `theme/lsaver.css` for the clear page and the fade in.
- Appinfo in `app/public-saver/`. `npm --prefix app run build:saver` builds it into `app/dist-saver/` (`LUMEN_TARGET=saver`).
  `npm run deploy:saver` installs it over SSH, and `npm run deploy:home` installs it after the other two.
- watch.sh: `SAVER=com.samrenfrew.lumen.saver`. In the photo-screensaver branch, with no Lumen build in front and the app installed, it
  launches the screensaver with `{"from":"<app>"}` (for the log only). While `/tmp/lumen-ambient-on` exists and the screensaver is
  running, LG's screensaver is only ended, as it is while Lumen's own shows.

## 2. Settings > Side apps

A group of its own in Settings, under Privacy **(default)**.

- **One row per side app** with an on/off switch: Lumen Guide, Lumen Quick Settings, Lumen Screensaver. The row's value says Off, On
  with the version, or Working.
- **On** downloads the newest `.ipk` from the stable Homebrew Channel listing and installs it. **Off** uninstalls it. Both then restart
  the TV scripts (as the TV scripts row does), so watch.sh's key binds follow at once. watch.sh already does nothing for an app that is
  not installed.
- **Update**, shown when at least one side app is on: "Up to date", or "Update 2 apps"; OK updates every side app that is on and
  behind.
- The listing is the Homebrew Channel's own index,
  `https://raw.githubusercontent.com/samrenfrew/lumen-webos/hbc-repo/stable/apps.json`, read when the group opens. It has each app's
  version, `.ipk` URL and SHA-256. Nothing new to host. Updating from the Homebrew Channel works as before; Lumen then just sees the new
  version.

### How the install works

`lib/root.ts` gets three fixed commands, in the same shape as the rest:

- `sideAppVersions()`: the `version` in each side app's installed `appinfo.json`, or none.
- `installSideApp(id, url, sha256)`: the id must be one of the three, the URL a release download from this repo for that id, and the
  SHA-256 64 hex characters. It downloads the `.ipk` to `/tmp` with curl, checks the SHA-256 (refuses on a mismatch), installs it with
  `com.webos.appInstallService/dev/install` (what `deploy:home` uses), removes the file and restarts the TV scripts.
  The SHA-256 comes from the listing, which the page reads over HTTPS, so the download itself is checked even if curl on the TV
  cannot check GitHub's certificate **(verify)**.
- `uninstallSideApp(id)`: the same removal as Settings > Apps uses, limited to the three ids, then the TV scripts restart.

The Homebrew Channel's own service may have an install method too; it is not used, since `dev/install` is already seen working here.

### Not doing (yet)

- Adding a newly installed side app to the memory manager's keep list (`com.webos.memorymanager.KeepAlive`). On this TV the Guide and
  Quick Settings are already on it. Someone else's TV would get the same warm opens only after doing that by hand.
- A "needs Lumen x.y" check. The settings file's `version` covers what matters now.

## 3. Versions of their own

Each side app has its own version, and it goes up only when that app changes. Lumen's version still goes up with every release.

- Quick Settings and the screensaver start at **1.0.0**. The Guide was released as 2.2.0 with Lumen's number, and a lower version would
  read as older to the Homebrew Channel, so the Guide carries on its own line from **2.2.1** (it has changed since 2.2.0) **(default)**.
- Side apps share code with Lumen (Control Center, the guide rows, the screensaver), so a change to shared code can change a side app
  with nobody touching its folder. So the release decides, not a person:
  - `tools/build_release.py` builds each side app and takes a SHA-256 of its built files (every file's path and bytes, with the
    `version` taken out of `appinfo.json`).
  - It compares that with the last release's, kept in `stable/side-apps.json` on the `hbc-repo` branch.
  - The same: the version stays. Different: the patch goes up by one. A version raised by hand in `public-<app>/appinfo.json` above the
    last release's wins, for a minor or major step.
  - It writes the new `side-apps.json` beside the `.ipk` files, and the workflow puts it on `hbc-repo` with the listing.
- A side app whose version stayed is still built and listed (its `.ipk` is the same app), so the listing always has all of them; the
  Channel and Lumen see the same version and offer no update.
- On a branch push, the dev build artifact has all four `.ipk` files, with the same version rule.

## 4. Lumen Screensaver in place of LG's (agreed 2026-11-13; Sam: treat it as working, he will feed back)

### Why

Over Plex, watch.sh's hand-over shows LG's screensaver, then Plex, then the photos: it sees LG's screensaver up to a second late,
launches ours, and ends LG's 1 s later, but ours takes about 4.5 s to show. The trials (Status) mounted our page in LG's screensaver
app's place, as Aerial does (`docs/research/prior-art.md`). The TV then launched it on its own timer, it drew in 1.3 to 1.5 s, LG's
screensaver never showed, and a key went back to Plex where it was. Two runs out of three were clean. The first hung (Status).

### What changes

- **The TV launches it.** With Settings > Screensaver Enabled and Lumen Screensaver installed, LG's screensaver app
  (`com.webos.app.screensaver`) is our page. watch.sh no longer launches anything when LG's screensaver starts over an app.
- **Same app, two ways in.** The build in `com.samrenfrew.lumen.saver` stays the source. A copy of its folder is kept in
  `/var/lib/homehook/saver-app` with LG's id in `appinfo.json` (`"defaultWindowType": "screenSaver"`, `"type": "web"`, everything else as
  ours), and that copy is bind-mounted over `/usr/palm/applications/com.webos.app.screensaver`. A copy, not a mount of our folder, so an
  install of the side app cannot change the page under the TV while it is up, and our own `appinfo.json` stays ours. Quick Settings'
  Screensaver button still launches `com.samrenfrew.lumen.saver` as an overlay.
- **One script owns it**, the same shape as the Home takeover: `home-setup.sh saver-mount on|off|check` stages the copy, mounts it, and
  has the app manager read it with `blockSystemApp` then `releaseBlockSystemApp` (no restart of the app manager, so open apps stay
  open). If the app manager does not report `"type": "web"` after that, it unmounts and LG's screensaver stays. `off` unmounts and
  does the same block and release. Every step goes to `/tmp/homehook.log`.
- **At boot**, `home-setup.sh guard` (which `60-homehook` already runs at each boot and TV scripts restart) runs `saver-mount on` when
  `/var/lib/homehook/saver-mount` exists, as it does for the Home takeover's `home-mount`.
- **Kept up to date.** After Settings > Side apps installs or updates the screensaver, and after `deploy:home`, the copy is staged again
  if the side app's version or files changed. A changed copy is swapped only while the screensaver is not up.
- **Lumen sets it.** Settings > Screensaver Enabled with the side app installed writes `saver-mount` and runs `on`. Disabled, or the side
  app turned off in Side apps, runs `off`. New fixed commands in `lib/root.ts`, in the same shape as `homeSetup`.
- **watch.sh** keeps every other screensaver rule as it is: Disabled (end LG's), the video wallpaper, Frame, and the old hand-over as
  the fallback when the mount is off or failed.

### When Lumen's own screensaver is already up

Lumen's idle timer (1 minute) shows Lumen's own screensaver. LG's timer (10 minutes) still runs, since nothing is pressed, so 9
minutes later the TV launches ours on top of Lumen's.

- **Our page stays clear and ends it.** At start it checks `/tmp/lumen-ambient-on` (Lumen writes it while its own screensaver shows).
  If it is there, the page draws nothing, asks the TV to end the screensaver (`turnOnScreen`, through `lib/root.ts`) and closes. It is
  clear until its first photo is ready, so Lumen's photos carry on with no visible change. watch.sh still ends LG's screensaver in this
  case, as today, as a backstop.
- **Lumen in front without its own screensaver** (Settings, the Guide, or its timer off): ours shows over Lumen, and a key leaves Lumen
  where it was. Today watch.sh opens Lumen's own instead. Both show the same photos; this way has one path for every app.
- **(verify)** Whether `window.close()` alone ends the TV's screensaver state, or only `turnOnScreen` does.
- **To try later:** Aerial sets `com.webos.service.screenmanager/setScreenSaverMode` to `manual` for its own id. If Lumen could do that
  for itself while its own screensaver shows, LG's timer might not fire over Lumen at all. Not known.

### The hang

On the first trial no key ended it, and Plex was closed while it was up (Lumen started again 20 s in). The second trial could not
see why: the TV lists no DevTools page under LG's screensaver id. So:

- The page writes each key it gets, and its close, to `lumen.log` (dev builds), so a second hang shows whether keys arrived.
- watch.sh notices when the app the screensaver came up over has been closed while it is up, logs it, and ends the screensaver, so
  the screen does not sit on photos with nothing under them.
- If it is stuck, the remote's power button still turns the TV off. **(verify)** that the next switch-on shows Lumen, not the photos.

### Not doing

- LG's video ads screensaver (`com.webos.app.videoads`, present on this TV). Aerial mounts over it too, because some TVs launch it
  instead. Lumen's Privacy settings already block LG's ads; mount it only if the log ever shows it launched.

### Steps (each leaves the TV working)

1. The page: key and close log lines, the `lumen-ambient-on` check, and `turnOnScreen` before close. Harmless as an overlay.
2. `home-setup.sh saver-mount` and its line in the guard. Tried over SSH with Sam watching, before Lumen calls it.
3. watch.sh: no hand-over while the mount is on, and the closed-app check.
4. Settings: Screensaver Enabled and Side apps call it. Re-stage after installs.

## Status

First build 2026-11-11, on branch `live-tv-preview`. Type check and build pass for all four apps. In a desktop browser the screensaver
page stays clear, fades in once the first picture has loaded, and a key takes it down. Nothing below has been seen on the TV yet.

- **(verify)** Lumen Screensaver over Plex (paused) and over YouTube: LG's screensaver starts after its timer, the app stays in sight, the
  photo fades in over it, a key fades it out and the app is where you left it, with the remote.
- **(verify)** The fade-in: no flash of black before the photo, and the 8 s fallback on a TV with no network.
- **(verify)** The window shows without the 8 s wait: the page paints an invisible dot at once, as the guide's hidden start does.
- **(verify)** Backlight: lowered while it shows (Brightness set to Dim), back to the level for now after a key.
- **(verify)** Memory with the screensaver up over Plex, the Guide and Quick Settings hidden (`tools/tv/memwatch.sh`). A 4K photo is
  about 33 MB decoded and a crossfade holds two; Immich previews are smaller.
- **(verify)** Quick Settings' Screensaver button opens it over the app.
- **(verify)** Settings > Side apps: the versions shown, Off then On for each (uninstall, download, SHA-256 check, install), the key
  binds following without a restart of the TV, and Update. Also whether curl on the TV can fetch from GitHub over HTTPS.
- **(verify)** Over Live TV: whether LG's screensaver ever starts there, and if it does, that Live TV keeps playing under it.
- **Seen 2026-11-13, over Plex (paused), LG timer 3 min:** watch.sh ends LG's screensaver 1 s after it launches ours, but ours takes about 4.5 s to show, so LG's screensaver shows, then Plex, then the photos. A key closes it and Plex is where it was. Tried instead: our page bind-mounted over `com.webos.app.screensaver` as a web app (the Aerial way, `docs/research/prior-art.md`), with watch.sh stopped. The TV launched it on its timer and it drew in 1.5 s with no LG screensaver, but no key ended it and Plex was closed while it was up (Lumen relaunched 20 s in). Ended with `turnOnScreen`, then unmounted and `blockSystemApp`/`releaseBlockSystemApp` put LG's back. Cause not known: keys may have gone to Lumen's card under it, or the page's close path fails under LG's id. Next try with DevTools on the page.
- **Second trial, same day, same setup:** two runs over paused Plex, both clean. The photos came up straight from Plex (Sam: "went straight into the photos"), first frame in 1.3 s, OK went back to Plex with Plex still running. The TV does not list a page under LG's screensaver id in DevTools, so the key probe in the page could not be read; the first trial's hang is still unexplained (Plex closed and Lumen relaunched under it). A backstop started with `setsid nohup` over SSH died when the session closed. LG's screensaver was put back and watch.sh restarted after.
- **Section 4, step 1 built (2026-11-13, dev build on the TV, as the overlay):** `lsaver key`, `lsaver close` and `lsaver open` lines in
  `lumen.log`, `endScreensaver` before close, and the `lumenOwnSaverUp` check (`lib/root.ts`). Seen over Plex: "open: showing" with a
  stale `/tmp/lumen-ambient-on` left by an install mid-show (Plex in front, so the check ignored it), then "key 13", "close: key", the
  flag cleared and Plex in front. **(verify)** with the mount: the page ends itself over Lumen's own screensaver with nothing drawn.
- **Section 4, step 2 built (2026-11-13, scripts on the TV):** `home-setup.sh saver-mount on|off`, `saver=` in `state`, and the guard
  puts the mount back at boot and stages the copy again after the side app changes (so `60-homehook` did not change). `saver-mount on`
  worked on the TV (the app manager reported `web`), and `off` put LG's back (`flutter`). Over SSH run it with `ssh -tt`: in a session
  with no terminal, `luna-send` printed nothing to a pipe, so the first `on` read the app manager's answer as a refusal and gave LG's
  back. The guard at boot is not affected (it has mounted the Home takeover again after every restart). Seen at 12:33 with the mount
  on: over Lumen's own screensaver the TV launched ours, the page logged "Lumen's own screensaver is up, ending", drew nothing, and the
  TV was back to Active 35 ms after it entered Screen Saver (the window was up 0.4 s, clear).
- **Section 4, steps 3 and 4 built (2026-11-13, dev build and scripts on the TV):** watch.sh stands back while the mount is on (only
  logs, and ends the screensaver if the app it came up over leaves the front), and Lumen runs `saver-mount on` (or `off`) 10 s after
  each launch and after Settings > Screensaver changes. `saver-mount off` does nothing when nothing is in place, so the launch call is
  cheap. **(verify)** over Plex and YouTube with the scripts installed: straight into the photos, a key back to the app, and
  `homehook.log` saying "Lumen Screensaver (in LG's place)". **(verify)** after a reboot the guard mounts it again
  (`home-setup.sh status` says "saver-mount: on"). **(verify)** Screensaver Disabled in Settings puts LG's back.
- **First release:** the next push to `main` publishes Quick Settings 1.0.0, the screensaver 1.0.0 and the Guide 2.2.1, the versions in
  their `public-<app>/appinfo.json`, since there is no `side-apps.json` yet. It writes the first one; from the release after, each
  version follows its hash. Checked locally: two builds of the same code hash the same, and the version rule gives 1.0.0, 2.2.1 (same
  hash), 2.2.2 (changed), 1.0.5 (changed, after 1.0.4) and 1.1.0 (raised by hand).
- **Dev builds** (`deploy:home`) carry the version in `public-<app>/appinfo.json`, so Settings > Side apps may call a dev build
  up to date when it is not the released one. Off then On puts the released one on.
