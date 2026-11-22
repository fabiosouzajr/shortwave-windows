# Browse headings, player heart, country flags, broad search — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Larger Browse section headings, a one-click "add to library" heart in the player view, bundled flags on station cards, and search by any mix of text (name or genre), country, codec and minimum bitrate.

**Architecture:** Pure helpers (request encoding, result merging, flag path) live next to the code that uses them and carry the unit tests; UI is GTK Builder `.ui` + Rust template children as elsewhere in `src/ui`. Search filtering is server-side via radio-browser `/json/stations/search`; text runs a `name` and a `tag` request concurrently and merges them.

**Tech Stack:** Rust, gtk4-rs, libadwaita, reqwest, serde_urlencoded, futures-util, GResource, MSYS2 UCRT64 on Windows.

**Spec:** `docs/superpowers/specs/2026-11-22-browse-search-player-polish-design.md`

## Global Constraints

- Tests/clippy run from MSYS2 UCRT64 with `export CARGO_HOME=$PWD/_build/cargo-home` (see `docs/BUILDING-WINDOWS.md`); full app via `.\build-aux\windows\build.ps1 -Run`.
- No flag emoji; no runtime download of flags; flags bundled from lipis/flag-icons `flags/4x3/`, two-letter ISO codes only, MIT license recorded in `NOTICE.md`.
- Flag growth budget ~1.5 MB measured as compressed gresource; if over, minify SVGs, never drop a country.
- No confirmation dialog on the heart; never insert a library station twice.
- All filtering server-side; each request `limit 1000`, `order votes`, `reverse true`.
- New user-visible strings go through `i18n`/`translatable="yes"`; exact strings: "Station added", "Add to Library", "In Library", "Search stations or genres", "Country", "Codec", "Bitrate", "Any", "≥ {} kbit/s".
- Leave the pre-existing uncommitted change in `src/main.rs` alone; never `git add` it.

## Review Focus

1. Codec "AAC+" in the URL → must be sent as `codec=AAC%2B`, not a space (test in Task 2).
2. The same station matched by name and by genre → listed once (test in Task 2).
3. Country codes in odd case/whitespace (`"br"`, `" BR "`) or with no bundled flag (`"ZZ"`, `""`) → flag found / image hidden, no GTK warning (tests in Task 3).
4. Typing fast or changing a dropdown while a search is in flight → only the newest search's results appear (manual check in Task 5).
5. Station removed from the library elsewhere while playing → heart returns to outline and becomes clickable again (manual check in Task 4).

---

### Task 1: Larger Browse headings

**Files:**
- Modify: `data/gtk/search_page.ui:89-93`, `:111-115`

**Interfaces:** none.

- [ ] **Step 1:** Replace `<class name="heading"/>` with `<class name="title-2"/>` on the "Popular Stations" and "Discover New Stations" labels only.
- [ ] **Step 2: Verify** — `.\build-aux\windows\build.ps1 -Run`; Browse page headings render clearly larger than card names (16 px) and smaller than the banner.
- [ ] **Step 3: Commit** — `git add data/gtk/search_page.ui && git commit -m "feat(ui): larger section headings on the browse page"`

---

### Task 2: Search API — codec field, countries endpoint, merged name+genre search

**Files:**
- Modify: `src/api/station_request.rs`, `src/api/client.rs`, `src/api/mod.rs`
- Create: `src/api/country.rs`
- Test: inline `#[cfg(test)]` in `station_request.rs` and `client.rs`

