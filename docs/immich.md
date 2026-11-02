# Immich photos as the wallpaper

Settings > Wallpaper can show your own photos from an Immich server on the home network.

## Set up

1. In Immich: Account Settings > API Keys > New API Key. Give it only these permissions: `album.read`, `asset.read`, `asset.view`,
   `memory.read`, `face.read`, `stack.read`. It can then read photos and nothing else. Without `face.read` everything still works, but photos are
   not placed on their faces (see below). Without `stack.read`, every photo in a stack can show, not only its cover.
2. On the TV: Settings > Immich. Type the Server address (as you would open it in a browser on the home network: scheme, host and port,
   no path; `http://` is added when left out) and the API key. OK on a row opens a text field and the TV's keyboard. OK saves, Back
   cancels.
3. Choose Test connection. Lumen asks the server for its version, then for its albums with the key. When both answer, it writes
   `/var/lib/homehook/immich.json` and the photo options open up. When one fails, a message says which.

Over ssh instead (the same file):

```sh
cat > /var/lib/homehook/immich.json <<'JSON'
{"url": "http://192.168.1.10:2283", "key": "PASTE-THE-KEY-HERE"}
JSON
chmod 600 /var/lib/homehook/immich.json
```

Settings > Wallpaper > Wallpaper type > Immich photos then picks the photos for the home screen.

## Sources

A stack shows as its cover photo only. Lumen reads the list of stacks from Immich every 6 hours and leaves the other photos in each stack out.

- **Recent**: random photos taken in the last 6 months. Settings > Immich > Recent photos from changes this to the last month, 3 months or year.
- **On this day**: Immich's memories for today (photos from this date in past years); Recent when there are none.
- **Favourites**: random from the photos marked as favourites.
- **All my photos**: random from everything.
- **Your albums**: each album with photos in it is listed by name.

Landscape photos fill the screen. Portrait photos show in pairs, side by side, so they are not cropped to a strip. The two photos in a pair
come from different days, so a pair is not two shots of the same moment. A portrait with no partner from another day in the batch is
skipped. Square photos are skipped, since they lose too much in either layout. Recent and All also skip pictures with no camera make in
their EXIF, which drops screenshots and saved images. The next picture comes from other days than the one before it (both days,
for a pair), so two pictures in a row are not from the same day. Lumen searches again for one before it gives this up. Recently shown photos (the last 40) are not repeated, counted per photo, so a
portrait does not come back soon in a new pair.

A photo is cropped to fill the screen, or its half of it for a pair, so part of it is always cut off. Lumen moves the crop so the faces
in it stay in view, a little above the middle. A photo with no faces keeps more of its top than its bottom, since heads and subjects tend
to sit high. A 9:16 phone portrait in a pair loses about 300 px top and bottom together, so this matters most there.

## How it works

- Immich sends no CORS headers outside its dev mode, so a TV web app cannot call its API directly. Every API call goes through curl on the
  TV (`immichRequest` in `app/src/lib/root.ts`, which only accepts a plain address, a plain key and six fixed paths).
- Face positions: the search says only which photos have people, not where their faces are (checked on Immich 3.3). So when a photo with
  people is picked, Lumen asks `/api/faces?id=<id>` for its face boxes, one small request per photo. Each photo's shape and face centre
  are kept in localStorage (`lumen-immich-frames-v1`, the newest 300), and `picturePosition` in `lib/immich.ts` turns them into a
  `background-position` for the wallpaper and the screensaver.
- The photo itself is loaded as an ordinary image from `/api/assets/<id>/thumbnail?size=preview&apiKey=...` (about 1440 px on the long
  side), which needs no CORS. It is never sent through images.weserv.nl or anywhere else off the home network.
- The blurred copies the glass and the lower rows show are made on the TV from Immich's small thumbnail, once per photo. For a pair, both
  thumbnails are blurred side by side, as the pair shows.
- The next photo is chosen 8 s after the current one is up and its blur goes into the boot record, so a cold start paints that blur at once
  and the photo fades in over it, as for the cloud wallpapers.
- The key is kept in the app's localStorage on the TV (to build the photo URL at start-up without a root call) and in
  `/var/lib/homehook/immich.json`. Use a read-only key. Settings writes the file with `writeFile` in `lib/root.ts`, so it does not get
  the `chmod 600` the ssh steps give it.

## Screensaver

Settings > Screensaver: set Screensaver to Enabled and Show to Immich photos, and tick one or more sources under Immich photos. With several ticked, each batch of photos comes from one of them at random. When the TV's screensaver starts, `watch.sh` opens Lumen on
its ambient screen instead: your photos full-screen, a 1.6 s crossfade every 30 seconds (Settings > Screensaver > Time on each photo: 10 s to 5 min), each photo zooming in very slowly, and the clock,
date and weather bottom left. Any key returns to the home screen.

- Settings > Screensaver > Start on the home screen after (30 seconds to 5 minutes): when the home screen is left alone that long, Lumen
  opens the photos itself, so LG's screensaver never shows first. Control Center > Art mode opens the photos at once, even with another Screensaver choice. The TV's own
  screensaver timer still covers other apps.
- The first photo is loaded 8 seconds after the home screen shows, so both of these open on it about 1.2 s in, once the clock has slid
  into place. Each next photo loads 3 seconds into the one before it, so a crossfade never waits on the network. Measured 2026-11-02 on the
  home network: a photo takes about 0.2 s to download and decode, and a new batch from Immich about 0.6 s. (verify)
- Needs the updated `tools/tv/watch.sh` on the TV (it looks for `/var/lib/homehook/ambient-screensaver`, which the setting creates).
- LG's screensaver keeps starting after its timer while the ambient screen is up (nothing is pressed); `watch.sh` sees
  `/tmp/lumen-ambient-on` and only ends it.
- With the video wallpaper on and Lumen in front, the video wallpaper wins, as before.
- Like Lumen Gallery's screensaver, it takes the screen from whatever app was in front (a paused Plex film, for example).
