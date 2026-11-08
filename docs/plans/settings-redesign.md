# Settings: tvOS layout

Written 2026-10-17. Reference: Apple TV Settings (a sidebar picture on the left, a list of rows on the right,
each row opens a page with the same layout, Back goes up one level).

## Pages now

The page stack below is built. On 2026-10-26 the option pages were regrouped from first principles. The rule: each page is named after
the screen it changes, and anything two screens share (the weather location, the Immich server) is set once, on a page of its own.

| Row | Contents |
|---|---|
| Apps, Sources, Channels | grid pages, unchanged. Channels shows only with the Live TV row on |
| Home Screen | Live TV row, Apps per row, Hide app names, Show recents indicator, Hide the dock, Dim when the dock is hidden, Date and weather, Weather widget |
| Live TV | Dev builds only, with the Live TV row on: Preview in the Guide, Preview on the Live TV row, Sound. Added 2026-11-06 |
| Wallpaper | Wallpaper type, Choose wallpaper, then only the rows the type uses (Change automatically and New picture on startup for pictures and photos, Video wallpaper hours for video), Dim the wallpaper |
| Screensaver | Screensaver (Enabled or Disabled), then only when Enabled: Start on the home screen after, Show (Wallpaper pictures or Immich photos, only with Immich connected), Immich photos (only when Show is Immich photos), Time on each picture, Brightness, Print look, Weather widget |
| Brightness | Brightness schedule, then the start time and backlight for Day, Evening and Night, Change (at once or over a time), Eye Comfort. Added 2026-11-02 |
| Appearance | Appearance, Focus highlight, Reduce motion, Dock blur |
| Photos | Server address, API key, Test connection, Recent photos from |
| Language and Region | Language, Clock, AM/PM on the clock (not with a 24-hour clock), Weather country, Weather city |
| Home and Power | Home button opens, When the TV turns on, Restart the TV now (only while a restart is needed). Added 2026-11-01, `docs/home-and-power-on.md` |
| Privacy | LG data agreements, Limit ad tracking, Live Plus, Block LG ad servers, Stop LG tracking services, Block TV software updates. Added 2026-11-01, `docs/privacy.md` |
| System | TV scripts, Show system apps, Restore settings from the TV; dev builds add the frame-rate counter and Liquid glass dock |

What changed from the first grouping:
- The Screensaver page starts with Enabled or Disabled. Disabled runs LG's default screensaver and hides the rest of the page. Enabled shows the wallpaper pictures picked in
  Choose wallpaper, or Immich photos. There is no Off: turning the TV's screensaver off belongs to the service menu, with its burn-in
  warnings (2026-10-27). Each launch removes the `/var/lib/homehook/no-screensaver` flag that the old Off set.
- Immich choices show only once Immich is connected. The Photos page is now called Immich.
- Each Weather widget is one choice, Off, Small or Large, instead of a switch and a size row. The stored fields are unchanged.
- The Weather page is gone. Date and weather moved to Home Screen, and the location to Language and Region.
- Photos is new: the Immich server is typed and tested there (`docs/immich.md`), and Recent photos from moved there, as it changes the
  wallpaper and the screensaver.
- Live app list via Homebrew Channel is gone; the app list always comes from the Homebrew Channel, which installs Lumen.
- Liquid glass dock is in dev builds only, as it is too slow on this TV. A release build loads it as off.
- Apps, Sources and Channels stay top-level pages, as they are the places people go to change what the home screen shows (Sam's call).

The first plan (2026-10-17) and its layout notes are built; they are in git history (`git log -- docs/plans/settings-redesign.md`).
