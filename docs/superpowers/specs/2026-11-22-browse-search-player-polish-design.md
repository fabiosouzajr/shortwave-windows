# Browse headings, player heart, country flags, broad search

## Intent
Four usability changes requested together:
1. "Popular Stations" / "Discover New Stations" on the Browse page are too small
   to read as section titles.
2. Saving the playing station should be one click from the player view, with no
   confirmation dialog, only a short "Station added" message.
3. Each station card should show a small flag next to its country.
4. Search should be broad: station names, genres, countries, codecs and
   bitrates, alone or combined, e.g. "all FLAC stations in Brazil" with no
   name typed.

## Current state
- Headings use the libadwaita `heading` class (body-sized bold),
  `data/gtk/search_page.ui:90` and `:112`.
- Player view controls row (`data/gtk/player_view.ui:120`): connect · play/stop
  stack · details. Window actions are installed in `src/ui/window.rs`
  (`player.show-station-details` at line 85). `SwLibrary` has `add_station` and
  `contains_station`; `SwApplicationWindow::show_notification(text)` shows an
  `adw::Toast`.
- `SwStationRow` (`src/ui/station_row.rs`) builds a subtitle
  `"{Country} · {tags}"` from `StationMetadata` (`country`, `countrycode`).
- Search (`src/ui/pages/search_page.rs::search_changed`) sends
  `StationRequest::search_for_name(text, 1000)` (votes desc) to
  `/json/stations/search`; empty text returns to the discover view.
  `StationRequest` already has `countrycode`, `tag`, `bitrate_min`; it has no
  `codec` field.
- radio-browser `/stations/search` ANDs all given parameters; there is no
  "match any field" parameter.

## Goals
- Headings visibly larger than card text.
- Heart button in the player view; one click adds the current station to the
  library and shows a toast "Station added".
- Flag image before the country in every station card (Library and Search).
- Search by any combination of: text (matches name or genre), country, codec,
  minimum bitrate. Any single criterion is enough to run a search.

## Non-goals
- No heart in the gadget (mini) player or the bottom player toolbar.
- No flags in the station dialog, player view or tray/notifications.
- No free-text parsing of countries/codecs/bitrates out of the typed text
  (approach B/C below), no saved searches, no sort controls.
- No change to the discover (no-search) content itself.

## Anti-goals
- No confirmation dialog on the heart.
- A station already in the library must never be inserted twice
  (`queries::insert_station` would fail/duplicate).
- No flag emoji: Windows' Segoe UI Emoji renders regional indicators as letters
  ("BR"), not flags.
- No runtime download of flag images from a third-party CDN; flags are bundled.
- Filtering must happen server-side; filtering only the 1000 returned rows
  client-side would silently drop matches.

## Approaches (search) — A chosen by the user
- **A. Filter dropdowns + text matches names or genres (chosen).** Explicit,
  predictable, covers "FLAC in Brazil" with two dropdowns and no text.
- B. Single smart text box parsing codec/country/bitrate out of the words.
  Rejected: names like "Radio France" / "Jazz Brasil FM" get mis-parsed, missing
  results are hard to explain, country names would need matching in every
  translated language.
- C. A plus B as visible, undoable dropdown auto-fill. Deferred: ~2x the work,
  B's guessing problems remain; can be layered on A later.

## Design

### 1. Headings
Replace `heading` with `title-2` on both labels in `search_page.ui`. Keep
`xalign 0`, margins unchanged. (My call: `title-2` rather than `title-1`, which
is already used by the "Discover over 50,000 public stations" banner.)

### 2. Heart in the player view
- New `GtkButton` `library_button`, `circular`, `valign center`, placed between
  the play stack and the details button in `controls_box` (inherits the row's
  `has-station` sensitivity).
- Icon `heart-outline-thick-symbolic` when the station is not in the library,
  `heart-filled-symbolic` when it is. Both must be bundled as app icons (the
  Windows icon theme cannot be assumed to have them), added under
  `data/icons/hicolor/scalable/actions/` and the gresource like the existing
  `details-symbolic`.
- Click (not in library): `library.add_station(station)`, then
  `window.show_notification(&i18n("Station added"))`, icon switches to filled.
- When already in the library: filled icon, button insensitive, tooltip
  "In Library". Tooltip otherwise "Add to Library". (My call: the request only
  asks for adding; removing stays in the station dialog.)
