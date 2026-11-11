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

cleanyourtv mounts its own files over LG's screensaver app instead. That is the same kind of mount as the gear menu route Sam dropped
(a bad file can take the compositor down), so it is not used.

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
- **First release:** the next push to `main` publishes Quick Settings 1.0.0, the screensaver 1.0.0 and the Guide 2.2.1, the versions in
  their `public-<app>/appinfo.json`, since there is no `side-apps.json` yet. It writes the first one; from the release after, each
  version follows its hash. Checked locally: two builds of the same code hash the same, and the version rule gives 1.0.0, 2.2.1 (same
  hash), 2.2.2 (changed), 1.0.5 (changed, after 1.0.4) and 1.1.0 (raised by hand).
- **Dev builds** (`deploy:home`) carry the version in `public-<app>/appinfo.json`, so Settings > Side apps may call a dev build
  up to date when it is not the released one. Off then On puts the released one on.
