# Lumen Native

Lumen Native is a tvOS-style home screen for a rooted LG webOS TV. It is a native app, written in Rust and drawn with OpenGL. It replaces Lumen, the web app that came before it, and does what that app did, on far less of the TV's memory and graphics power.

This repository is a personal fork of [ammar027/lumen-webos](https://github.com/ammar027/lumen-webos), which is built on [partridgeworks/webos-launcher](https://github.com/partridgeworks/webos-launcher) (MIT). Changes here are not sent back to either project.

## Why it moved to native

The web Lumen ran inside the TV's web engine, and that engine was the limit. A low-end TV has a weak graphics chip and little memory to spare, so the web Lumen dropped frames on most moves, and its screensaver used so much memory that the TV restarted on its own. Lumen Native draws each screen itself, so it keeps only what is on screen.

Measured on the TV below:

| | Web Lumen | Lumen Native |
|---|---|---|
| Graphics memory, home screen | about 86 MB | about 30 to 33 MB |
| Graphics memory, screensaver | about 180 MB | about 30 MB |
| Down and Up between the rows | about 40 frames a second, with dropped frames | every frame the screen shows, none late |
| From launch to the first frame | the full home screen after about 3.5 s | 0.7 to 1.0 s |
| Coming back from another app | | 0.2 to 0.4 s |

The screen shows 50 or 60 frames a second, depending on the last video it played. The web engine always drew 60, so at 50 it also dropped one frame in six. Lumen Native follows the screen's own beat.

The details are in [docs/plans/native-rewrite.md](docs/plans/native-rewrite.md) and [docs/plans/native-saver.md](docs/plans/native-saver.md).

## The target TV

| | |
|---|---|
| TV | LG 43UT91006LA |
| Firmware | webOS 25 |
| Set-up | Rooted, with the [Homebrew Channel](https://www.webosbrew.org/) |

Lumen Native is built for this one TV: its graphics chip, its SDL and its system libraries. It may not start on other webOS TVs, and nobody has tried it on one. If you have a different TV, see [The old web Lumen](#the-old-web-lumen) below.

## What it does

- **Home screen.** A glass bar of your apps under the clock, date and weather, with the hourly forecast. More rows of apps below, a Recent row, folders, dots under apps that are still open, re-ordering, and a menu on a long OK press. The pointer works too.
- **Liquid glass.** The app bar and glass buttons bend the wallpaper at their edges. Settings > Appearance turns it off.
- **Live TV.** A row of aerial channels with what is on now and next, and a live picture of the channel in focus. The Guide and the Full guide, with typed channel numbers. Settings > Channels picks and orders the row.
- **Lumen Guide.** A narrow channel guide over the Live TV picture. LIST opens it on Now, GUIDE opens the Full guide.
- **Control Center and Inputs.** Sound, picture, backlight, Eye Comfort, a sleep timer, the screensaver and the TV's inputs.
- **Lumen Quick Settings.** Control Center over any app, on the remote's gear button.
- **Screensaver.** Your wallpaper pictures, or your own photos from an [Immich](https://immich.app/) server, with a large clock and the weather. Portrait photos show in pairs, and each photo slowly grows so nothing stays still. It can dim the backlight.
- **Lumen Screensaver.** The same screensaver over the app you are watching, in LG's screensaver's place, so a key takes you back to where you left off.
- **Settings.** Wallpapers (Landscapes and Art, each with its title and credit), Immich, the brightness schedule for each part of the day, the side apps, Home and Power, Privacy (withdraws LG's data agreements and blocks its ad servers), TV scripts and backup.

### Left out from the web Lumen

These did not come over. Some may come back later.

- The video wallpaper.
- The light look. Lumen Native is dark only.
- Focus strength and hide the dock as settings.
- The cloud wallpaper library. The built-in Landscapes and Art stay.
- Languages other than English.

## Screenshots

These screenshots come from the TV. The wallpaper is "Amanhecer no Hercules" by Carlos Perez Couto (CC BY-SA 3.0).

![Home screen: a glass bar of apps, the clock, weather and hourly forecast top left, Control Center, Settings and Inputs top right](docs/screenshots/home.jpg)
![Moving down: more rows of apps and the Live TV row over a blurred wallpaper](docs/screenshots/more-apps.jpg)
![Live TV row: a channel tile in focus shows what is on now, the time left and what is next](docs/screenshots/live-tv.jpg)
![Control Center: sound, picture, quick actions and backlight](docs/screenshots/control-center.jpg)
![Screensaver: a large clock and date bottom left, with the weather and hourly forecast bottom right](docs/screenshots/screensaver.jpg)
![Settings: one page for each part of the home screen](docs/screenshots/settings.jpg)

## Install

You need a rooted LG TV with the [Homebrew Channel](https://www.webosbrew.org/), with root turned on. Lumen Native runs its TV commands as root, so it does not work without it.

1. Open the Homebrew Channel on the TV, then its settings.
2. Add this repository URL: `https://raw.githubusercontent.com/samrenfrew/lumen-webos/hbc-repo/stable/apps.json`
3. Install **Lumen Native** from the Channel's app list. On the TV it is called "Home".
4. Open it, then go to Settings > System > TV scripts and press OK. This puts the scripts Lumen Native needs on the TV and starts them. They start again at each boot.
5. Optional: in Settings > Side apps, turn on Lumen Guide, Lumen Quick Settings and Lumen Screensaver. They do not show in the TV's app list. The remote's buttons and the screensaver open them.

The [releases page](https://github.com/samrenfrew/lumen-webos/releases) also has each `.ipk`. You can install one with [Dev Manager](https://github.com/webosbrew/dev-manager-desktop/releases) or `ares-install`.

The public build has no third-party app logos and no SF font. Apps without an icon get a plain tile, and the text uses LG's own font.

### What the TV scripts do

Without the TV scripts, these do not work: the Recent row, the screensaver in LG's place, the brightness schedule, Home and Power, Privacy, and the LIST, GUIDE and gear buttons for the side apps.

The scripts live in `/var/lib/homehook/` on the TV, and the Homebrew Channel starts them at boot through `/var/lib/webosbrew/init.d/60-homehook`. To remove them, delete that file on the TV and restart it.

### Open Lumen Native with the Home button

Settings > Home and Power > Home button opens makes the Home button, Exit and Back out of an app open Lumen Native. The same page can make the TV start on Lumen Native when it is switched on.

If you use [LG Input Hook](https://github.com/Simon34545/lginputhook), you can map the Home button to `com.samrenfrew.lumen.native` there instead.

### Moving from the web Lumen

Lumen Native has a new app id, `com.samrenfrew.lumen.native`, so it installs beside the web Lumen and never replaces it through a Channel update.

- **Your settings come with you.** Both apps read the same settings file, so your apps, wallpaper, Immich server and brightness schedule are there on the first launch.
- **Remove the old one when you are happy.** Settings > Home and Power shows an "Old Lumen" row while the web Lumen or its side apps are installed. Remove points the Home button and power-on at Lumen Native first, then uninstalls the old apps. If the TV still opens the old one at power-on, they go at the next restart.

## The old web Lumen

The web Lumen is frozen at version 2.2.1. It stays downloadable, but it gets no more fixes. Use it if Lumen Native does not start on your TV, or if you need something it left out.

The Homebrew Channel still lists it as "Lumen (old web version)", with its Guide and Quick Settings. Or:

1. Download `com.samrenfrew.lumen_2.2.1_all.ipk` from the [v2.2.1 release](https://github.com/samrenfrew/lumen-webos/releases/tag/v2.2.1), and the Guide and Quick Settings `.ipk` files if you want them. Leave out the screensaver one: its memory use made the TV restart.
2. Install them with [Dev Manager](https://github.com/webosbrew/dev-manager-desktop/releases) or `ares-install`.

The web Lumen runs in the TV's web engine, so it should work on more webOS TVs than Lumen Native. It was written for webOS 22 and later moved to webOS 25. Its source is in [app/](app/), and its README is in the [v2.2.1 tag](https://github.com/samrenfrew/lumen-webos/tree/v2.2.1#readme).

## Found a bug?

Please [open an issue](https://github.com/samrenfrew/lumen-webos/issues). Say:

- your TV model and webOS version,
- which Lumen and which version (the Homebrew Channel's app list shows it),
- what you did, what you expected and what happened.

The app's log helps most. It is at `/tmp/lumen-native-<uid>/log` on the TV until it restarts, and each launch also writes a line to `/var/lib/homehook/lumen.log`. Attach either one.

## Set up for development

Lumen Native lives in [native/lumen/](native/lumen/). It is one program, packaged four times: the home screen, Lumen Screensaver, Lumen Guide and Lumen Quick Settings.

You need:

- Docker, for the build container (`native/lumen/Dockerfile`, with the Rust version in `rust-toolchain.toml`).
- The webOS SDK from [openlgtv/buildroot-nc4](https://github.com/openlgtv/buildroot-nc4) (`arm-webos-linux-gnueabi_sdk-buildroot`), unpacked in `~/toolchains/` with `relocate-sdk.sh` run, or set `WEBOS_SDK`.
- Node 22 or later, for packaging and the TV tools.
- SSH access to the rooted TV.

Then:

1. Install the packages: `npm --prefix app install`.
2. Copy `.env.example` to `.env` and set `LUMEN_TV_HOST` to an SSH alias or `root@<tv-ip>`.
3. Put your SSH public key on the TV, so `ssh root@<tv-ip>` works with no password.
4. Optional: put the private icons, wallpapers and font in `app/private/`. The app works without them.

### Check work on your computer

| Command | What it does |
|---|---|
| `sh native/lumen/test.sh` | Runs the tests in the build container. |
| `sh native/lumen/host-shot.sh out.png [keys...]` | Draws a screen to a picture with software graphics, after pressing the keys given. |
| `sh native/lumen/shots.sh` | Draws every screen in `native/lumen/shots/list` and compares it with its kept picture. `--keep` takes them again. |

### Install on the TV

```sh
npm run deploy:native
```

This builds Lumen Native and installs it on the TV over SSH. Add `LUMEN_LAUNCH=1` to open it, `LUMEN_SAVER=1` to install Lumen Screensaver too, and `LUMEN_SIDE=1` to install Lumen Guide and Lumen Quick Settings. `sh tools/build_native.sh` builds the four `.ipk` files without installing them.

### Work on the live app

A dev build listens for commands on the TV. Open a tunnel, then use [tools/tv/nctl.mjs](tools/tv/nctl.mjs):

```sh
ssh -f -N -L 19997:127.0.0.1:9997 root@<tv-ip>
node tools/tv/nctl.mjs where            # what is in focus
node tools/tv/nctl.mjs key down ok      # press remote keys
node tools/tv/nctl.mjs set <name> <v>   # change a timing, size or glass value live
node tools/tv/nctl.mjs shot out.png     # save a picture of the screen
```

Values set live are kept until `nctl.mjs reset`. On dev builds, Settings > Appearance > Glass lab has sliders for every glass value.

## Contributing

Lumen Native is built for one TV and one person's taste.

- Bug reports are welcome, from any TV. Reports from other models are the most useful.
- Pull requests are accepted for bug fixes only.
- New features and design changes are not accepted. If you want something different, fork the project. The MIT licence lets you do this.

Commit subjects name the area, then say what changed for the user, in plain words: `Live TV: channel logos fill the tile, clear of the progress bar`. A push to `main` publishes a release to GitHub and the Homebrew Channel, so do not push only to get a build onto the TV.

## Research

The TV's graphics chip and memory shaped every choice in this project.

| Document | What it covers |
|---|---|
| [docs/plans/native-rewrite.md](docs/plans/native-rewrite.md) | The native rewrite: what it set out to do, its budgets, and where each part stands. |
| [docs/research/spikes.md](docs/research/spikes.md) | Short tests on the TV, one for each open question, including the ones that decided on Rust and native. |
| [docs/plans/native-saver.md](docs/plans/native-saver.md) | The native screensaver, and its measurements against the web one. |
| [docs/research/RESEARCH.md](docs/research/RESEARCH.md) | tvOS design rules, Liquid Glass, and the cost of each glass effect on this TV. |
| [docs/research/frame-rate.md](docs/research/frame-rate.md) | What limited the web Lumen's frame rate. |
| [docs/keep-alive.md](docs/keep-alive.md) | How apps and Lumen stay open when you change apps. |
| [docs/research/live-tv.md](docs/research/live-tv.md) | How to read the channel list, the logos and the guide, tune a channel, and show the live preview. |
| [docs/research/prior-art.md](docs/research/prior-art.md) | Other custom TV home screens and forks of Lumen, and what this project took from them. |

## Credits

Lumen was first made by [ammar027](https://github.com/ammar027/lumen-webos), on top of [partridgeworks/webos-launcher](https://github.com/partridgeworks/webos-launcher) (MIT).

Other projects gave ideas or code. Thank you to their authors.

| Project | What it gave Lumen |
|---|---|
| [codefarm-eu/lumen-webos](https://github.com/codefarm-eu/lumen-webos) (MIT) | Its "Lumen bar", a dock drawn over other apps, showed that an overlay app works. Lumen Guide grew from it. Also the idea that the screensaver goes back to the app it came up over. |
| [quangtdev/lumen-webos](https://github.com/quangtdev/lumen-webos) (MIT) | The wallpaper fade without the stall, and the idea of a night backlight. |
| [juanrusch/lumen-webos](https://github.com/juanrusch/lumen-webos) (MIT) | App names without the store tagline. |
| [rorygallagher2024/glasshouse](https://github.com/rorygallagher2024/glasshouse) (MIT) | The Home takeover in `tools/tv/home-setup.sh`, the Privacy agreement and host lists in `tools/tv/privacy.sh`, the live channel preview, and how to read Live TV's last channel. |
| [lasharor/cleanyourtv-WebOS](https://github.com/lasharor/cleanyourtv-WebOS) | More LG ad and tracking hosts for Settings > Privacy. |
| [rudra-avatar/Hearth](https://github.com/rudra-avatar/Hearth) | The first approach to Home and Power: LG's default Home app setting and the switch-on app. |
| [ecklf/lg-luma](https://github.com/ecklf/lg-luma) (MIT) | A second working example of the Home takeover on webOS 25. |
| [openlgtv/buildroot-nc4](https://github.com/openlgtv/buildroot-nc4) | The webOS SDK that builds Lumen Native. |
| [webOS Homebrew](https://www.webosbrew.org/) and [LG Input Hook](https://github.com/Simon34545/lginputhook) | Root access, app installs, and remote button binds. |

[docs/research/prior-art.md](docs/research/prior-art.md) has the full review of these projects. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for licences. This project is not affiliated with LG, Apple or any app vendor.

Licence: [MIT](LICENSE).
