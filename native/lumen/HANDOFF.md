# Handoff: Lumen Native, phases 2 to 6 (written 2026-11-24, afternoon)

Sam's instruction (2026-11-24): keep building the whole app with sensible default choices; come back only with "The build is done,
it's installed on the TV". Performance and budgets are tuned after that, together. Branch `native`. Delete this file when done.

## Done and merged on `native`

- Phase 1 written up in `docs/plans/native-rewrite.md`. Keep-alive lists on the TV include `com.samrenfrew.lumen.native` (rebooted).
- Home: rest state (first key wakes focus), scrolling grid, app names, top buttons, weather line, small forecast, picture caption,
  hold OK to move, menu (uninstall / remove / folders), folders, toasts, number keys, Magic Remote pointer, night dim, wallpaper
  fade, built-in tiles, launch views `recent` and `controls`. Settings edits are written back to `/var/lib/homehook/lumen-config.json`.
- Control Center and Inputs (helper, merged). Installed and checked on the TV 15:12: real backlight and modes read.
- Live TV data layer `src/livetv.rs` (helper, merged; not yet run on the TV).
- Tools: `tools/tv/nctl.mjs` (control channel, tunnel port 19997), `native/lumen/shots.sh` (+ `shots/list`), `host-shot.sh` keys
  include `hold` and digits. `npm run deploy:native` installs (allowed now without asking each time).

## Still running when this was written (helpers in git worktrees under `.claude/worktrees/`)

Each commits on its own branch `worktree-agent-<id>`. To merge: `git merge --no-edit <branch>`; conflicts so far were always
"keep both sides" in `main.rs` and `root.rs` (watch for a lost closing `}` where both sides ended a function). Then
`sh native/lumen/test.sh`, `sh native/lumen/shots.sh`, `git worktree remove --force <path>`.

- Settings screen (`src/settings_screen.rs`).
- Live TV screens: row on home, Guide, Full guide, typed numbers, `{"view":"guide"}` (`src/guide.rs`, `home.rs`).
- Screensaver as a mode of the binary, `{"view":"ambient"}`, the saver app id, Immich picker (`src/saver.rs`).

If a helper stopped half way, its branch still has its commits: `git branch | grep worktree-agent`, read its last commit.

## Then

1. Merge all, test, shots, `LUMEN_LAUNCH=1 npm run deploy:native`, check each screen on the TV with `nctl.mjs` (`where`, `shot`).
2. Not yet decided with Sam: point the Home button at Lumen Native (`tools/tv/home-setup.sh`), and packaging the saver/guide side apps.
3. Tell Sam: "The build is done, it's installed on the TV", with the known gaps (grep `ponytail:` in `native/lumen/src`) and the
   measured budget misses in `native-rewrite.md` phase 1.
