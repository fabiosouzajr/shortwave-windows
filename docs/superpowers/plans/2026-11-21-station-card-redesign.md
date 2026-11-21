# Station Card Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recover missing station logos from the station homepage, and restyle `SwStationRow` (88 px logo, bold name, codec icon + bitrate meter).

**Architecture:** A new pure module `src/api/logo_discovery.rs` extracts icon URLs from homepage HTML. `CoverLoader` gains `load_station_cover()`, which tries favicon → homepage candidates → `/favicon.ico` and caches hits and a 7‑day "no logo" marker. `SwStationCover` calls it, so every station cover (cards, player) benefits. The card's quality line is rebuilt from small `StationMetadata` helpers and CSS.

**Tech Stack:** Rust, gtk4-rs/libadwaita, reqwest, cacache, regex (existing deps only).

**Spec:** `docs/superpowers/specs/2026-11-21-station-card-redesign-design.md`

## Global Constraints

- No new crates. Use `regex`, `url`, `reqwest`, `cacache` already in `Cargo.toml`.
- No third-party favicon service; only the station's own favicon/homepage hosts are contacted.
- Cover sizes outside `station_row.ui` unchanged (player toolbar 64, dialogs/player view 192).
- No DB schema or API changes.
- "No logo" marker TTL: 7 days. Homepage read cap: 512 KB.
- Bitrate bar thresholds: ≥64, ≥128, ≥192, ≥320 kbit/s.
- Lossless codecs: FLAC, ALAC, WAV, PCM (case-insensitive). `UNKNOWN` codec is treated as empty.
- Unit text reuses the existing translatable wording `kbit/s`.
- Tests/clippy run in the MSYS2 UCRT64 shell with `export CARGO_HOME=$PWD/_build/cargo-home` (see `docs/BUILDING-WINDOWS.md`). App run: `.\build-aux\windows\build.ps1 -Run`.

## Review Focus

1. `href` written as protocol-relative (`//cdn.x/logo.png`) or root-relative (`/logo.png`): it must resolve against the final page URL, not be dropped. (Task 1 test)
2. Attribute order and quoting vary: `<link href='x' rel="icon">`, uppercase `<LINK REL=...>`, and `&amp;` in the URL. All must be found. (Task 1 test)
3. `data:` URIs and `.svg` mask-icons (`rel="mask-icon"`) are not useful logos and must be skipped. (Task 1 test)
4. A homepage that redirects, returns 404, or isn't HTML: the loader must fall through to `/favicon.ico`, then the "no logo" marker, never panic or hang the queue. (Task 2 manual check)
5. Codec strings like `aac+`, ` MP3 `, `UNKNOWN`, and a bitrate of 0 or negative: the card shows no empty icon, bars, or stray separator. (Task 3 test)

---

### Task 1: Homepage icon discovery

**Files:**
- Create: `src/api/logo_discovery.rs`
- Modify: `src/api/mod.rs` (add `mod logo_discovery;`, `pub use logo_discovery::{icon_candidates, root_favicon};`)

**Interfaces:**
- Produces: `pub fn icon_candidates(html: &str, page_url: &Url) -> Vec<Url>`, ordered best-first, deduplicated, http(s) only.
- Produces: `pub fn root_favicon(page_url: &Url) -> Option<Url>`, which returns `<scheme>://<host>[:port]/favicon.ico`.

Ordering rule: `apple-touch-icon` / `apple-touch-icon-precomposed` first, then `icon` / `shortcut icon` sorted by the largest `sizes` value descending (`any` counts as largest, missing `sizes` counts as 0), then `og:image` (`<meta property="og:image" content=...>`). Tags are matched case-insensitively with attributes in any order and either quote style (or unquoted). Decode `&amp;` in URLs. Skip `data:` and `rel="mask-icon"`.

- [ ] **Step 1: Write the failing tests** (in a `#[cfg(test)] mod tests` in the new file)

```rust
fn page() -> Url { Url::parse("https://radio.example/live/index.html").unwrap() }

#[test] fn candidates_order_touch_then_largest_icon_then_og() {
    let html = r#"<link rel="icon" sizes="32x32" href="/i32.png">
      <meta property="og:image" content="https://cdn.example/og.jpg">
      <link rel="icon" sizes="192x192" href="i192.png">
      <link rel="apple-touch-icon" href="/touch.png">"#;
    let got: Vec<String> = icon_candidates(html, &page()).iter().map(|u| u.to_string()).collect();
    assert_eq!(got, vec!["https://radio.example/touch.png",
        "https://radio.example/live/i192.png", "https://radio.example/i32.png",
        "https://cdn.example/og.jpg"]);
}
#[test] fn candidates_any_attribute_order_and_quotes() {
    let html = "<LINK HREF='//cdn.example/a.png?x=1&amp;y=2' REL='Shortcut Icon'>";
    assert_eq!(icon_candidates(html, &page())[0].as_str(), "https://cdn.example/a.png?x=1&y=2");
}
#[test] fn candidates_skip_data_and_mask_icon() {
    let html = r#"<link rel="icon" href="data:image/png;base64,AAA"><link rel="mask-icon" href="/m.svg">"#;
    assert!(icon_candidates(html, &page()).is_empty());
}
#[test] fn candidates_deduplicated() {
    let html = r#"<link rel="icon" href="/a.png"><link rel="shortcut icon" href="/a.png">"#;
    assert_eq!(icon_candidates(html, &page()).len(), 1);
}
#[test] fn root_favicon_keeps_scheme_host_port() {
    let u = Url::parse("http://radio.example:8080/a/b?q").unwrap();
    assert_eq!(root_favicon(&u).unwrap().as_str(), "http://radio.example:8080/favicon.ico");
}
```