**Interfaces:**
- Produces:
  - `StationRequest.codec: Option<String>` (serialized `codec`, `None` in `Default`).
  - `StationRequest::filtered(countrycode: Option<String>, codec: Option<String>, bitrate_min: Option<u32>) -> StationRequest` — sets those plus `limit 1000`, `order "votes"`, `reverse true`.
  - `pub struct Country { pub name: String, pub iso_3166_1: String, pub stationcount: u32 }` (`Deserialize, Debug, Clone`), re-exported from `crate::api`.
  - `client::countries() -> Result<Vec<Country>, Error>` — GET `json/countries` (new `static COUNTRIES` in `api/mod.rs`), drops entries with empty `iso_3166_1`, sorted by `name` (case-insensitive).
  - `client::search_stations(text: Option<String>, filters: StationRequest) -> Result<Vec<SwStation>, Error>` — `None` text: one request with `filters`. `Some(text)`: `futures_util::future::join` of `filters` + `name=text` and `filters` + `tag=text`; Ok/Ok → merge; one Ok → that one; Err/Err → the first error.
  - `fn merge_by_votes(a: Vec<StationMetadata>, b: Vec<StationMetadata>) -> Vec<StationMetadata>` (private in `client.rs`) — de-dup by `stationuuid` (first wins), stable sort by `votes` descending.
  - Refactor `station_request` internally to share a `fn request_metadata(request: StationRequest) -> Result<Vec<StationMetadata>, Error>` helper; `station_request`'s signature is unchanged.

- [ ] **Step 1: Write failing tests**

```rust
// station_request.rs
#[test]
fn codec_is_url_encoded() {
    let r = StationRequest { codec: Some("AAC+".into()), ..Default::default() };
    assert!(r.url_encode().contains("codec=AAC%2B"));
}
#[test]
fn filtered_sets_votes_order_and_limit() {
    let r = StationRequest::filtered(Some("BR".into()), Some("FLAC".into()), Some(128));
    let q = r.url_encode();
    for part in ["countrycode=BR", "codec=FLAC", "bitrate_min=128", "limit=1000", "order=votes", "reverse=true"] {
        assert!(q.contains(part), "{q} missing {part}");
    }
}

// client.rs
fn md(uuid: &str, votes: i32) -> StationMetadata {
    StationMetadata { stationuuid: uuid.into(), votes, ..Default::default() }
}
#[test]
fn merge_dedups_and_sorts_by_votes() {
    let out = merge_by_votes(vec![md("a", 5), md("b", 50)], vec![md("b", 50), md("c", 20)]);
    let ids: Vec<_> = out.iter().map(|m| m.stationuuid.as_str()).collect();
    assert_eq!(ids, ["b", "c", "a"]);
}
```

- [ ] **Step 2: Run** `cargo test --locked codec_is_url_encoded filtered_sets merge_dedups` — Expected: compile FAIL (no `codec` / `filtered` / `merge_by_votes`).
- [ ] **Step 3: Implement** the Produces list above.
- [ ] **Step 4: Run** `cargo test --locked` — Expected: all PASS.
- [ ] **Step 5: Commit** — `git add src/api && git commit -m "feat(api): codec filter, country list and merged name/genre search"`

---

### Task 3: Bundled flags on station cards

**Files:**
- Create: `data/flags/<cc>.svg` (two-letter files from flag-icons `flags/4x3/`, pinned release tag recorded in `NOTICE.md`)
- Modify: `data/de.haeckerfelix.Shortwave.gresource.xml.in` (new `<gresource prefix="@PATH_ID@/flags/">`, one `<file compressed="true" alias="<cc>.svg">flags/<cc>.svg</file>` per flag), `NOTICE.md`, `data/gtk/station_row.ui`, `src/ui/station_row.rs`
- Test: inline tests in `station_row.rs`

**Interfaces:**
- Produces: `fn flag_resource_path(countrycode: &str) -> Option<String>` in `station_row.rs` — trims, lowercases; `None` unless exactly two ASCII letters; returns `format!("{}/flags/{cc}.svg", config::PATH_ID)`. Existence is checked separately in `set_metadata` with `gio::resources_get_info(path, ResourceLookupFlags::NONE).is_ok()`.