- State must refresh when the playing station changes and when the library
  changes elsewhere (station dialog add/remove, library page delete). Builder
  picks the mechanism (e.g. `items-changed` on the library model + player
  `station` notify).
- Local and orphaned stations: they are already library stations, so the
  filled/insensitive state covers them.

### 3. Flags on station cards
- Bundle an MIT-licensed 4:3 SVG flag set (lipis/flag-icons `flags/4x3/*.svg`)
  into the gresource under `@PATH_ID@/flags/<cc>.svg`, `compressed="true"`.
  Only the two-letter ISO 3166-1 alpha-2 files; skip region and non-country
  flags (`gb-eng`, `es-ct`, `eu`, `un`, …) since lookups use `countrycode`
  only. Record the license in `NOTICE.md`.
- Size budget: ~1.5 MB of growth, measured as the compressed gresource. If
  over, minify the SVGs (svgo or equivalent, one-off, committed results);
  never drop a country's flag. (My call.)
- In `station_row.ui`, put a `GtkImage` `flag_image` (≈16×12 px) before
  `subtitle_label` in a horizontal box with 6 px spacing.
- In `set_metadata`: if `countrycode` is non-empty and
  `@PATH_ID@/flags/<lowercase cc>.svg` exists, load it with
  `gtk::Image::set_from_resource` and show it; otherwise hide the image. The
  subtitle text stays as today.
- Rows are recycled by the search `GridView`, so the flag must be reset on
  every `set_metadata` call.

### 4. Broad search
**UI** (`search_page.ui`, under the existing search entry, same `AdwClamp`
widened as needed; wraps on narrow windows):
```
[ Search stations or genres…                    ]
[Country: Any ▾]  [Codec: Any ▾]  [Bitrate: Any ▾]
```
- Placeholder text becomes "Search stations or genres".
- Country: `GtkDropDown` with search enabled, "Any" first, then countries from
  `GET /json/countries` sorted by name, shown as "Name (station count)" or just
  name (builder's call). Value sent: `countrycode` (ISO code).
- Codec: fixed list Any, MP3, AAC, AAC+, OGG, Opus, FLAC. Builder verifies the
  exact strings radio-browser stores/accepts (case-insensitive substring match
  is the API default for `codec`).
- Bitrate: Any, ≥ 64, ≥ 128, ≥ 192, ≥ 320 kbit/s → `bitrate_min`. Stations
  with unknown bitrate (stored as 0) are excluded by any "≥" choice; accepted.
  (My call.)
- The search entry becomes sensitive and the dropdowns appear only after the
  discover page has loaded (as today). The country list loads on first map with
  the discover data; if it fails, the country dropdown shows only "Any" and the
  rest of search still works.

**Behaviour**
- A search runs when the text changes (existing 750 ms delay) or any dropdown
  changes. If text is empty and every dropdown is "Any", show the discover view
  (as today).
- Add `codec: Option<String>` to `StationRequest` (serialized as `codec`; add to
  `Default`).
- With text: send two requests concurrently, both carrying the dropdown
  filters — one with `name=text`, one with `tag=text` — merge, de-duplicate by
  station UUID, sort by votes descending. Each request keeps `limit 1000`,
  `order votes`, `reverse true`. If one request fails and the other succeeds,
  show the successful results; if both fail, existing error handling.
- Without text: one request with only the dropdown filters, same order/limit.
- Stale responses: if the user changes the query while a search is in flight,
  only the latest search's results may be shown (search generation counter).
- Empty result → existing "No Results" page.

## Strings (new, translatable)
"Station added", "Add to Library", "In Library", "Search stations or genres",
"Country", "Codec", "Bitrate", "Any", "≥ {} kbit/s". Only `po/` source
extraction is required; no translations are added.

## Verification
- Unit tests: `StationRequest` URL-encodes `codec`; merge/de-dup/sort helper for
  the name+tag results; flag resource path from country code (lowercasing,
  empty code).
- Manual run on Windows: headings larger; heart adds + toast + becomes filled,
  is filled for an already-saved station, updates after removing the station
  from the library; flags show for e.g. BR/GB/US and are absent for
  stations without a code; "Brazil + FLAC" with empty text returns results;
  "jazz" returns both name and tag matches.

## Left to the builder
- Exact widget layout/CSS for the filter row and flag sizing.
- How library-state changes are observed for the heart.
- Country dropdown label format.
