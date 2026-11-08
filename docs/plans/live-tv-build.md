# Live TV row and guide: build plan

Written 2026-10-17 at the end of the Live TV spike. Both phases are built and released; this file now records how they work. The spike trail (every call tried, what failed and why)
is in `docs/research/live-tv.md`; this file only keeps what the build needs.

## What we are building

**Phase 1**
1. A **Live TV row** in the view under the dock (press Down from the glass bar): a titled row of channel tiles between the glass bar and the
   app grid. Each tile shows the channel's logo, its number and a thin bar for how far through the current programme is. The focused tile
   shows the channel name, what's on now (title, times, time left) and what's next underneath it, like an app tile's label. OK switches the
   TV to that channel. The row ends with a **Guide** tile.
2. A **Guide screen, Now and Next**: one row per channel (number, logo, now with times and progress, next with its start time). Up/Down
   moves, OK switches to the channel, Back returns to Home. Only two listings per channel are needed, not the full schedule.

**Phase 2 (later)**
3. **Full guide**: a time grid (two hours across, channels down, a line at now). Since 2026-11-08 a rail at the left edge
   (`components/Rail.tsx`, shared with Lumen Guide) switches between Now, Next and the Full guide. It replaced the switch at the top right.

**Not doing**
- Anything drawn over Live TV from inside Lumen. Live TV is a separate app, and Lumen is not on screen while it plays. A separate
  overlay app, Lumen Guide, does this instead (`docs/plans/guide-overlay.md`).
- There is no Recent row any more (every app is in the grid), so the Live TV row is the only titled row.

## Status

