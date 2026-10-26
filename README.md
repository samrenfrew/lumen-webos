# Lumen

Lumen is a tvOS-style home screen for a rooted LG webOS TV.

This repository is a personal fork of [ammar027/lumen-webos](https://github.com/ammar027/lumen-webos). That project is built on [partridgeworks/webos-launcher](https://github.com/partridgeworks/webos-launcher) (MIT). Changes in this fork are not sent back to either project.

## The target TV

| | |
|---|---|
| TV | LG 43UT91006LA |
| Firmware | webOS 25 |
| Web engine | Chromium 120 |
| Set-up | Rooted, with the [Homebrew Channel](https://www.webosbrew.org/) and [LG Input Hook](https://github.com/Simon34545/lginputhook) |

The original project was written for an LG 65UQ8040PSB on webOS 22 (Chromium 87). Some rules in the code were tuned for that TV. This fork treats each of those rules as an assumption until it works on the TV above. [PLAN.md](PLAN.md) lists what is still to verify.

The build still targets `chrome79` and a classic script, not a module. The TV's graphics chip is weak, so only `transform` and `opacity` animate.

## What this fork adds

These are the main changes since the fork. [PLAN.md](PLAN.md) has the full list and the status of each item.

- **Live TV.** A row of aerial channels with what is on now, a Now and Next guide, and a full week's guide. The Guide button on the remote can open it. See [docs/live-tv.md](docs/live-tv.md) and [docs/live-tv-build.md](docs/live-tv-build.md).
- **Immich photos.** Your own photos from an Immich server as the wallpaper and as a screensaver. Portrait photos show in pairs. See [docs/immich.md](docs/immich.md).
- **Photo screensaver.** It starts after a set time on the home screen, or from Art mode in the Control Center. It can dim the backlight and has a Print look for paintings.
- **Control Center.** One grid of tvOS-sized glass controls. Sound, Picture, Theme and Clock open a list of choices, as on tvOS 26.
- **Settings.** A stack of pages in the Apple TV layout, not tabs. See [docs/settings-redesign.md](docs/settings-redesign.md).
- **Glass.** A light live blur on the dock and the top buttons, a colour fringe on the dock rim, and a highlight that follows focus. Settings > Appearance > Dock blur turns the dock blur on or off.
- **Open apps.** A dot above each app that is still open in the background. Lumen opens with no launch screen and comes back from other apps faster. See [docs/keep-alive.md](docs/keep-alive.md).
- **Wallpapers.** An Art category of 110 paintings, 300 more landscapes, and a title and credit for each picture.
- **Speed.** A quicker cold start. In live tests on the TV, Down and Up between the rows run at about 40 fps. See [Research](#research).
- **TV tools.** Install over SSH, DevTools access from a Mac, frame-rate scripts, and a watcher that re-pairs a Stadia controller in Moonlight.

The features from the original project are still here. These include the glass dock, the second row and Recent row, folders, video wallpapers, nine languages and the weather.

## Screenshots

These screenshots show the original project, before this fork's changes. The wallpaper is a public-domain painting from the Cleveland Museum of Art.

![Home screen: a glass dock of apps, date and weather top left, Inputs and Control Center top right](docs/screenshots/home.jpg)
![Moving down: the second row of apps appears over a blurred wallpaper](docs/screenshots/more-apps.jpg)
![Control Center: sound, picture, sleep timer, screensaver, quick actions and volume](docs/screenshots/control-center.jpg)
![Settings, Options tab: a scrolling list with a scroll indicator](docs/screenshots/settings-options.jpg)

## Install from the Homebrew Channel

Each push to `main` publishes a stable build. The GitHub workflow ([.github/workflows/release.yml](.github/workflows/release.yml)) does this.

To get the builds on the TV:

1. Open the Homebrew Channel on the TV.
2. Open the Channel's settings.
3. Add this repository URL: `https://raw.githubusercontent.com/samrenfrew/lumen-webos/hbc-repo/stable/apps.json`
4. Install Lumen from the Channel's app list.

The [releases page](https://github.com/samrenfrew/lumen-webos/releases) also has each `.ipk`. You can install one with [Dev Manager](https://github.com/webosbrew/dev-manager-desktop/releases) or `ares-install`.

The public build does not include third-party app logos, private wallpapers or the SF font. Apps without an icon get a plain tile.

## Set up for development

You need Node 22 or later, Python 3, and SSH access to the rooted TV. The DevTools scripts in `tools/tv/` need Node 22.

1. Install the app's packages: `npm --prefix app install`.
2. Copy `.env.example` to `.env`. The `.env` file is gitignored.
3. Set `LUMEN_TV_HOST` in `.env` to an SSH alias or `root@<tv-ip>`.
4. Put your SSH public key on the TV, so that `ssh root@<tv-ip>` works with no password.
5. Optional: put private icons, wallpapers and the font in `app/private/`. The top of [app/vite.config.ts](app/vite.config.ts) gives the layout. The app works without this folder.

### Check work on the Mac

- Type check and build: `npm --prefix app run build`.
- Preview: build, then serve `app/dist` on port 5198 (the `app-dist` entry in [.claude/launch.json](.claude/launch.json)).

There is no test suite. The preview and the TV are the checks.

### Try a change on the TV over DevTools

The TV serves Chrome DevTools for the running page. Try each change there first. Install a build only when the change is right. This saves one install for each try.

1. Open a tunnel to the TV's DevTools port:

   ```sh
   ssh -f -N -L 19998:127.0.0.1:9998 root@<tv-ip>
   ```

2. Use [tools/tv/cdp.mjs](tools/tv/cdp.mjs) to work on the live page:
   - `node tools/tv/cdp.mjs css '<css>'` adds CSS to the page. `css ''` removes it.
   - `node tools/tv/cdp.mjs eval '<expr>'` runs JavaScript in the page.
   - `node tools/tv/cdp.mjs shot out.png` saves a screenshot.
   - `node tools/tv/cdp.mjs key down enter back` presses remote keys.
3. Write each change into the source when it is right. A relaunch removes the live changes.

A screenshot shows black where a video wallpaper plays. The video is on a hardware layer under the page.

### Install a dev build over SSH

```sh
npm run deploy:home
```

This command type checks, builds Lumen as `com.samrenfrew.lumen` with the dev tools on, copies the `.ipk` to the TV, installs it and starts it. It replaces the installed Lumen, so the Home button opens the new build.

`npm run deploy` installs a second copy, `com.samrenfrew.lumen.dev` ("Home Dev"), next to the main one. The Home button does not open this copy.

After each start, a dev build writes a "boot" line to `/var/lib/homehook/lumen.log` on the TV. [docs/startup-perf.md](docs/startup-perf.md) tells you how to read it.

### A TV in Developer Mode (not rooted)

Lumen also runs on a TV with only LG's Developer Mode app. It starts as an app there: the Home button still opens LG's home, and the root features (the live app list, Live TV, the Control Center's TV settings) are off. `tools/devmode.sh` installs and reverts builds on such a TV. It logs in as `prisoner` on port 9922 and works without `scp`.

1. In the Developer Mode app, turn on Dev Mode Status and Key Server. Download the key from `http://<tv-ip>:9991/webos_rsa`.
2. Make a copy of the key without the passphrase the app shows: `cp webos_rsa webos_rsa_nopass && chmod 600 webos_rsa_nopass && ssh-keygen -p -m PEM -f webos_rsa_nopass`.
3. Set `LUMEN_TV_IP` and `LUMEN_TV_KEY` in `.env`.

| Command | What it does |
|---|---|
| `sh tools/devmode.sh deploy` | Saves Lumen's settings from the TV, builds, installs and starts the build. The first time, it also downloads the release the TV had. |
| `sh tools/devmode.sh revert` | Installs that release again and puts back the settings from before the first deploy. |
| `sh tools/devmode.sh backup` / `restore <file>` | Saves the settings into `backup/`, or puts a saved copy back. |
| `sh tools/devmode.sh find-apps` | Lists the apps the TV has. Developer Mode refuses `listLaunchPoints`, so it tries each id in `tools/devmode/candidates.txt`. |
| `sh tools/devmode.sh home` | Puts the apps and folders from `tools/devmode/home.json` on the home rows. |
| `node tools/devmode/icons.mjs` | Draws one matching tile for each app in `tools/devmode/apps.mjs` into `app/private/iconpack/`, and writes the catalogue (`catalog.local.ts`) and `home.json` to go with it. Brand logos come from `tools/logos/raw/` when you have them there; without them each tile gets a wordmark or icon. `LUMEN_TILE_SCALE=0.45` saves smaller tiles for a TV short on memory. |

The DevTools port (9998) needs no root. `tools/tv/perf/keyfps.mjs` and `keytrace.mjs` measure frames and main-thread work per key over the same tunnel.

### Other build commands

| Command | What it does |
|---|---|
| `npm run build:ipk` | Builds an `.ipk` into `build/`. |
| `npm run release` | Builds the public `.ipk` into `release/`, with no `app/private/`. |
| `npm run push-font` | Copies the SF font to `/var/lib/homehook/fonts/` on the TV. Lumen copies it into its own folder after each update. |
| `python3 tools/build_catalog.py` | Turns `tools/catalog.json` into the cloud wallpaper list. |
| `python3 tools/harvest_open.py [met\|cleveland\|commons]` | Adds paintings and landscapes, with credits, to the catalogue. |

## Commit and push

Write a commit subject that names the area, then says what changed for the user. Use plain words.

```
Live TV: channel logos fill the tile, clear of the progress bar
```

Start a commit that changes only docs with `docs/<file>:`.

Do the work on a branch named `<area>/<topic>`, for example `home/dock-names`. Know what each push does:

- A push to a branch other than `main` makes a dev build (`1.1.0xxx`). The workflow keeps it as a run artifact for 14 days. It does not publish it.
- A push to `main` publishes a stable release and tags it `v<version>`. The version comes from `app/public/appinfo.json`. If that tag exists, the workflow uses the next free patch number. To start a new minor or major version, change `appinfo.json`.
- A Channel update replaces a build that you installed over SSH.

Do not push only to get a build onto the TV. Use `npm run deploy:home` for that.

## TV-side scripts

The scripts in [tools/tv/](tools/tv/) run on the TV. They go in `/var/lib/homehook/`, and [tools/tv/60-homehook](tools/tv/60-homehook) in `/var/lib/webosbrew/init.d/` starts them at boot.

- [watch.sh](tools/tv/watch.sh) keeps the Recent list, opens Lumen after the TV wakes, starts Lumen hidden after a boot, starts the photo screensaver, and ends LG's screensaver when Settings > Screensaver is Off.
- [stadia.sh](tools/tv/stadia.sh) pairs a Stadia controller again in Moonlight after a drop, when you hold Stadia + Y.

LG Input Hook owns the Home button on this TV. `watch.sh` takes over LG Home only when `/var/lib/homehook/home-takeover` exists.

## Research

The TV's graphics chip is the main limit on this project. These documents record what was measured on the TV and what was decided.

| Document | What it covers |
|---|---|
| [RESEARCH.md](RESEARCH.md) | tvOS design rules, Liquid Glass, TV browser performance, and the cost of each glass effect on this TV. |
| [docs/frame-rate.md](docs/frame-rate.md) | What limits the frame rate (the GPU draw of each frame), and the scripts that measure it. |
| [docs/down-up-lag-review.md](docs/down-up-lag-review.md) | A review of the lag on Down and Up, and the test results that got it to about 40 fps. |
| [docs/startup-perf.md](docs/startup-perf.md) | The cold-start freeze, its fix, and how to read the boot timeline. |
| [docs/keep-alive.md](docs/keep-alive.md) | How to keep apps and Lumen open when you change apps. |
| [docs/spikes.md](docs/spikes.md) | Short tests on the TV, one for each open question in the plan. |
| [docs/live-tv.md](docs/live-tv.md) | How to read the channel list, the logos and the guide, and how to tune a channel. |

The main findings:

- Any backdrop filter, such as a live blur, cuts the frame rate about in half while something moves over it.
- The cost is in the GPU draw, not in the app's script or layout.
- A blurred backdrop drawn once into a small canvas costs much less than a live blur over the full screen.
- Refraction (Liquid Glass) works only on the dock. On the panels it drops to 7 fps.

## Credits

Lumen was first made by [ammar027](https://github.com/ammar027/lumen-webos), on top of [partridgeworks/webos-launcher](https://github.com/partridgeworks/webos-launcher) (MIT). See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). This project is not affiliated with LG, Apple or any app vendor.

Licence: [MIT](LICENSE).
