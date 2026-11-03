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

Lumen is tested on this one TV only. Other webOS TVs may work, but nobody has checked them.

The original project was written for an LG 65UQ8040PSB on webOS 22 (Chromium 87). Some rules in the code were tuned for that TV. This fork treats each of those rules as an assumption until it works on the TV above. [docs/plans/PLAN.md](docs/plans/PLAN.md) lists what is still to verify.

The build still targets `chrome79` and a classic script, not a module. The TV's graphics chip is weak, so only `transform` and `opacity` animate.

## Requirements

- An LG webOS TV with root access. Lumen runs its TV commands as root through the Homebrew Channel. Without root, Control Center, app removal and the TV-side scripts do not work.
- The [Homebrew Channel](https://www.webosbrew.org/), with root enabled.
- [LG Input Hook](https://github.com/Simon34545/lginputhook), if you want the remote's Home button to open Lumen.
- Optional: an [Immich](https://immich.app/) server for your own photos as the wallpaper and screensaver. See [docs/immich.md](docs/immich.md).

## What this fork adds

These are the main changes since the fork. [docs/plans/PLAN.md](docs/plans/PLAN.md) has the full list and the status of each item.

- **Live TV.** A row of aerial channels with what is on now, a Now and Next guide, and a full week's guide. The Guide button on the remote can open it. See [docs/research/live-tv.md](docs/research/live-tv.md) and [docs/plans/live-tv-build.md](docs/plans/live-tv-build.md).
- **Immich photos.** Your own photos from an Immich server as the wallpaper and as a screensaver. Portrait photos show in pairs. See [docs/immich.md](docs/immich.md).
- **Photo screensaver.** It starts after a set time on the home screen, or from Art mode in the Control Center. It can dim the backlight and has a Print look for paintings.
- **Control Center.** One grid of tvOS-sized glass controls. Sound, Picture, Theme and Clock open a list of choices, as on tvOS 26.
- **Settings.** A stack of pages in the Apple TV layout, not tabs. See [docs/plans/settings-redesign.md](docs/plans/settings-redesign.md).
- **Glass.** A light live blur on the dock and the top buttons, a colour fringe on the dock rim, and a highlight that follows focus. Settings > Appearance > Dock blur turns the dock blur on or off.
- **Open apps.** A dot above each app that is still open in the background. Lumen opens with no launch screen and comes back from other apps faster. See [docs/keep-alive.md](docs/keep-alive.md).
- **Wallpapers.** An Art category of 110 paintings, 300 more landscapes, and a title and credit for each picture.
- **Speed.** A quicker cold start. In live tests on the TV, Down and Up between the rows run at about 40 fps. See [Research](#research).
- **TV tools.** Install over SSH, DevTools access from a Mac, frame-rate scripts, and a watcher that re-pairs a Stadia controller in Moonlight.

The features from the original project are still here. These include the glass dock, the second row and Recent row, folders, video wallpapers, nine languages and the weather.

## Screenshots

These screenshots come from the TV. The wallpaper is "Amanhecer no Hercules" by Carlos Perez Couto (CC BY-SA 3.0).

![Home screen: a glass dock of apps, the clock, weather and hourly forecast top left, Control Center, Settings and Inputs top right](docs/screenshots/home.jpg)
![Moving down: the second row of apps and the Live TV row over a blurred wallpaper](docs/screenshots/more-apps.jpg)
![Live TV row: a channel tile in focus shows what is on now, the time left and what is next](docs/screenshots/live-tv.jpg)
![Control Center: sound, picture, theme, clock, quick actions and backlight](docs/screenshots/control-center.jpg)
![Screensaver: a large clock and date bottom left, with the weather and hourly forecast bottom right](docs/screenshots/screensaver.jpg)
![Settings: one page for each part of the launcher](docs/screenshots/settings.jpg)

## Install from the Homebrew Channel

Each push to `main` publishes a stable build. The GitHub workflow ([.github/workflows/release.yml](.github/workflows/release.yml)) does this.

To get the builds on the TV:

1. Open the Homebrew Channel on the TV.
2. Open the Channel's settings.
3. Add this repository URL: `https://raw.githubusercontent.com/samrenfrew/lumen-webos/hbc-repo/stable/apps.json`
4. Install Lumen from the Channel's app list.

The [releases page](https://github.com/samrenfrew/lumen-webos/releases) also has each `.ipk`. You can install one with [Dev Manager](https://github.com/webosbrew/dev-manager-desktop/releases) or `ares-install`.

The public build does not include third-party app logos, private wallpapers or the SF font. Apps without an icon get a plain tile.

### After you install: the TV-side scripts

The Channel installs only the app. Some features also need the scripts in [tools/tv/](tools/tv/) on the TV. Without them, these do not work:

- Lumen opens again after the TV wakes.
- The Recent row.
- Settings > Screensaver > Enabled replaces LG's screensaver with this app's. It shows your wallpaper pictures, or your Immich photos when Immich is connected.

To install the scripts, run these commands from a copy of this repository. Replace `<tv-ip>` with your TV's address.

```sh
ssh root@<tv-ip> 'mkdir -p /var/lib/homehook /var/lib/webosbrew/init.d'
scp tools/tv/watch.sh tools/tv/stadia.sh tools/tv/home-setup.sh root@<tv-ip>:/var/lib/homehook/
scp tools/tv/60-homehook root@<tv-ip>:/var/lib/webosbrew/init.d/
ssh root@<tv-ip> 'chmod +x /var/lib/homehook/*.sh /var/lib/webosbrew/init.d/60-homehook && /var/lib/webosbrew/init.d/60-homehook'
```

The last command starts the scripts now. After that, the Homebrew Channel starts them at each boot. `stadia.sh` is only for a Stadia controller in Moonlight. It does nothing if you do not have one.

To remove the scripts, run `rm /var/lib/webosbrew/init.d/60-homehook` on the TV, then restart the TV.

### Open Lumen with the Home button

Use LG Input Hook to map the Home button to the app `com.samrenfrew.lumen`.

Settings > Home and Power can make LG's own Home routes (Exit, Back out of an app) open Lumen too, and the TV start on Lumen when it is switched on. See [docs/home-and-power-on.md](docs/home-and-power-on.md). Settings > Privacy withdraws LG's data agreements and blocks its ad servers and tracking services; see [docs/privacy.md](docs/privacy.md). Both need the TV scripts, which Settings > System > TV scripts installs from the app. Neither is tested on the TV above yet.

If you do not use Input Hook, `watch.sh` can do this instead. Create the file `/var/lib/homehook/home-takeover` on the TV. When LG Home opens, the script then opens Lumen. This is slower than Input Hook, because LG Home shows for a moment first.

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

After each start, a dev build writes a "boot" line to `/var/lib/homehook/lumen.log` on the TV. [docs/research/startup-perf.md](docs/research/startup-perf.md) tells you how to read it.

### Other build commands

| Command | What it does |
|---|---|
| `npm run build:ipk` | Builds an `.ipk` into `build/`. |
| `npm run release` | Builds the public `.ipk` into `release/`, with no `app/private/`. |
| `npm run push-font` | Copies the SF font to `/var/lib/homehook/fonts/` on the TV. Lumen copies it into its own folder after each update. |
| `python3 tools/build_catalog.py` | Turns `tools/catalog.json` into the cloud wallpaper list. |
| `python3 tools/harvest_open.py [met\|cleveland\|commons]` | Adds paintings and landscapes, with credits, to the catalogue. |

## Contributing

Lumen is built for one TV and one person's taste. This sets what is accepted:

- Bug reports are welcome. Reports from other TV models are the most useful. Give the model, the webOS version and what you saw.
- Pull requests are accepted for bug fixes only.
- New features and design changes are not accepted. If you want something different, fork the project. The MIT licence lets you do this.

### Commit and push

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

- [watch.sh](tools/tv/watch.sh) keeps the Recent list, opens Lumen after the TV wakes (when Settings > Home and Power > When the TV turns on is Lumen), starts Lumen hidden after a boot, starts the photo screensaver, and ends LG's screensaver when Settings > Screensaver is Off.
- [stadia.sh](tools/tv/stadia.sh) pairs a Stadia controller again in Moonlight after a drop, when you hold Stadia + Y.

LG Input Hook owns the Home button on this TV. `watch.sh` takes over LG Home only when `/var/lib/homehook/home-takeover` exists.

## Research

The TV's graphics chip is the main limit on this project. These documents record what was measured on the TV and what was decided.

| Document | What it covers |
|---|---|
| [docs/research/RESEARCH.md](docs/research/RESEARCH.md) | tvOS design rules, Liquid Glass, TV browser performance, and the cost of each glass effect on this TV. |
| [docs/research/frame-rate.md](docs/research/frame-rate.md) | What limits the frame rate (the GPU draw of each frame), and the scripts that measure it. |
| [docs/research/down-up-lag-review.md](docs/research/down-up-lag-review.md) | A review of the lag on Down and Up, and the test results that got it to about 40 fps. |
| [docs/research/startup-perf.md](docs/research/startup-perf.md) | The cold-start freeze, its fix, and how to read the boot timeline. |
| [docs/keep-alive.md](docs/keep-alive.md) | How to keep apps and Lumen open when you change apps. |
| [docs/research/spikes.md](docs/research/spikes.md) | Short tests on the TV, one for each open question in the plan. |
| [docs/research/live-tv.md](docs/research/live-tv.md) | How to read the channel list, the logos and the guide, and how to tune a channel. |

The main findings:

- Any backdrop filter, such as a live blur, cuts the frame rate about in half while something moves over it.
- The cost is in the GPU draw, not in the app's script or layout.
- A blurred backdrop drawn once into a small canvas costs much less than a live blur over the full screen.
- Refraction (Liquid Glass) works only on the dock. On the panels it drops to 7 fps.

## Credits

Lumen was first made by [ammar027](https://github.com/ammar027/lumen-webos), on top of [partridgeworks/webos-launcher](https://github.com/partridgeworks/webos-launcher) (MIT). See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). This project is not affiliated with LG, Apple or any app vendor.

Licence: [MIT](LICENSE).
