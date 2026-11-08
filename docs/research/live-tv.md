# Live TV channel list: spike done

The build plan that came out of this, with mockups, is `docs/plans/live-tv-build.md`.

Goal: read the TV's aerial (Freeview, DVB-T, digital only) channel list, so Lumen can show channel tiles and later a guide
(docs/plans/PLAN.md, Spikes and Later item 9). The TV has been scanned and Live TV shows channels.

All commands so far are read-only and run as root over SSH on the TV.

## What we know (2026-10-17)

- EPG-related services on the TV include `com.webos.app.universalepg.nav`, `com.webos.epgdb`, `com.webos.service.iepg` and several
  `com.webos.service.broadcast-*` services (from `ls /usr/share/luna-service2/services.d /usr/share/dbus-1/system-services`).
- `com.webos.epgdb` is a db8 instance (`find`, `get`, `dump`, `getProfile`, ...). Not queried yet: we don't know its kind names.
- `luna://com.webos.service.iepg/getChannelGroupList '{}'` works: groups All (0), DTV (1), Radio (2), Favourites A to E (200 to 204).
- `luna://com.webos.service.iepg/getChannelList` always returns `"channelListCount": 0` and no error, whatever we pass:
  - no parameters (reply echoes `dataType: 3`, `dataSource: 2`, `ipAntennaSupport: true`, `ipChanInteractiveUrl` on `cdplauncher.lgtvcommon.com`);
  - `dataSource` 0 to 3 against `dataType` 0 to 4;
  - `channelGroup`, `channelGroupId`, `channelGroupName`, `groupId` set to All/DTV or 0/1.
- Working guess, not confirmed: `iepg` is LG's internet channels (LG Channels), not the aerial list. The `ip*Support` fields and the logo
  server URL point that way.
- `ls-monitor` while flipping channels in Live TV: the first filter caught only image paths from LG's Always Ready and Gallery apps;
  the second filter (image paths removed) printed nothing. So either Live TV's calls don't go over the bus in a form `ls-monitor` shows,
  or the filter is wrong. Unknown which.

- **Step 1 result (2026-10-17): `ls-monitor` works and found the guide data.** With Live TV in front, `com.webos.service.legacy-broadcast`
  (shown truncated as `legacy-broadcas.db`) writes programmes into `com.webos.epgdb` with `//put` and removes old ones with `//del`.
  Kind: `com.webos.service.legacybroadcast.programSCH:4`. Each programme has `programName`, `description`, `genre`, `startTime`/`endTime`
  (UTC) and `localStartTime`/`localEndTime` as `"2026,10,17,16,40,00"`, `duration` in seconds, `isAD`, `isSubtitle`, CRIDs, and
  `signalChannelId` like `"16522_18176_9018"`. Guess: that is service id, transport stream id, original network id (9018 is the UK
  Freeview network id). Programme rows do not carry the channel name or number, so the channel list is still to find, probably another
  kind in `epgdb` or a method on `legacy-broadcast`.
- **Step 1b (2026-10-17): reading `epgdb` as root works.** `luna-send -n 1 -f luna://com.webos.epgdb/find
  '{"query":{"from":"com.webos.service.legacybroadcast.programSCH:4","limit":1}}'` returns a programme. The capture has only two guide
  kinds, `programSCH:4` (schedule, about 11,900 writes) and `programPF:4` (present/following, about 290). No channel kind went into
  `epgdb`. Busiest TV services in the capture: `legacybroadcast.program`, `utp.broadcast`, `app.livetv`, `tvrmclient.tv`,
  `legacybroadcast.broadcast`. The channel list most likely comes from `utp.broadcast` or `legacybroadcast.broadcast`. Next: list the
  methods Live TV calls on those two, and look for `channelName`/`channelNumber` in any reply.
