# Station card: logos, name and quality line

## Intent
Station cards (Library and Search pages, both `SwStationRow`) look bare: most
logos are missing, the ones that load are shown small, the name doesn't stand
out, and `AAC · 128 kbit/s` is a plain dimmed line. Make the card easier to
scan and the station's identity and stream quality visible at a glance.

Measured on 2026-11-21 (radio-browser, 300 random stations): 41% have no
`favicon`, ~20% of the rest are dead links or HTML; 112 of the 122 without a
favicon have a `homepage`. Decoding is not the problem: the Windows bundle
ships loaders for PNG/JPEG/ICO/WebP/SVG/GIF. Most favicons that do load are
>128 px, so "too small" is the 64 px card slot, not the images.

## Goals
- Recover logos from the station's homepage when `favicon` is missing or fails.
- Bigger logo, bolder name, codec icon + bitrate meter on the card.

## Non-goals
- No change to cover sizes outside `SwStationRow` (player toolbar 64 px,
  dialogs/player view 192 px).
- No database/schema or API changes; no change to the station dialog.

## Anti-goals
- No third-party favicon service: station domains are not sent anywhere but
  the station's own site.
- No homepage re-fetch on every launch for sites that have no icon.
- Cards with no codec/bitrate (local stations) must not show an empty line.

## Approaches (logo fallback)
1. **Fallback inside `CoverLoader`, regex-scan the homepage HTML (chosen).**
   One file plus passing the homepage through; uses the existing `regex` dep.
   Misses odd markup; acceptable for best effort.
2. Same with a real HTML parser (`scraper`). Rejected: heavy html5ever tree in
   the Windows bundle for a few `<link>` tags.
3. Resolve once, store the logo URL in the station DB. Rejected: schema change
   and fights the periodic metadata refresh.

Card look was chosen from browser mockups: direction B ("quality meter") with
A's logo size and name style.

## Design (decided)
**Logo lookup** (`src/api/cover_loader.rs`, `src/ui/station_cover.rs`)
- `SwStationCover` passes both `favicon` and `homepage` to the loader. It now
  also tries when only `homepage` is set.
- Order: `favicon` → homepage candidates → `<homepage origin>/favicon.ico`.
  First one that downloads *and* decodes wins.
- Homepage candidates, extracted with `regex` from the first ~512 KB of the
  HTML: `<link rel="apple-touch-icon…">`, `<link rel="icon"|"shortcut icon">`
  (prefer the largest `sizes`), `<meta property="og:image">`. Resolved against
  the final (post-redirect) page URL. Largest/apple-touch first, `og:image`
  last.
- Rendered result cached as today; homepage-derived results cached under a
  homepage-based key. A "no logo found" marker is cached for 7 days so
  icon-less sites aren't re-fetched each launch.

**Card** (`data/gtk/station_row.ui`, `src/ui/station_row.rs`, `style.css`)
- Logo 88 px (was 64). Card `width_request` 300 → 340.
- Name: bold, ~16 px (CSS class on `station_label`), still ellipsized.
- Quality line: `[codec icon] CODEC │ ▂▄▆█ 128 kbit/s`
  - Codec icon: two bundled symbolic icons, a music note for lossy codecs and
    a distinct mark for lossless (FLAC, ALAC, WAV/PCM). Codec name in bold.
  - Bitrate meter: 4 bars lit at ≥64, ≥128, ≥192, ≥320 kbit/s. Number in a
    monospace font with tabular digits; unit dimmed and smaller.
  - Codec part hidden when codec is empty (or `UNKNOWN`); bitrate part hidden
    when bitrate is 0; whole line hidden when both are absent.

## Left to the builder
- Exact regexes, the bars widget (CSS-styled boxes suggested), icon artwork,
  colours (follow Adwaita accent/dimmed), and whether
  `StationMetadata::formatted_quality()` is replaced or kept (remove it and its
  tests if it becomes unused).
- Unit text: reuse the existing translatable "kbit/s" wording.
