# Lumen Quick Settings: Lumen's Control Center over any app

Agreed with Sam on 2026-11-10: a separate app, and in Lumen the gear button opens Lumen's own Control Center. The layout, the look and
the items marked **(default)** were picked by Claude for the first build; Sam reviews them on the TV.

## Why it can work

The same way Lumen Guide does (`guide-overlay.md`): a second app in an overlay window, kept running hidden, opened by a key bound with
LG Input Hook. The gear button is code 139 for Input Hook, and a bind stops LG's own gear menu (`docs/research/spikes.md`, "Gear menu").

## What it does

- **A separate app**, "Lumen Quick Settings", id `com.samrenfrew.lumen.quick`, hidden from LG's app list.
- **The gear button, by the app in front at the press.** While the app is installed, `tools/tv/watch.sh` (`gear_bind`) binds the
  gear to `tools/tv/gear.sh`. The script asks the TV which app is in front (about 30 ms). With Lumen in front it launches Lumen with
  `{"view":"controls"}`, which opens Control Center on home, or closes it. With any other app it launches Lumen Quick Settings.
  Without the app installed, the gear is left to the TV everywhere.
- **Kept started hidden**, in Lumen too: `watch.sh` (`quick_start`) starts it hidden when it is not open, as the app in front
  changes, when the scripts start and 8 s after each wake. A gear press then shows it in about 0.2 s.
- **The controls** are Lumen's Control Center (`components/ControlCenter.tsx`, which now takes its layout as a prop) **(default)**:
  Sound and Picture; the backlight bar and Eye Comfort; All settings (LG's Settings app), Screensaver (Lumen's), Screen off and Restart.
  Theme and Clock are left out, since they are Lumen's own settings and a second app cannot see them.
- **Closes** on Back, on the gear again, after an action that leaves it, and after 30 seconds without a key **(default)**. It hides
  rather than closes, as the guide does.

## How it looks (defaults)

- At the top right, where Lumen's Control Center is. No blur, since an overlay cannot see the app under it: the controls have a darker
  tint (`--glass-tint` and `--cc-focus-fill` get new values in `theme/lquick.css`), and a soft dark corner sits behind them.
- The panel fades and grows in from the top right corner. The lists (Sound, Picture) open as they do in Lumen.

## How it is built

- `app/quick.html` and `app/src/quick/main.tsx`, styles in `app/src/theme/lquick.css`, appinfo in `app/public-quick/`.
  `npm --prefix app run build:quick` builds it into `app/dist-quick/` (`LUMEN_TARGET=quick` in `vite.config.ts`).
  `npm run deploy:quick` installs it over SSH, and `npm run deploy:home` installs it after Lumen and the guide.
- Preview on the Mac: the `quick-dist` entry in `.claude/launch.json` (port 5197), at `/quick.html`.

## Status

First build installed on 2026-11-10 (dev build), with the new `watch.sh` on the TV.

- **Seen working:** with Lumen in front, a launch with `{"view":"controls"}` opened Control Center and a second one closed it
  (sent over SSH, read with `lumen.where()`). Sam saw the panel over Live TV and said it looks great. Lumen's own Control Center has the same rows as before the layout change.
- **(verify)** On the TV: the gear in Lumen opens and closes Control Center. In another app (Plex, Live TV) the gear shows Quick
  Settings over the picture, quickly; Back and the gear close it and the app has the remote again. Sound, Picture, the backlight and
  Eye Comfort change the app underneath, and show its values, not Lumen's. All settings opens LG's Settings. The look over a bright
  picture.
- **Release (2026-11-11):** `tools/build_release.py` builds it and the workflow lists it in the Homebrew Channel beside Lumen, from
  1.0.0, with a version of its own (`docs/plans/side-apps.md`). Its Screensaver button opens Lumen Screensaver over the app when that
  is installed, else Lumen's own.
- **The first gear press opened Lumen (fixed 2026-11-10):** the first build rewrote the bind as the app in front changed, right
  after the guide's own rewrite of the same file. Input Hook kept the old bind, so 3.5 s into Live TV the gear still launched Lumen.
  The bind is now fixed and `gear.sh` decides at the press. Seen: the script opens and closes Control Center with Lumen in front.
- **Warm and cold opens (measured 2026-11-10, launch to the window showing):** 0.2 s when it is open and hidden; 3 to 4 s cold.
  In a cold open the panel has its first frame 1.0 to 1.6 s after the page starts, and the TV shows the window about 2 s after
  that. Calling `webOSSystem.stageReady()` at the start did not shorten it (4.1 and 3.3 s), so it was taken out again.
- **The TV closed it while hidden:** a few seconds after each hide, so most presses were cold. It was added to
  `com.webos.memorymanager.KeepAlive` on 2026-11-10 (`setConfigs`), which the memory manager reads only at boot.
  After the reboot at 17:13 the same day it held: started hidden once at boot, it was still open 31 minutes later with Lumen in
  front, and the first launch then showed the window in 0.53 s. **(verify)** it also stays open under other apps (Plex, Live TV).
- **A hidden start holds the remote for about 2 s:** the window must be up once before it can hide (as the guide's), and keys go to
  it meanwhile. Until the reboot this happens at each change of app, since the TV keeps closing it.
- **Open:** Game mode, where the gear opens LG's Game Dashboard, is not handled. On an HDR picture the TV has other picture mode
  names, which the Picture list does not offer.