- **Step 1c (2026-10-17): channel names found on the bus.** Methods seen: `utp.broadcast` calls `legacybroadcast.broadcast` with
  `getCurrentChannel`, `changeChannel`, `getChannelState`, `getBroadcastComponentList`, `setBroadcastComponent`, and calls a separate
  service `com.webos.service.legacybroadcast.channel` (only `/ci/getSasMessage` seen in this capture). Replies carry
  `channelName` ("BBC ONE Yks HD", "BBC TWO HD", "ITV1 HD", "Channel 4 HD"), `channelNumber` ("101"...) and two channel id forms:
  `"1_41_101_101_16522_17546_9018"` (guess: type, ?, major, minor, service id, transport stream id, network id) and the short
  `"16522_17546_9018"` that matches the guide's `signalChannelId`. Only the channels flipped to appear, so this is the current-channel
  reply, not a list. Next: try `getChannelList` on `legacybroadcast.channel`, `utp.broadcast` and `legacybroadcast.broadcast`, and
  capture the `changeChannel` payload for tuning from Lumen.
- **Step 1d (2026-10-17):** `getChannelList` is `Unknown method` on all three (`legacybroadcast.channel`, `utp.broadcast`,
  `legacybroadcast.broadcast`). Next: ask each service for its method list (luna-service2 introspection, and the api-permissions
  files). The `changeChannel` grep printed nothing, probably because `[^»]` breaks on the multibyte `«` in busybox grep; use a plain
  grep with `cut` instead.
- **Step 1e (2026-10-17): method list for `com.webos.service.legacybroadcast.channel`** (introspection `{"type":"flat"}` on
  `/com/palm/luna/private/introspection`, first 40 shown). Read-only candidates for the list: `getPrimaryChannelList`,
  `getChannelZappingList`, `getChannelGroup`, `getChannel`, `getCurrentChannelId`, `getLastChannelId`, `getChannelGroupList`,
  `getChannelTypeList`, `getChannelModeList`, `getDirectTuneChannel`. **Never call** the writing ones: `clearChannelMap`, `deleteChannel`,
  `saveChannelList`, `updateChannelMap`, `revertChannelMap`, `importChannelMap`, `moveChannel`, `setChannel*`, `changeChannelNumber`,
  `changeChannelType`, `sortAnalogChannelList`, `*ForAstra`; they can wipe or reorder the tuned channels.
  Tuning: Live TV calls `luna://com.webos.service.utp.broadcast/changeChannelUp`; `utp.broadcast` then calls
  `legacybroadcast.broadcast/changeChannel` with `{"broadcastId":..., "channelId":"1_41_102_102_16522_17472_9018",
  "currentChannelId":..., "sessionId":8}`. The broadcastId and sessionId belong to Live TV's own session, so from Lumen the likely
  route is launching `com.webos.app.livetv` with a channel id (still a guess).
- **Step 1f (2026-10-17)**, each with `'{}'`: `getPrimaryChannelList` → `ERROR_06 invalid param(channelId)` (probably the related
  channels of one channel, not a list); `getChannelZappingList` → `Message status unknown` plus a cancel error (no reply in time, or it
  wants parameters); `getChannelGroup` → `{"channelGroupId":0,"channelGroupName":"All"}` (the current group). Next: list every `get*`
  method and category (the first introspection was cut at 40 lines), and retry `getChannelZappingList` with a group and a channel id.
- **Step 1g (2026-10-17):** categories `/`, `/oipfdrmagent`, `/ci`, `/scan`. Full `get*` list saved on the TV in `/tmp/intro.txt`.
  List-shaped ones worth trying: `getChannelZappingList`, `getChannelZappingListCount`, `getSortedList`,
  `getChannelListToSolveConflictLcnForLiveTV`, `getLogoImage` (for tiles later). `getChannelZappingList` gives
  `Message status unknown` with `{"channelGroupId":0}`, `{"channelGroupName":"All"}` and a channel id too. Guess: `-n 1` stops luna-send
  at a first status message before the real reply. Next: try the count and `getSortedList`, and the zapping list without `-n 1` for 5 s.
  If that fails, capture with `ls-monitor` while opening Live TV's own channel list (the list button), to copy its exact call.