- [ ] **Step 1:** Download the pinned flag-icons release into a scratch dir, copy only `flags/4x3/[a-z][a-z].svg` into `data/flags/`, generate the gresource entries with a one-off script (not committed). Add an MIT notice block for flag-icons to `NOTICE.md`.
- [ ] **Step 2: Measure** — build, then compare compiled gresource size before/after (`ls -l _build/data/*.gresource`). If growth > 1.5 MB, run `svgo` over `data/flags/` once and commit the minified files. Report the number in the commit message.
- [ ] **Step 3: Write failing tests**

```rust
#[test]
fn flag_path_normalises_code() {
    assert_eq!(flag_resource_path(" BR ").unwrap(), flag_resource_path("br").unwrap());
    assert!(flag_resource_path("br").unwrap().ends_with("/flags/br.svg"));
}
#[test]
fn flag_path_rejects_bad_codes() {
    assert_eq!(flag_resource_path(""), None);
    assert_eq!(flag_resource_path("GBR"), None);
    assert_eq!(flag_resource_path("1A"), None);
}
```

- [ ] **Step 4: Run** `cargo test --locked flag_path` — Expected: FAIL (fn missing).
- [ ] **Step 5: Implement** `flag_resource_path`; in `station_row.ui` wrap `subtitle_label` in a horizontal `GtkBox` (spacing 6) with a leading `GtkImage id="flag_image"` (`pixel_size` 16, `valign center`, `visible False`); in `set_metadata` set from resource and show when the path exists, else `clear()` and hide — on every call (rows are recycled). Hide the whole subtitle box when both flag and text are hidden.
- [ ] **Step 6: Run** `cargo test --locked` — PASS. Then `build.ps1 -Run`: BR/GB/US stations show flags in Library and Search; a local station without a country shows none; no `Gtk-WARNING` about missing resources in the console.
- [ ] **Step 7: Commit** — `git add data/flags data/de.haeckerfelix.Shortwave.gresource.xml.in NOTICE.md data/gtk/station_row.ui src/ui/station_row.rs && git commit -m "feat(ui): country flags on station cards"`

---

### Task 4: Heart button in the player view

**Files:**
- Create: `data/icons/hicolor/scalable/actions/heart-outline-thick-symbolic.svg`, `heart-filled-symbolic.svg` (from GNOME icon-development-kit / Adwaita, CC0/CC-BY-SA as the existing actions icons — match their source and note in `NOTICE.md` if different)
- Modify: `data/de.haeckerfelix.Shortwave.gresource.xml.in` (actions prefix), `data/gtk/player_view.ui:213`, `src/ui/player/player_view.rs`

**Interfaces:**
- Consumes: `SwApplication::default().library()` → `add_station(SwStation)`, `contains_station(&SwStation) -> bool`, `model()` (`SwStationModel`, a `gio::ListModel`); `SwApplication::default().application_window().show_notification(&str)`; player `connect_station_notify`.
- Produces: template child `library_button: TemplateChild<gtk::Button>`, template callback `library_button_clicked`, private `fn update_library_button(&self)`.

- [ ] **Step 1:** Add the two icons and gresource entries. In `player_view.ui` insert `GtkButton id="library_button"` (`circular`, `valign center`, `clicked` → `library_button_clicked` swapped) between `playback_button_stack` and `details_button`.
- [ ] **Step 2: Implement** `update_library_button`: station in library → `heart-filled-symbolic`, insensitive, tooltip `i18n("In Library")`; otherwise `heart-outline-thick-symbolic`, sensitive, tooltip `i18n("Add to Library")`. Call it from `constructed`, on player `station` notify, and on library model `items-changed`.
- [ ] **Step 3: Implement** `library_button_clicked`: if a station is playing and not in the library → `add_station(station.clone())`, `show_notification(&i18n("Station added"))`. (Icon update follows from `items-changed`.)
- [ ] **Step 4: Verify** — `cargo clippy --locked && cargo test --locked`, then `build.ps1 -Run`:
  - play an unsaved station from Browse → outline heart; click → toast "Station added", heart filled/insensitive, station on Library page;
  - play a saved station → filled from the start;
  - **Review Focus 5:** with it playing, delete it on the Library page → heart outline and clickable again.