- [ ] **Step 2: Run to verify failure**
Run: `cargo test --locked candidates_ root_favicon`
Expected: compile error, `icon_candidates` not found.

- [ ] **Step 3: Implement both functions** with `regex` statics (`LazyLock<Regex>`): one regex for `<link …>`/`<meta …>` tags, one for `name=value` attribute pairs. Resolve with `page_url.join(href)`.

- [ ] **Step 4: Run tests**
Run: `cargo test --locked candidates_ root_favicon`
Expected: 5 passed.

- [ ] **Step 5: Commit**
`git commit -m "feat(api): discover station logos from homepage HTML"`

---

### Task 2: Cover loader fallback chain

**Files:**
- Modify: `src/api/cover_loader.rs`
- Modify: `src/api/client.rs:36` (make `USER_AGENT` `pub(crate)`)
- Modify: `src/ui/station_cover.rs:229-283` (`load_cover`)

**Interfaces:**
- Consumes: `icon_candidates`, `root_favicon` (Task 1).
- Produces: `pub async fn load_station_cover(&mut self, favicon: Option<&Url>, homepage: Option<&Url>, size: i32, cancellable: gio::Cancellable) -> Result<gdk::Texture>` on `CoverLoader`. The existing `load_cover(&Url, …)` keeps its signature (used by `platform/windows/smtc.rs`) and becomes a thin call to `load_station_cover(Some(url), None, …)`.
- Produces: `fn no_logo_marker_fresh(age: Duration) -> bool` (true when `age < 7 days`).

Behaviour:
- Cache key = `"{favicon or homepage}@{size}"` (favicon wins when present, so existing cache entries stay valid).
- Order: cached texture → if a marker `"nologo:{key}"` exists and is fresh, return `Err("no logo")` → favicon → (if homepage) GET homepage, read at most 512 KB, `icon_candidates(body, final_response_url)` → `root_favicon(homepage)`. First URL that downloads with a success status **and** decodes wins and is rendered and cached under the key. If all fail and a homepage was tried, write the marker (empty data). Read the marker's age from `cacache::metadata`.
- `HTTP_CLIENT` sets `.user_agent(USER_AGENT)` and the downloads call `error_for_status()`.
- Each download attempt needs its own temp file (the current single `tmp_file`/`tmp_stream` per request cannot be reused across attempts). Create one per attempt and delete it afterwards.
- `SwStationCover::load_cover`: the `else if let Some(favicon_url)` branch becomes "favicon or homepage present", and calls `load_station_cover(metadata.favicon.as_ref(), metadata.homepage.as_ref(), …)`. Don't log a warning for the `"no logo"` error. On failure, show the `fallback` (initials) page.

- [ ] **Step 1: Write the failing test**
```rust
#[test] fn no_logo_marker_ttl() {
    assert!(no_logo_marker_fresh(Duration::from_secs(86400 * 6)));
    assert!(!no_logo_marker_fresh(Duration::from_secs(86400 * 7)));
}
```
- [ ] **Step 2: Run** `cargo test --locked no_logo_marker`. Expected: FAIL (not found).
- [ ] **Step 3: Implement** the behaviour above.
- [ ] **Step 4: Verify**
Run: `cargo clippy --locked && cargo test --locked`. Expected: no warnings, all pass.
Then run `.\build-aux\windows\build.ps1 -Run` with `RUST_LOG=shortwave=debug`. Search for a station known to have an empty favicon but a homepage, and check that a logo appears (look one up first: `https://de1.api.radio-browser.info/json/stations/search?limit=50&order=random` and pick one where `favicon` is `""` and `homepage` is set). Restart the app: the logo shows instantly from cache. A station whose homepage 404s shows initials, and no homepage request appears on the second launch.
- [ ] **Step 5: Commit**
`git commit -m "feat(covers): fall back to homepage icons when favicon is missing"`

---

### Task 3: Quality helpers on StationMetadata

**Files:**
- Modify: `src/api/station_metadata.rs:83-97` and its tests (`quality_*`)