- **Step 1h (2026-10-17): `getChannelZappingListCount '{}'` → `{"channelListCount": 330}`.** The aerial list is there. `getSortedList` →
  `not handled`. luna-send refuses to run without `-n` or `-i`, so the 5 s test did not run. Next: `-i` with a timeout, then paging
  parameters in case the 330-channel reply is too big for one message.
- **Step 1i (2026-10-17):** `getChannelZappingList` with `-i` gives no reply at all in 5 s; with `{"startIndex":0,"count":5}` and
  `{"startIndex":1,"endIndex":5}` the same `Message status unknown`. Stop guessing parameters. Next: `ls-monitor` while opening Live TV's
  channel list and the Guide, then copy the exact call (and see whether the reply comes back on the bus at all).
- **Step 1j (2026-10-17): found the real call.** There is no separate Guide app (`listApps` has `livetv`, `channeledit`,
  `channelsetting`, `lgchannels`, ...; `com.webos.exampleapp.*.epg` are LG sample apps, not the Guide). Launching
  `com.webos.app.channeledit` over SSH (`applicationManager/launch`) with `ls-monitor` running showed it asks **`iepg`** after all, so the
  "iepg is internet channels only" guess was wrong. It just needs `channelMode`:
  `luna://com.webos.service.iepg/getChannelList {"dataType":0,"sort":0,"channelMode":"Terrestrial","attribute":[{"filterType":"Invisible","value":false}],"requestField":["channelId"]}`
  (also with `"Cable"`, `"Satellite"` and `["IP"]`), then `iepg/getChannelInfo {"dataType":0,"channelId":"1_41_104_104_16522_17664_9018"}`.
  It also calls `com.webos.service.utp.channel` `getChannelModeList '{}'` and `getChannelTypeList {"channelModeId":0..2}`.
  Next: the same `getChannelList` without `requestField`, to see which fields come back per channel.