**2026-10-17, phase 1 built** (since released):
- The Live TV row is the **last row, under the app grid** (Sam's call: not between the glass bar and the grid as in the mock). First 5
  channels by number, radio, data and skipped channels left out, with the Guide tile first (Sam's call, 2026-10-17; the mock has it last). Settings > Options > "Live TV row" (on by default).
- Channel tiles follow the tvOS suggestions rather than the mock: every logo on the same dark tile, no number on the tile (it is in the
  now/next block), progress as an inset capsule.
- The now/next spike: `programPF:4` works, two rows per channel (`isPresent` true for now). `startTime`/`endTime` are UTC
  ("2026,10,17,17,05,00"); Lumen uses those and works out now/next from the times. Rows carry `component`, `rating` and `description`
  blocks (about 1.5 KB each), so `nowNext()` asks for five fields with db8's `select`, and asks again for whole rows if the TV refuses that.
  Checked on the TV: with `select` and `limit: 500` the reply is 422 rows in one page (no `next`), 104 KB. Channels that are off air
  carry a placeholder as their current programme ("Programmes start at midnight", `isPresent: true`), shown as is.
  The one-channel filter was skipped: the row and the Guide need every channel anyway.
- Code: `lib/root.ts` (`channelList`, `nowNext`, `openChannel`), `lib/livetv.ts`, `hooks/useLiveTv.ts`, `components/ChannelTile.tsx`,
  `screens/Guide.tsx`. In the browser (dev) the row and the Guide show made-up channels.
- Channels: picked in Lumen's Settings > Channels (Sam's call, not the TV's Favourites list), ticked and re-ordered like Apps; until some
  are picked the row shows the first 5 by number. The row scrolls on so the focused channel's now/next block always fits on its right
  (Row's `slot`), and runs off the screen edges when scrolled.
- The Guide takes typed numbers: each digit moves the list (1, then 10, then 101), the digits show at the top and clear 2 s after the last.
- Start-up: Live TV waits for the wallpaper (no cache read, no row, no TV calls before it). Dev builds write a boot timeline to
  /var/lib/homehook/lumen.log 20 s after launch: `boot script 40, frame 310, ..., getChannelList 2900-4100, ...` (stage times and every
  root call's start-end, ms since the page started).

**2026-10-17, phase 2 built** (since released):
- The view is kept in the settings (`guideFull`), so the Guide opens on whichever was used last. The first build had a switch at the
  top right; since 2026-11-08 the rail at the left edge does this (Left from the list opens it, Up/Down and OK pick a view).
- The Full guide follows `4-full-guide.jpg`: the focused programme's details on top (channel, title, times, progress, S and AD), then
  two hours across in half-hour columns, seven channels down, a line at now. Up/Down keep the time the focus is at; Left/Right move a
  programme at a time and scroll the grid. It goes back as far as the half hour now is in, and on as far as the schedule was read (8 hours).
- One guide service read feeds the row, Now and Next and the Full guide, 8 hours ahead (`useLiveTv`). At start-up it asks only for the
  row's channels and the first 10 in the list (two calls). When the Guide opens, in either mode, it asks for the rest, nearest the focus
  first. A channel's read lasts 30 minutes. A channel still waiting shows "Loading…". The TV's now and next table is still read, as one
  quick call, and wins where both have a programme. The cache keeps the first read's next 4 hours, so a cold start draws from it.
- If the guide service refuses while the Guide is open, the schedule is read from `programSCH:4` with six fields (`root.ts` `schedule()`).
  Seen on the TV 2026-10-17: the kind has no index on `startTime` or `endTime`, so db8 refuses any time filter ("no index for query",
  -3965). Lumen reads every page with no filter and keeps what overlaps the next 8 hours. The kind holds about a week: over 20,000 rows,
  more than 40 pages, at about 155 ms a page. The grid shows Now and Next's listings at first and fills in every 3 pages. Each read writes
  a `guide` line to `lumen.log` with pages, rows and time.
- The schedule kind does not hold the programmes on now and next, so Now and Next's rows always stay in the grid and the schedule's
  rows fill in around them (`mergeListings`). **(verify)** that the current slot is filled on every channel.
- The programme description shows under the focused programme's time in the Full guide, two lines at most. It comes from the guide
  service (`getEventList` replies carry `description`), with Freeview's trailing flags such as "[S]" taken off. The schedule-table fallback
  does not select it, since reading it for every row is heavy, so then no blurb shows. Blurbs are left out of the Live TV cache.

## Mockups

In `docs/mockups/live-tv/` (1920×1080):

| File | What it shows |
|---|---|
| `0-current-home.jpg` | The current build's home, rendered 2026-10-17 (big clock, three round buttons, glass bar at the bottom). |
| `1-current-down.jpg` | The current build after Down: glass bar at the top over the blurred wallpaper, app grid below (in the browser only the Inputs tile shows; on the TV the other apps are there). |
| `2-live-tv-row.jpg` | Phase 1: the Live TV row between the glass bar and the app grid, BBC TWO focused, now/next under it, Guide tile at the end. |
| `3-now-and-next.jpg` | Phase 1: the Guide screen, Now and Next. The switcher at the top right is for phase 2; in phase 1 it can be left out. |
| `4-full-guide.jpg` | Phase 2: the full grid with the switcher on Full guide. |

Editable versions: the private design canvas "Lumen Live TV mocks", https://claude.ai/artifact/NrpUz3chDf7rxs1BdhHaD6 (readable with the
Artifact tool from the same account). In the mocks the logos are text stand-ins and most programme names are made up; the real build uses
the logo PNGs and guide data from the TV.

Look in the mocks: same glass, rim lights, focus scale (1.175 on grid tiles) and tile size as the existing rows, Geist, white text on the
blurred wallpaper. The Guide screen uses the blurred wallpaper with a dark veil (like Settings), rows as faint glass, the focused row as the
white capsule with dark text.

## The calls (all proven on the TV as root, 2026-10-17)

| Need | Call | Notes |
|---|---|---|
| Channel list | `luna://com.webos.service.iepg/getChannelList` `{"dataType":0,"sort":0,"channelMode":"Terrestrial","attribute":[{"filterType":"Invisible","value":false}]}` | 188 channels. Fields used: `channelId`, `channelNumber`, `channelName`, `signalChannelId`, `imgUrl`, `Radio`, `Data`, `skipped`, `groupIdList`. Live TV's edit screen passes `"requestField":["channelId"]`; a longer `requestField` list should shrink the reply a lot (untested with several fields). |
| Logos | read the file at `imgUrl` | PNGs, 2 to 3 KB, under `/tmp/gibbips/logo/`. 176 of 188 channels have one. `/tmp` is cleared on reboot and the TV refetches them, probably when Live TV runs; cache them in Lumen and fall back to a text tile. `readIcons()` in `lib/root.ts` already returns files as base64. |
| Now and next | `luna://com.webos.epgdb/find` on `com.webos.service.legacybroadcast.programPF:4` | See the spike above. Join to channels on `signalChannelId`. |
| Full schedule (phase 2) | same, on `com.webos.service.legacybroadcast.programSCH:4` | Read works (`limit:1` returned a programme). No index on the times, so no time filter: read every page and keep the window. |
| Guide service (used first) | `luna://com.webos.service.iepg/getEventList` `{"bFVPChannelId":true,"channelId":[up to 10 ids],"dataType":0,"fepgType":"Forward","localStartTime":"2026,10,18,06,30,00","localEndTime":...,"sort":0}` | What LG's Guide reads (`docs/research/live-tv.md` Step 2d). Takes a time window and ten channels a call. The Full guide reads the rows in view first, then the rest; the schedule table above is the fallback. The row's channels and the first 10 at start-up, every channel when the Guide opens. |
| Switch channel | `luna://com.webos.service.apiadapter/tv/openChannel` `{"channelId":"1_41_104_104_16522_17664_9018"}` | Opens Live TV on that channel, also from Home. Not tried without root, or with Live TV already open on another channel. |

Don't use: `utp.broadcast/changeChannel` (needs Live TV's broadcast id, which changes every time Live TV starts), launching Live TV with a
`channelId` parameter (ignored), `getChannelZappingList` (never answers luna-send).

Ids: `channelId` looks like `1_41_2_102_16522_17472_9018` (seven numbers); `signalChannelId` is the last three (`16522_17472_9018`) and is
what the guide rows carry.

## How it fits the code

- **Root calls.** `lib/root.ts` is the only place a shell command is built, one fixed command per export, every value checked first. Add
  `channelList()`, `nowNext()` (and later `schedule(from, to)`), `openChannel(channelId)`. Check `channelId` against
  `/^\d+(_\d+){6}$/` and `signalChannelId` against `/^\d+_\d+_\d+$/` before they go anywhere near the shell. Logo paths go through
  `readIcons()` (it already refuses anything that is not a plain absolute path).
- **New `lib/livetv.ts`.** Parses the list (drop `Radio`, `Data` and `skipped` channels), keeps channels and logos in the same cache as
  the app list so a cold start draws the row at once (the TV ends Lumen whenever another app opens, see `lib/boot.ts`), then refreshes in
  the background. Now/next: progress and "min left" are worked out in the app from the times; read the guide again when the focused
  channel's programme ends or the Guide screen opens, not on a timer.
- **The row.** `HomeRow` in `screens/Home.tsx` already has `titled` (it was for the Recent row). Add the Live TV row in `app.tsx`'s
  `rows` (around line 291) right after the glass bar's row, only when the setting is on and there are channels. Channel tiles need their
  own tile look (logo on a dark tile, number top left, progress bar along the bottom); the focused tile's label becomes the now/next block.
- **The Guide screen.** A new screen beside `screens/Home.tsx` and `screens/Settings.tsx`, opened by the Guide tile, closed with Back.
  Reuse the row/focus handling rather than writing new key handling.
- **Settings.** A "Live TV row" switch, and the channel choice (see Open questions).
- Keep to the performance rules in `docs/research/RESEARCH.md` section 4: no new live blur, only `transform` animates, the Guide list draws
  only the rows near the focused one.