- [ ] **Step 5: Commit** — `git add data/icons data/de.haeckerfelix.Shortwave.gresource.xml.in data/gtk/player_view.ui src/ui/player/player_view.rs NOTICE.md && git commit -m "feat(player): heart button adds the playing station to the library"`

---

### Task 5: Search filters UI and behaviour

**Files:**
- Modify: `data/gtk/search_page.ui:10-31`, `src/ui/pages/search_page.rs`, `data/gtk/style.css` (only if the filter row needs spacing tweaks)

**Interfaces:**
- Consumes (Task 2): `StationRequest::filtered`, `client::countries`, `client::search_stations`, `Country`.
- Produces: template children `country_dropdown`, `codec_dropdown`, `bitrate_dropdown: TemplateChild<gtk::DropDown>`, `filter_box: TemplateChild<gtk::Box>`; fields `countries: RefCell<Vec<Country>>`, `search_generation: Cell<u64>`.

- [ ] **Step 1: UI** — below the search entry (same clamp, raise `maximum_size` to ~600) add `filter_box` (a `GtkFlowBox` or wrapping `GtkBox`, `visible False` until loaded) holding three `GtkDropDown`s with tooltips "Country", "Codec", "Bitrate". Codec model: `GtkStringList` Any, MP3, AAC, AAC+, OGG, Opus, FLAC. Bitrate model: Any, then `i18n_f("≥ {} kbit/s", [64|128|192|320])` built in Rust. Country dropdown: `enable_search true`, model built in Rust. Placeholder text → "Search stations or genres". Make "Any" translatable.
- [ ] **Step 2: Load countries** — in `refresh_discover_page` after discover loads, `client::countries().await`; on Ok fill the country model ("Any" + names), store `countries`; on Err log a warning and keep "Any" only. Show `filter_box` together with making the entry sensitive.
- [ ] **Step 3: Search** — connect each dropdown's `selected` notify to the same async search as `search_changed`. Build filters from selections (index 0 → `None`; country → `countries[i-1].iso_3166_1`; codec → the label; bitrate → `[64,128,192,320][i-1]`). If text empty and all `None` → discover view (as today). Else increment `search_generation`, remember it, show spinner, `client::search_stations(text, StationRequest::filtered(..)).await`, and **discard the result if the generation changed meanwhile**. Then existing results / no-results / `handle_error` handling. Remove the now-unused `StationRequest::search_for_name` if nothing else uses it.
- [ ] **Step 4: Verify** — `cargo clippy --locked && cargo test --locked`, then `build.ps1 -Run`:
  - empty text, Country Brazil + Codec FLAC → results, all BR/FLAC;
  - "jazz" → includes stations named jazz and stations tagged jazz without it in the name;
  - Bitrate ≥ 320 + "rock" → every card shows ≥ 320 kbit/s;
  - reset all to Any and clear text → discover view;
  - **Review Focus 4:** type "a", then quickly "ambient" while Country is set → final grid matches "ambient" only;
  - Country dropdown search: type "bra" → Brazil selectable.
- [ ] **Step 5: Commit** — `git add data/gtk/search_page.ui src/ui/pages/search_page.rs data/gtk/style.css && git commit -m "feat(search): filter by country, codec and bitrate; text matches names and genres"`

---

### Task 6: Strings and final check

**Files:**
- Modify: `po/POTFILES.in` only if a new file with translatable strings was created (none expected; `country.rs` has none).

- [ ] **Step 1:** `grep -n "player_view\|station_row\|search_page" po/POTFILES.in` — all three `.ui` and `.rs` files already listed; confirm.
- [ ] **Step 2:** `cargo clippy --locked -- -D warnings && cargo test --locked` — Expected: clean, all PASS.
- [ ] **Step 3:** `git status` — only intended files changed; `src/main.rs` still carries its pre-existing unstaged change and was not committed.