- **Step 1k (2026-10-17): the aerial list works as root.** `luna-send -n 1 -f luna://com.webos.service.iepg/getChannelList
  '{"dataType":0,"sort":0,"channelMode":"Terrestrial","attribute":[{"filterType":"Invisible","value":false}]}'` → `channelListCount: 188`.
  Fields per channel include `channelNumber` ("1"), `channelId` ("1_41_1_101_16522_17546_9018": type, physical 41, major 1, minor 101,
  then the signal id), `signalChannelId` ("16522_17546_9018", the join key to the guide's programmes), `majorNumber`, `minorNumber`,
  `physicalNumber`, `HDTV`, `DTV`, `Radio`, `Data`, `ATV`, `skipped`, `PrimaryCh`, `payChan`, `adultFlag`, `isFreeviewPlay`,
  `groupIdList`, `favoriteIdxA..`, `playerService: "com.webos.service.legacybroadcast"`. Lumen can make this call through
  `app/src/lib/root.ts` (Homebrew Channel exec). Next: confirm the name and logo fields, then try tuning to a channel from SSH.
  Spike answer so far: **yes, the channel list and the guide are both readable.**
- **Step 1l (2026-10-17): names confirmed.** `channelName` on all 188 ("BBC ONE Yks HD" on 1, "BBC TWO HD" 2, "ITV1" 3, "Channel 4" 4,
  "5" 5, ...), plus `channelNameSortKey`. Logo candidates: `imgUrl` and `channelLogoSize` on every channel, `channelLogoServerUrl` once
  (LG's logo server, seen in Step 0). Next: look at `imgUrl` values, then tuning.
- **Step 1m (2026-10-17): logos are cached files on the TV.** `imgUrl` is a path like `/tmp/gibbips/logo/s210a81942c909451daf38c567c71af39`
  (no extension), `channelLogoSize` like `"H133x100"` or `"H356x200"` (empty for channel 1). Being in `/tmp`, they are probably
  fetched again after each boot (from `channelLogoServerUrl`, LG's `gftsDownload.lge` server), so Lumen should copy them (as root) or
  read them through the root exec rather than keep the paths. Next: check the files exist and what format they are.
- **Step 1n (2026-10-17): logos are PNGs.** 176 files in `/tmp/gibbips/logo` (for 188 channels, so a few have none), about 2 to 3 KB
  each, PNG signature. Lumen can read them as base64 through the root exec and cache them. Next: tuning, by launching
  `com.webos.app.livetv` with `{"params":{"channelId":...}}` (a guess).
- **Step 1o (2026-10-17): tuning not solved.** `luna-send -n 1 luna://com.webos.applicationManager/launch
  '{"id":"com.webos.app.livetv","params":{"channelId":"1_41_2_102_16522_17472_9018"}}'` returned `returnValue: true` but the channel did
  **not** change (Live TV was already in front). Next: the same launch with something else in front, so Live TV starts fresh and reads
  the parameter; if not, capture Live TV's own call when a channel number is typed on the remote.
- **Step 1p (2026-10-17):** same launch with Home in front: Live TV opened, but on the channel it was already on (BBC One), so the
  `channelId` launch parameter is ignored. Next: `ls-monitor` while typing a channel number on the remote, to copy Live TV's call.
- **Step 1q (2026-10-17):** typing 2 on the remote: `com.webos.app.livetv` calls `luna://com.webos.service.utp.broadcast/changeChannel`
  (multi-line payload, not seen in full yet), then `utp.broadcast` calls `legacybroadcast.broadcast/changeChannel` with
  `{"broadcastId":"bE439...","channelId":"1_41_2_102_16522_17472_9018","currentChannelId":"1_41_1_101_...","sessionId":5}`.
  Also seen: the HbbTV plugin calls `iepg/getChannelList {"channelGroup":["All"],"channelMode":["Tuner"],"dataType":0,"sort":0}`
  (another working form), and `nlpmanager` calls `iepg/getEventInfo {"channelMode":["Tuner","IP"],"programId":"bbc_two_hd_1/2026-10-1717:05","dataType":0}`.
  Next: print Live TV's full `changeChannel` payload.
- **Step 1r (2026-10-17): Live TV's tune call in full:** `luna://com.webos.service.utp.broadcast/changeChannel`
  `{"broadcastId":"00000001-559c-d46a-2902-723da034b853","channelId":"1_41_2_102_16522_17472_9018","connection_timeout":500,"fallback":true,"ignorable":false}`.
  The `broadcastId` is Live TV's own broadcast session, so another caller needs the current one. Next: send the same call as root with
  that id while Live TV is open, and list `utp.broadcast` methods that could return the current broadcast id.
- **Step 1s (2026-10-17): tuning works as root.** With Live TV open, `luna-send -n 1 luna://com.webos.service.utp.broadcast/changeChannel
  '{"broadcastId":"00000001-559c-d46a-2902-723da034b853","channelId":"1_41_2_102_16522_17472_9018","connection_timeout":500,"fallback":true,"ignorable":false}'`
  → `{"reason":"changeChannel","returnValue":true,"nextChannelId":"1_41_2_..."}` and the TV switched to BBC Two (seen on the TV).
  The broadcast id captured earlier was still accepted, even though `/tmp/ch.txt` had gone in between (a restart, or `/tmp` cleared).
  Guess: the id is fixed per tuner (`00000001-...`), not per Live TV session. `getCurrentChannel` and `getBroadcastState` with `'{}'` →
  `-106 No resource` (they probably want the broadcast id). `utp.broadcast` `get*` methods: getBroadcastState, getChannelChangeResult,
  getChannelChangeStatus, getChannelState, getCurrentChannel, getPlayState, getSignalStatus, getVideoInfo and others.
  Next: confirm the id survives Live TV closing and opening (launch Live TV over SSH, then change channel).
- **Step 1t (2026-10-17):** from Home, launch Live TV over SSH, wait 5 s, `changeChannel` to Channel 4 HD with the same broadcast id:
  Live TV opened on the last channel (BBC Two) and did not switch. `changeChannel` and `getCurrentChannel` both answered
  `-106 No resource: 00000001-559c-...`, so **the broadcast id is per Live TV session**: it changes when Live TV restarts (the Step 1s
  guess that it is fixed was wrong). Tuning this way needs the current id, which nothing found so far returns.
  Lead: Home Assistant's LG webOS integration changes channel by name with Live TV closed. It talks SSAP (the websocket API on port
  3000/3001): `ssap://tv/getChannelList`, then `ssap://tv/openChannel {"channelId":...}` (from aiowebostv; my understanding, not checked
  against its source this session). Next: `ls-monitor` while Home Assistant changes channel, to see which luna call the SSAP gateway
  makes. That call, made as root, is likely the one Lumen should use.
- **Online (2026-10-17):**
  - aiowebostv (Home Assistant's library): `set_channel` sends `ssap://tv/openChannel` with `{"channelId": channel}`; also
    `tv/getChannelList`, `tv/getCurrentChannel`, `tv/getChannelProgramInfo`, `tv/channelUp`, `tv/channelDown`
    (github.com/home-assistant-libs/aiowebostv, `endpoints.py`, `webos_client.py`).
  - ioBroker forum: `ssap://tv/openChannel {channelNumber: '1'}` also reported to work.
  - webos-forums.ru topic 6886 (LG 55UP8150, 2021): root tuning is `utp.broadcast/changeChannel` with the *current* broadcast id, and
    the author gets that id by scraping `ls-monitor` (`grep -m 4 broadcastId`). `getCurrentChannel` takes `{"broadcastId":...}`.
    So nobody there found a clean getter for the id either.
  Next: find which luna call the SSAP gateway maps `tv/openChannel` to (grep its config on the TV). That call needs no broadcast id.
- **Step 1u (2026-10-17):** a recursive grep over `/etc` and `/usr/share` is too slow on the TV; don't. The SSAP gateway is a JS service,
  `com.webos.service.secondscreen.gateway`, at `/usr/palm/services/com.webos.service.secondscreen.gateway` (`run-js-service`), with
  permissions in `/usr/share/luna-service2/{api,client}-permissions.d/` and `roles.d/`, and db8 kinds in `/etc/palm/db/kinds/` (client
  profiles and certificates). Next: grep only that service folder for `openChannel`.
- **Step 1v (2026-10-17):** `helpers/bus/filtering-uris.js` in the gateway lists both `tv/openChannel` and
  `com.webos.service.apiadapter/tv/openChannel`; `interfaces/tv.interface` and `interfaces/com.webos.service.apiadapter.interface` also
  mention it. So SSAP `tv/openChannel` is served by **`luna://com.webos.service.apiadapter/tv/openChannel`**. Next: call that as root
  with `{"channelId":...}` and Live TV closed.
- **Step 1w (2026-10-17): tuning solved.** With Live TV closed (Home in front), as root:
  `luna-send -n 1 luna://com.webos.service.apiadapter/tv/openChannel '{"channelId":"1_41_104_104_16522_17664_9018"}'` opened Live TV on
  Channel 4 HD (seen on the TV; the reply itself was not posted). No broadcast id or pairing needed. Not tried: from the app without
  root, and with Live TV already open on another channel (it should behave like Home Assistant's `set_channel`, which does both).
- **Step 2a (2026-10-18): the TV clears its guide at switch-on.** Lumen's log: at 21:47 the schedule kind had 31,006 rows; at 05:10 after
  switch-on it had 0, and now and next listed 0 channels at 05:23. Both fill again only while Live TV is tuned in. Lead: `iepg` may serve
  an online guide (Freeview Play) that does not need the tuner. Its methods include `getEpgStatus`, `getCurrentNextEventList`,
  `getEventList`, `getNextEvent`, `getEventInfo`, `getSearchEventList`, `getSeriesEventList`, `getProgramImg`, and the notifiers
  `notifyOnlineEventChanged` and `notifyFvpEventChanged` (FVP is Freeview Play). Next: call the `get*` ones with the guide empty.
- **Step 2b (2026-10-18):** `iepg/getEpgStatus {"dataType":0,"channelMode":["Tuner","IP"]}` (guide full at the time):
  `tunerOnlineEPGSupport: true`, `mergedEPGSupport: true`, `baseEpgOnline: false`, `networkStatus: true`, `tunerEpgStatus` ok,
  `ipEpgStatus` off ("channelplus setting is off", that is LG Channels). So the TV can show an online guide for aerial channels but is
  not using one now. `getCurrentNextEventList` and `getEventList` with those parameters (and a `channelId`): "bad parameters".
  Next: capture the parameters LG's own Guide sends to `iepg` with `ls-monitor`.
- **Step 2c (2026-10-18):** LG's Guide is `com.webos.app.livemenu`. On opening it calls `iepg` `getEpgStatus`, `getFavoriteGroupList`,
  `getCurrentFavoriteGroup`, `getGenreMapTable`, `getCountryPolicy`, then `getEventList`. Payloads span several lines in `ls-monitor`, so
  `grep` on one line shows only `«{`; read them with `grep -A`. A filter on `iepg/get` matches nothing: service and method are separate fields.
- **Step 2d (2026-10-18): the Guide's schedule call.** LG's Guide asks `luna://com.webos.service.iepg/getEventList` with
  `{"bFVPChannelId":true,"channelId":["1_41_1_101_16522_17546_9018", ...10 ids],"dataType":0,"fepgType":"Forward",
  "localStartTime":"2026,10,18,06,30,00","localEndTime":"2026,10,18,08,30,00","sort":0}` (a two-hour window, ten channels a call;
  local times, `iepg` adds the UTC ones and passes it on to `com.webos.service.pbsw`). A first call with one channel and start equal to
  end returns what is on now. The reply is `programList`, with `programName`, `description`, `startTime`/`endTime` (local),
  `DSTStartTime`/`DSTEndTime`, `duration`, `signalChannelId`, `isPresent`, `seriesTitle`, `fepgImgUrl`. Seen with the broadcast guide full.
  Unknown: whether it answers (from Freeview Play online) while the broadcast guide is empty after switch-on.
- **Step 2e (2026-10-19): the now and next table can be part filled.** Lumen's log: 211 channels listed, only 28 with something on now, and
  every channel on the row empty, while the guide service had them all. Lumen now asks the guide service for any row channel with nothing on
  now (every 5 minutes while one stays empty), and the row also uses what the Full guide read.
- **Step 2f (2026-10-19): one guide service read.** Now and Next stayed on "No guide data" for channels off the row until the Full guide
  had read them. Now one guide service read drives everything: the row's channels and the first 10 at start-up, the rest when the Guide opens.

## Answer

| What | Call | Notes |
|---|---|---|
| Channel list | `iepg/getChannelList {"dataType":0,"sort":0,"channelMode":"Terrestrial","attribute":[{"filterType":"Invisible","value":false}]}` | Root. 188 channels. `channelNumber`, `channelName`, `channelId`, `signalChannelId`, `imgUrl`, `Radio`, `HDTV`, `skipped`. |
| Logos | read the file at `imgUrl` | Root. PNG, 2 to 3 KB, under `/tmp/gibbips/logo`, so cache them. 176 of 188 have one. |
| Guide | `epgdb/find {"query":{"from":"com.webos.service.legacybroadcast.programSCH:4", ...}}` | Root. Join on `signalChannelId`. `programPF:4` holds now and next. Filled while Live TV runs, so it may be thin if Live TV has not been opened lately (untested). |
| Tune | `apiadapter/tv/openChannel {"channelId":...}` | Root. Opens Live TV on that channel, even from Home (Step 1w). Don't use `utp.broadcast/changeChannel` (needs Live TV's per-session broadcast id, Step 1t) or a launch parameter (ignored, Step 1o). |

Still unknown: whether `openChannel` works without root; whether the guide data stays filled when Live TV hasn't been used for a while, and the `epgdb` query syntax for one
channel and a time window (db8 `where` on `signalChannelId` and `startTime` needs an index on those fields; check `getProfile` or try it).

## Live preview in a page (from Glasshouse, 2026-11-06; untested on this TV)

Glasshouse (rorygallagher2024/glasshouse, MIT) shows a silent live picture of the channel in focus on its Home and guide.
Its notes (`docs/development/TV-RESEARCH.md`) and code (`server/assets/home.html`, `startPreview`) say, for a G3 on webOS 9.2.2:

- A muted `<video>` with one `<source type="service/webos-broadcast-preview" src="tv://<channelId>">` tunes that channel in about
  120 ms and leaves the TV's last channel as it was. `service/webos-broadcast` with the same source also plays, but makes it the
  last channel. The channel id is the full one, as `iepg/getChannelList` gives it ("1_41_1_101_16522_17546_9018").
- The picture's format shows a moment after `playing`. A channel off air plays with no format and a black picture.
- To let the tuner go: remove the `<source>` children and any `src`, then `load()`. Hiding the element keeps the tuner.
- Timing: it waits 400 ms (600 ms for an internet stream) on a tile before it starts, and gives up after 4 s with no `playing`.
- Moving the playing `<video>` to another parent lost its picture on webOS 9.2.2, so it keeps one element and moves it with
  `left`/`top`/`width`/`height`.

To find out on this TV: whether `webos-broadcast-preview` plays at all on webOS 25, how long it takes, whether it stops Live TV or a
recording, and whether it plays on the hardware video layer under the page (as the video wallpaper does), so the guide needs a clear
hole in its backdrop where the box is.

Spike on this TV, 2026-11-06, over DevTools from Lumen's home screen (a fixed 640x360 muted `<video>` added to the page):

- It plays on webOS 25. In the first try, BBC Two reached `playing` at 1920x1080 about 1.45 s after `load()`, and a retune to ITV1
  took about 0.73 s. Lumen's screensaver started during that try, which is the likely cause of the delay.
- Two timed runs later the same day, with nothing else running, matched Glasshouse. Twelve tunes, half in the same multiplex and half to
  another (the fifth part of the channel id), took 113 to 334 ms to `playing`. The first, from cold, was the slowest. Six more tunes,
  each after emptying the video and calling `load()` (as `release` in `LivePreview.tsx` does), took 115 to 156 ms. Neither a change of
  multiplex nor the release step costs anything that can be measured.
- A `stalled` event comes about 3 s after `playing`, but the picture keeps moving. Do not treat `stalled` as a failure.
- The box is black in a DevTools screenshot, the same as the video wallpaper, so the picture is on the hardware layer under the page.
  Sam saw a moving picture in a box laid over the Full guide, with no change to the guide's backdrop: the `<video>` cuts its own
  hole through the page.
- The picture shows about 2.5 s after `playing`, not with it (2026-11-06). `playing` came at about 340 ms, but the TV kept the video
  muted until about 2.9 s, and Sam saw black for that whole time. After `playing` the page sees no change: `currentTime` stays 0,
  `webkitDecodedFrameCount` stays 0 and `requestVideoFrameCallback` never fires. The real sign is the TV's mute state:
  `luna://com.webos.service.avmutemanager/listMuteStatus {}` answers from Lumen's page with no root, and `videoMute[].videoMuteStatus`
  turns false when the picture appears. It does not take `subscribe`, so poll it. Between tunes the list is empty.
  `ls-monitor` shows `legacybroadcast.avmute` setting the mute when the tune starts and clearing it when the picture is ready.
  The wait changes per channel: 1.96 s (5), 2.12 s (6), 2.88 s (7) and 3.45 s (10) from the tune to the unmute. Sam watched through a
  half see-through logo and saw the picture appear just as the unmute came, so the mute state is a true sign of the picture.
- Live TV's last channel is unchanged: after a BBC One preview, Live TV still opened on 202 (CBeebies HD), the channel Sam had been
  watching. `com.webos.service.tv.channel` and `tv.broadcast` do not exist on this TV, so the check was by eye.

## Plan (do in order, stop when one works)

1. **Check `ls-monitor` sees anything at all.** Save the raw output and look at it unfiltered:
   ```
   timeout 20 ls-monitor > /tmp/lsmon.txt 2>&1; wc -l /tmp/lsmon.txt; grep -ciE 'channel' /tmp/lsmon.txt; head -40 /tmp/lsmon.txt
   ```
   Change channel during the 20 seconds with Live TV in front. If it shows traffic, grep `/tmp/lsmon.txt` for the service names next to
   the channel change.
2. **Find which service declares a channel list method.** The bus permission files list every method:
   ```
   grep -rlE 'getChannelList|getCurrentChannel|channelList' /usr/share/luna-service2/ /etc/luna-service2/ 2>/dev/null
   grep -rhoE '"[A-Za-z0-9./_-]*(getChannelList|getCurrentChannel)[A-Za-z0-9_]*"' /usr/share/luna-service2/ 2>/dev/null | sort -u
   ```
   Any service other than `iepg` that turns up here is the next candidate. Try it with `'{}'` first, then with a group from the list above.
3. **Query `epgdb` directly.** Find its kind names, then `find` with a small limit:
   ```
   grep -rlis epg /etc/palm/db/kinds /etc/palm/db_kinds /usr/palm/db/kinds 2>/dev/null
   luna-send -n 1 -f luna://com.webos.epgdb/find '{"query":{"from":"<kind name>","limit":3}}'
   ```
4. **Fall back to LG's second-screen API (SSAP).** This is what phone remote apps use, and community libraries (lgtv2, aiowebostv) call
   `ssap://tv/getChannelList`, `ssap://tv/getCurrentChannel` and `ssap://tv/getChannelProgramInfo` over the websocket on port 3000 (3001
   for TLS). It needs a one-off pairing prompt on the TV, but no root, so Lumen could call it itself. Untested on webOS 25.
5. **Write down the answer** in `docs/research/spikes.md`: which call works, the fields per channel (number, name, id, logo), and how to tune to
   one from Lumen (launching Live TV with a channel id is a guess until tried). Then tick the Live TV spike in docs/plans/PLAN.md.

If none of these returns the aerial channels, the Freeview tiles idea goes to Not doing in docs/plans/PLAN.md, with the reason.

## Reading Live TV's channel (2026-11-06)

For the Guide to open on the channel being watched. Tested over SSH on this TV.

- `apiadapter/tv/getCurrentChannel {}` gives the last channel opened through apiadapter (Lumen's `openChannel`), not one changed to with
  the remote inside Live TV. With Live TV closed it gave nothing. Lumen's page may not call apiadapter; the root path can.
- `/mnt/lg/cmn_data/lastchannel.json` (`lastMajorNum`) and the call above both updated only when Live TV next opened and tuned.
- `utp.boot/getLastChannelId {}` gives `{"channelId": "1_44_10_10_..."}`. Glasshouse (`server/lib/home.js:938`, `:1011`) reads it
  as Live TV leaves, and its comment says a change inside Live TV is not seen until then. That is the Guide button's case.
- `legacybroadcast.broadcast/getCurrentChannel` and `utp.broadcast/getCurrentChannel` need a `broadcastId` that only Live TV holds;
  a forum post finds it with `ls-monitor`. `{"subscribe":true}` without one fails. `apiadapter/tv/channelUp` does not exist here.
- `getLastChannelId` and `lastchannel.json` follow a channel changed with the remote while Live TV is in front, within a second or
  two (a watcher polling every 0.5 s saw 6, 5 and 4 as Sam changed them from Home Assistant). An earlier miss was the old Guide left
  open behind Live TV, which never asked again.
- With the build using `getLastChannelId`, a `{"view":"guide"}` launch (the Guide button) opened the Guide on channel 4, the one
  Live TV was on.