**Interfaces:**
- Produces: `pub fn display_codec(&self) -> Option<String>`, which is the trimmed, uppercased codec, or `None` if empty or `UNKNOWN`.
- Produces: `pub fn is_lossless(&self) -> bool`, true for FLAC/ALAC/WAV/PCM.
- Produces: `pub fn bitrate_level(&self) -> u8`, 0–4 lit bars (0 when bitrate ≤ 0 or < 64).
- Removes: `formatted_quality()` and its 5 tests (unused after Task 4).

- [ ] **Step 1: Write failing tests** (replace the `quality_*` tests; keep the `quality(codec, bitrate)` helper)
```rust
#[test] fn codec_display() {
    assert_eq!(quality(" aac+ ", 0).display_codec().as_deref(), Some("AAC+"));
    assert_eq!(quality("UNKNOWN", 0).display_codec(), None);
    assert_eq!(quality("  ", 0).display_codec(), None);
}
#[test] fn codec_lossless() {
    assert!(quality("flac", 0).is_lossless());
    assert!(!quality("MP3", 0).is_lossless());
}
#[test] fn bitrate_levels() {
    for (b, l) in [(-1, 0), (0, 0), (63, 0), (64, 1), (128, 2), (191, 2), (192, 3), (320, 4), (1411, 4)] {
        assert_eq!(quality("", b).bitrate_level(), l, "bitrate {b}");
    }
}
```
- [ ] **Step 2: Run** `cargo test --locked codec_ bitrate_levels`. Expected: FAIL (not found).
- [ ] **Step 3: Implement** the three methods and remove `formatted_quality`. Leave `station_row.rs` compiling by switching it to the new helpers minimally; Task 4 builds the full UI.
- [ ] **Step 4: Run** `cargo test --locked`. Expected: all pass.
- [ ] **Step 5: Commit**
`git commit -m "refactor(api): codec and bitrate helpers for the station card"`

---

### Task 4: Card restyle

**Files:**
- Create: `data/icons/hicolor/scalable/status/codec-lossy-symbolic.svg` (music note), `data/icons/hicolor/scalable/status/codec-lossless-symbolic.svg` (a distinct mark, e.g. a diamond or note with a waveform). 16×16, single colour `#2e3436`, matching the existing symbolic icons.
- Modify: `data/de.haeckerfelix.Shortwave.gresource.xml.in:28-31` (add both icons to the `icons/16x16/status/` prefix)
- Modify: `data/gtk/station_row.ui`
- Modify: `src/ui/station_row.rs`
- Modify: `data/gtk/style.css` (append)

**Interfaces:**
- Consumes: `display_codec()`, `is_lossless()`, `bitrate_level()` (Task 3).

Layout (replace `quality_label` with a horizontal `GtkBox` `quality_box`, spacing 6, CSS class `station-quality`):
`[codec_icon GtkImage] [codec_label bold] [separator label "│" dimmed] [bars_box: 4 GtkBox children, class "bar", heights 4/7/9/11 px, valign end] [bitrate_label .numeric monospace] [unit_label "kbit/s" dimmed, smaller]`

- `station_row.ui`: cover `size` 64 → 88. Template `width_request` 300 → 340. `station_label` gets class `station-name`.
- CSS: `.station-name { font-weight: 700; font-size: 16px; }`. `.station-quality .bar { min-width: 3px; border-radius: 1px; background: alpha(currentColor, .15); }`. `.station-quality .bar.lit { background: var(--accent-color); }`. `.station-quality .bitrate { font-family: monospace; }`. `.station-quality .unit { font-size: smaller; }`.
- `station_row.rs` `set_metadata`: codec icon + label visible only when `display_codec()` is `Some`. Icon name `codec-lossless-symbolic` if `is_lossless()`, else `codec-lossy-symbolic`. Bars, bitrate and unit visible only when `bitrate > 0`, with `bars[i]` getting `lit` when `i < bitrate_level()`. Separator visible only when both parts are visible. `quality_box` visible when either part is. Tooltip on `bars_box`: the bitrate text. Unit via `i18n("kbit/s")`; if that string isn't in `po/`, use `i18n_f("{} kbit/s", …)` and split off the number instead of adding a new string.

- [ ] **Step 1: Implement** the UI, CSS, icons and row code above.
- [ ] **Step 2: Verify**
Run: `cargo clippy --locked && cargo test --locked`, then `.\build-aux\windows\build.ps1 -Run`.
Expected: no warnings, all tests pass. In Search and Library: 88 px logos, a bold 16 px name, `♪ MP3 │ ▂▄░░ 128 kbit/s`. A FLAC station shows the lossless icon and 4 lit bars. A local station (no codec/bitrate) has no quality line. A station with only a codec shows no separator or bars. Check in both light and dark mode.
- [ ] **Step 3: Commit**
`git commit -m "feat(ui): bigger logos, bold name and codec/bitrate meter on station cards"`
