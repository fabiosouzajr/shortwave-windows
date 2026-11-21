# Station card: codec and bitrate

## Intent
While browsing stations (Library and Search pages), each station card should show the stream's codec and bitrate, so audio quality is visible without opening the station dialog.

## Goals
- `SwStationRow` shows e.g. `AAC · 128 kbit/s` on the card.
- Works on both Library and Search pages (both use `SwStationRow`).

## Non-goals
- No sorting/filtering changes (sorter already supports Codec/Bitrate).
- No API, database or metadata changes: `StationMetadata.codec` / `.bitrate` already exist.
- No change to the station dialog.

## Anti-goals
- Cards must not grow taller or look different for stations with no codec/bitrate data (e.g. local stations).
- No new translatable strings if avoidable.

## Constraints
- Card is 300px wide minimum; the existing subtitle already ellipsizes, so quality info must not be appended to it.
- Reuse the existing `i18n_f("{} kbit/s", ...)` string from `station_dialog.rs`.

## Approaches
1. **Separate dimmed line under the subtitle (chosen).** Never truncated by long tag lists; hidden when empty. Costs one extra line of height on cards that have data.
2. Append to the subtitle line. Rejected: first thing ellipsized at card width.
3. Trailing chip/badge next to the play button. Rejected: competes with local/orphaned icons and play button for horizontal space.

## Design (decided)
- `data/gtk/station_row.ui`: add a `GtkLabel` (`quality_label`) after `subtitle_label`, same props and `dimmed` class.
- `src/ui/station_row.rs`: in `set_metadata`, build text from `codec` (if non-empty) and `bitrate` (if != 0) joined with ` · `; set text and visibility (hidden if empty).
- Codec shown as provided by the API (no case normalization beyond what the dialog does, i.e. none).

## Left to the builder
- Exact label spacing/tooltip, and verifying visually on Windows build.
