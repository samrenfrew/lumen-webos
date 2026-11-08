# Lumen

A tvOS-style launcher for a rooted LG webOS TV. `app/` is the Preact app (a fork of partridgeworks/webos-launcher, never sent upstream). `tools/` holds the build scripts and the TV-side scripts. `README.md` covers build and install.

## The TV

The target TV is an LG 43UT91006LA on webOS 25 (Chromium 120). The repo was first written for a webOS 22 TV, so any rule tuned for that TV is an assumption until it is seen working on this one. `docs/plans/PLAN.md` holds the roadmap and the **Verify on the TV** list; read its "Where things stand" section before starting work.

- Code that needs a TV answer is written defensively (harmless if the answer is no) and marked **(verify)** in `docs/plans/PLAN.md` until Sam has seen it work.
- The build still targets `chrome79` and a classic script, not a module (`app/vite.config.ts`). Keep it that way unless Sam decides otherwise.
- The TV's graphics chip is weak. Animate only `transform` and `opacity`; glass is a light live blur on the focused control only, with a plain tint and rims elsewhere; the full-screen backdrops are a blurred copy of the wallpaper, not live blur. The reasons are in `docs/research/RESEARCH.md` section 4.
- Numbers use the font's normal, proportional digits. Never set `font-variant-numeric: tabular-nums` or `'tnum'` on text, because it spaces digits too far apart (the 1s in "11°"). The one exception is the clock time, so it does not jitter as it changes.
- The stylesheet is split by part in `app/src/theme/`, imported in order in `main.tsx`. Colours, glass, shapes and timings are tokens in `tokens.css`; the light look and the focus strength only give them new values in `themes.css`. To change a look, edit its one rule or token. Never add an override rule further down. To prove a CSS refactor changes nothing, run `tools/style-snapshot.mjs` on a dev build before and after, then `tools/style-diff.mjs`.
- Mark the focused part of any new screen or panel with a `<part>--focused` class (`tile--focused`, `slist__row--focused`). On dev builds, `lumen.where()` (`app/src/lib/where.ts`) finds focus from that class, so an agent on DevTools can tell where Sam is. The `tv` skill says how to read it.
- Every root command goes through `app/src/lib/root.ts`, one fixed command per export, with each value checked against a list or pattern. Add new root commands there, in the same shape.

## Getting a build onto the TV

**Hard rule: DevTools first, build last.** Put every change Sam is to try into the live page over DevTools (`cdp.mjs css` or `eval`, the `tv` skill), and let him judge it there. That includes JS changes such as a duration: patch them live with `eval`. Write the change into source as you go. Run `npm run deploy:home` only when Sam says it feels right, or tells you to build. Never install a build per try. If a change truly cannot be made live, say so and ask before building.

A dev build is a local build that you install on the TV over SSH: `npm run deploy:home`. Do not push a branch only to get a build onto the TV, and do not ask Sam to update in the Homebrew Channel.

A push to `main` publishes a stable release to GitHub and the Homebrew Channel (`.github/workflows/release.yml`). The version is the one in `app/public/appinfo.json`, or the next free patch above it. A push to any other branch only builds a dev `.ipk` as a workflow artifact and publishes nothing. Push only when Sam asks. A Channel update replaces a local install.

Every push to `main` makes a release, docs-only pushes included, so each one needs release notes. When the release run has passed, add the notes above the text the workflow wrote (the build commit, install line and SHA-256), then a `---` line. Write them for someone who uses the TV: one line per change, in plain words, taken from the commit subjects since the last release. Use `gh release edit v<version> --repo samrenfrew/lumen-webos --notes-file <file>`. Always pass `--repo`: without it, `gh` reads the `upstream` remote (ammar027's repo), not this one. Show Sam the notes before you publish them.

This Mac has SSH and DevTools access to the TV. With it you can install a build, read the log, run JS or CSS in the live page, take screenshots, or develop live while Sam watches. The `tv` skill (`.claude/skills/tv/SKILL.md`) has the steps and quirks.

Dev builds write a "boot" timeline line to `/var/lib/homehook/lumen.log` after each launch. `docs/research/startup-perf.md` explains how to read it.

## Checking work locally

- Type check and build: `npm --prefix app run build`.
- Preview: build, then start the `app-dist` entry in `.claude/launch.json` (port 5198). The `app` entry (Vite) fails on this Mac with `EPERM uv_cwd`, because macOS does not let the preview start Node in Documents. There is no test suite; the preview and the TV are the checks.
- `app/private/` (icons, wallpapers, font) is gitignored and may be absent. The app must still work without it, since the public release is built that way.

## Docs

Plans and specs live in `docs/plans/` (`PLAN.md`, `live-tv-build.md`, `settings-redesign.md`, `guide-overlay.md`). Research, audits, spikes and measurements live in `docs/research/` (`RESEARCH.md`, `live-tv.md`, `startup-perf.md`, `frame-rate.md`, `spikes.md`, `design-audit.md`, `prior-art.md`, `down-up-lag-review.md`). Docs that describe how a feature works now stay in `docs/` (`immich.md`, `keep-alive.md`). Do not put a plan, spec or finding in `.scratch/`: it is deleted when the work ships. Read the one for the area you touch before changing it, and update it when a finding changes.

## Commits

Subject lines name the area, then say what changed for the user, in plain words: `Live TV: channel logos fill the tile, clear of the progress bar`. Docs-only commits start with `docs/<file>:`.
