# Settings: tvOS layout

Written 2026-10-17. Plan only, nothing built. Reference: Apple TV Settings (a sidebar picture on the left, a list of rows on the right,
each row opens a page with the same layout, Back goes up one level).

## What is there now

`screens/Settings.tsx` is one screen with a top tab bar: Apps, Sources, Channels (only with Live TV), Wallpaper, Options.
- Options (`components/OptionList.tsx`) is one flat list of 20 rows built in `app.tsx` (around line 370). Each row is tall: a title, a
  two-line description and a switch or value. Only about five fit on screen, so finding one means scrolling the whole list.
- List options open a pop-up (`components/ChoicePicker.tsx`) over the list.
- A hint line above the tabs explains the keys for each tab.
- The re-order button floats at the right end of the tab bar (`global.css` around line 628).

What makes it clunky: no grouping, descriptions repeated on every row, the pop-up breaks the flow, and the tab bar plus hint take the
top of the screen.

## Target layout

Two columns on every list page, as on tvOS.
- Left column, about 40%: a large rounded square with a glyph (`components/Glyph.tsx`) for the focused row on the top page. On a
  sub-page it shows the page's glyph, and the focused row's description in small grey text under it.
- Right column, about 60%: a page title at the top centre, then single-line rows. Each row has its label on the left, and on the right
  its value in grey text (`On` / `Off`, `London`) or a chevron for a row that opens a page. The focused row is white with dark text, as
  in the reference screenshot.
- No descriptions inside rows. No hint line on list pages.

Navigation is a page stack. OK on a chevron row pushes a page, Back pops one, Back on the top page closes Settings. Focus on each page
is kept when you come back to it.

## Pages

Top page rows, in this order:

| Row | Opens | Contents |
|---|---|---|
| Apps | grid page | today's Apps tab |
| Sources | grid page | today's Sources tab |
| Channels (Live TV only) | grid page | today's Channels tab |
| Wallpaper | list page | Choose wallpaper (grid page: today's Wallpaper tab), Change wallpaper automatically, Video wallpaper hours, Dim the wallpaper at night |
| Home Screen | list page | Live TV row, Hide the dock, Apps in the glass bar, Hide app names |
| Screensaver | list page | Screensaver, Time on each photo |
| Weather | list page | Date and weather, Weather country, Weather city |
| Appearance | list page | Appearance, Focus highlight, Reduce motion, Live blur |
| Language and Clock | list page | Language, Clock, AM/PM on the clock |
| System | list page | Live app list via Homebrew Channel, Show system apps, Restore settings from the TV, Frame-rate counter (dev builds) |

Every one of today's 20 options has a home. None are removed.

Row types on a list page:
- Switch option: value text `On` / `Off`. OK flips it in place.
- List option (has `choices`): value text is the current choice. OK pushes a page that lists the choices, with a tick on the current
  one. OK sets it and pops. This replaces `ChoicePicker`.
- Action (Restore): no value. OK runs it.

Grid pages (Apps, Sources, Channels, Choose wallpaper) are full width, not two columns, because a grid needs the room. The page title
sits at the top centre. The re-order button sits at the top right, in line with the title. The hint line stays on grid pages only,
because hold-OK to re-order and hold-OK to favourite cannot be found without it.

## Build order

Each step leaves Settings working.

1. Page stack and list page. A new `SettingsList` component draws the two columns and rows. The top page uses it, and each grid tab
   becomes a pushed page. The tab bar goes. Options stays as one pushed "Options" page for now, so nothing is lost mid-way.
2. Groups. Add a `group` field to `OptionItem`, set it per option in `app.tsx`, and build the list pages above from it. Remove the
   Options page.
3. Choice pages. List options push a choices page. Delete `ChoicePicker` once nothing uses it.
4. Left column. Glyph square for the top page, glyph plus focused description for sub-pages. Remove descriptions from rows.
5. Styling pass: light mode, Reduce motion (no slide between pages), focus highlight levels, the blurred backdrop
   (`.screen--settings::before` in `global.css`), and new words in `lib/i18n.ts`.

Steps 1 and 2 touch the same files, so build them one after the other. Steps 3 and 4 can run together after step 2.

## Checks on the TV

- Every option from today's list can be found and changed.
- Back from any depth returns to the row you came from.
- Re-order and Favorites still work on the grid pages.
- The page opens without a visible delay. Settings is in the main bundle (see `PLAN.md`, lazy loading was dropped), so keep the new
  component small and draw only the current page.

## Open questions

- Are the groups and their names right? They are a first guess.
- Should a switch row show `On` / `Off` text (tvOS) or keep today's switch?
