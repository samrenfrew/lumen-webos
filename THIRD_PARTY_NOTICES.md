# Third-party notices

This repository holds the launcher's source (`app/`), build tools, notes and TV-side scripts. The material below belongs to others.

| Material | Licence | Used for |
|---|---|---|
| [partridgeworks/webos-launcher](https://github.com/partridgeworks/webos-launcher) | MIT, © webos-launcher contributors | the base of `app/`: forked at upstream commit `1e2c39b9a988ceb1b258e40e3592bda84a2aa076` (2026-09-23), with this project's changes committed on top. Upstream's own licence and notices are kept in `app/LICENSE` and `app/THIRD_PARTY_NOTICES.md` |
| [rorygallagher2024/glasshouse](https://github.com/rorygallagher2024/glasshouse) | MIT, © 2026 Rory Gallagher | methods adapted into `tools/tv/home-setup.sh` (the Home takeover) and `tools/tv/privacy.sh` (agreement flags and the ads and telemetry host list) |
| [lasharor/cleanyourtv-WebOS](https://github.com/lasharor/cleanyourtv-WebOS) | no licence stated | host names added to the block list in `tools/tv/privacy.sh` |
| Forks of Lumen: [codefarm-eu](https://github.com/codefarm-eu/lumen-webos), [quangtdev](https://github.com/quangtdev/lumen-webos), [juanrusch](https://github.com/juanrusch/lumen-webos) | MIT | ideas and small changes taken into `app/` and `tools/tv/` (overlay app, wallpaper fade, app names); see `docs/research/prior-art.md` |
| [Geist](https://github.com/vercel/geist-font) typeface | SIL Open Font License 1.1, © The Geist Project Authors | `app/src/assets/geist-wght.woff2` (and `tools/branding/`) |
| [Preact](https://preactjs.com) | MIT, © Jason Miller | the UI library (comes with the upstream clone) |
| [Simple Icons](https://simpleicons.org) glyph paths | CC0 | `tools/glyphs.json`, used to draw some app tiles |
| Wallpaper catalogue (`tools/catalog.json`) | links and titles only | points to pictures on Reddit's image host and to their posts; the pictures belong to their authors and are loaded from the internet, not stored here |
| Video wallpaper (`tools/wallpaperVideos.ts`) | links only | streams [Mixkit](https://mixkit.co/license/#videoFree) free stock videos (free use, no attribution needed) and Apple's tvOS aerial screensaver videos from Apple's own server; nothing is stored here or on the TV, and the videos belong to their makers |
| Weather | [Open-Meteo](https://open-meteo.com) (CC BY 4.0), location from [ipwho.is](https://ipwho.is) | optional date/weather line |

App icons, logos and names of third-party apps are trademarks of their owners. They are not included in this repository and this project is not
affiliated with or endorsed by them, or by LG Electronics, Apple Inc. or Samsung. "LG" and "webOS" are trademarks of LG Electronics.
