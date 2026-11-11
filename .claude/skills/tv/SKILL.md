---
name: tv
description: Sam's TV over SSH from this Mac. Use to install a build on the TV, read lumen.log, run JS or CSS in the live Lumen page, take a TV screenshot, or develop live while Sam watches the TV.
---

# The TV over SSH

The TV is `root@192.168.0.20` (also `LUMEN_TV_HOST` in the gitignored `.env`). Key auth works from this Mac, so every call uses `-o BatchMode=yes`. Add `-o LogLevel=ERROR` to hide the post-quantum warning the TV prints on each connection. If ssh fails inside the sandbox, rerun it with the sandbox off.

## Consent

Hard rule (CLAUDE.md): a change Sam is to try goes into the live page first, with `css` or `eval`, and he judges it there. Install a build with `npm run deploy:home` only when he says it feels right or tells you to build. Never install a build per try. Do not ask Sam to update in the Homebrew Channel. If SSH fails (the TV is in standby or off the network), or Sam has said not to touch the TV, say the change waits for the TV. Key presses change what is on Sam's screen, so use them only in a task that needs them. Reading the log, `eval` of read-only expressions and screenshots need no ask.

## Install a build

1. Run `npm run deploy:home`. It type checks, builds Lumen as `com.samrenfrew.lumen` with the dev tools on, copies the `.ipk` to the TV, installs it and launches it. A final `"install has been disconnected."` after `"installed"` is normal.
2. Done is Lumen in front: `luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}'` (with `ssh -tt`) gives `com.samrenfrew.lumen`. It can take about 4 seconds. If ssh stops answering, the TV is in standby: say that Lumen starts on the new build at the next switch-on.

Plain `npm run deploy` uses the app id in `.env`, which is `com.samrenfrew.lumen.dev` ("Home Dev"). That installs a second app beside Lumen, and the Home button does not open it. Use it only when Sam asks for a side-by-side copy.

A push to `main` publishes a Channel release, and a Channel update replaces the local install. A branch push publishes nothing. Push only when Sam asks for it.

## The log

`/var/lib/homehook/lumen.log` is on the TV, and only ssh can read it. The app cannot read it. Use `grep ' boot ' … | tail -3`, `grep guide … | tail`, or `grep ' nav ' …`. `docs/research/startup-perf.md` explains the boot line.

## DevTools: eval, CSS, screenshots, keys

WebAppMgr serves Chrome DevTools on the TV's `127.0.0.1:9998`. Open the tunnel once a session:

```sh
curl -s -m 3 http://127.0.0.1:19998/json >/dev/null || ssh -o BatchMode=yes -o LogLevel=ERROR -o ExitOnForwardFailure=yes -f -N -L 19998:127.0.0.1:9998 root@192.168.0.20
```

If the tunnel goes stale, run `pkill -f 19998:127.0.0.1:9998` and open it again. Then use `node tools/tv/cdp.mjs`:

- `cdp.mjs` talks to Lumen's own page. Set `LUMEN_CDP_PAGE=guide` or `LUMEN_CDP_PAGE=quick` to talk to an overlay app's page instead.
- First, find out where Sam is: `eval 'JSON.stringify(lumen.where())'`. It gives the screen (`home`, `settings`, `guide`, `full guide`, `ambient`), whether Control Center is open, and each focused part with its label, its index and the row it sits in. The last entry is what OK presses. Read it before and after a key press instead of guessing from a screenshot. It exists on dev builds only (`npm run deploy:home`); if `lumen` is undefined, the TV runs a Channel release.
- `eval '<expr>'` runs in the live page. Wrap objects in `JSON.stringify(...)`. Use it to check a **(verify)** item from page state (classes, `getComputedStyle`, `getBoundingClientRect`) before you ask Sam to look.
- `css '<css>'` sets a `<style id="live">` block, and `css ''` clears it.
- `shot <file.png> [x,y,w,h]` saves a 1920x1080 PNG of about 2 MB. Save it in the scratchpad, and read it yourself before you send it.
- `key down down enter 500 back` presses remote keys, and a number waits that many ms.

Quirks:

- The video wallpaper plays on a hardware layer under the page, so a screenshot shows black where the video is. Reading the video's pixels returns black too. Still wallpapers and Immich photos capture normally.
- A key press wakes the screensaver. Some screens save state: for example, opening the Full guide makes the Guide open there next time. Put things back as they were (Back to home, the same guide mode) after a key-driven screenshot.
- The page id changes at every relaunch. `cdp.mjs` looks it up each time.
- Anything set over DevTools is gone after a relaunch or an install. Interacting with the page can also undo an injected effect.
- The TV is wired only (192.168.0.20 on eth0). Sam switched its Wi-Fi off on 2026-10-31 after a day when both were up on the same subnet and the TV's TLS connections to the PC hung. If `ip -4 addr` shows wlan0 again, that is the first suspect for odd network results.
- Keep file searches on the TV narrow (`-maxdepth`, a few directories). A `find /` on 2026-10-31 ran for minutes and the TV rebooted during it.

## Live development with Sam

Use this for a look-and-feel change (spacing, size, position, colour), where one round trip through a dev build is slow.

1. This is the default for any change Sam is to try, not only look-and-feel. If Sam cannot see the TV now, make the change in source and wait for him; do not build.
2. Inject the change with `css` (or `eval` for a DOM or state change). Say in one line what to look at, and wait for his reply. Make one change per round.
3. When Sam says it looks wrong ("things moved", "can you see?"), take a `shot` and measure with `eval`. Do not guess. For example, an absolute label once collapsed the tiles from 212px to 144px, and `getBoundingClientRect` showed it at once.
4. A heavy effect can look right and still drop the frame rate. Sam judges smoothness on the TV, and the TV's graphics chip is weak (`docs/research/RESEARCH.md` section 4).
5. When Sam says it is right, write the final values into source (the matching file in `app/src/theme/`, a token in `tokens.css` or `themes.css` where the value is one, or the component), and install it with `npm run deploy:home`. Then clear the live block. The injected CSS is never the record.
