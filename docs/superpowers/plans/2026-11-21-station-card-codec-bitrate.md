# Station card: codec and bitrate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show the stream's codec and bitrate (e.g. `AAC · 128 kbit/s`) on each station card in the Library and Search pages.

**Architecture:** Add a pure formatting method on `StationMetadata` (unit-testable, no GTK), add one dimmed `GtkLabel` to `station_row.ui`, and set it from `SwStationRow::set_metadata`. Data already exists in the metadata; no API/DB change.

**Tech Stack:** Rust, GTK4/libadwaita (composite templates), gettext via `crate::i18n`.

**Spec:** `docs/superpowers/specs/2026-11-21-station-card-codec-bitrate-design.md`

## Global Constraints

- Reuse the existing translatable string `"{} kbit/s"` via `i18n_f` (already used in `src/ui/station_dialog.rs:215`); add no new translatable strings.
- Separator between codec and bitrate is ` · ` (same as the subtitle line).
- Card must not change height for stations with no codec and no bitrate (label hidden).
- New label uses the same properties and `dimmed` style class as `subtitle_label`.
- Tests/clippy run from MSYS2 UCRT64 with `export CARGO_HOME=$PWD/_build/cargo-home` (see `docs/BUILDING-WINDOWS.md`).

## Review Focus

- Codec known, bitrate 0 (common for HLS/local-ish entries): show `MP3` alone, no stray separator.
- Bitrate known, codec empty: show `128 kbit/s` alone.
- Neither known (local stations): label hidden, no empty line.
- Whitespace-only codec string from the API: treated as empty.
- Station metadata updates after the row exists (`connect_metadata_notify`): label refreshes because it is set inside `set_metadata`.

---

### Task 1: Codec/bitrate line on the station card

**Files:**
- Modify: `src/api/station_metadata.rs` (add method + `#[cfg(test)] mod tests` at end of file)
- Modify: `data/gtk/station_row.ui` (new label after `subtitle_label`)
- Modify: `src/ui/station_row.rs` (new template child, set in `set_metadata`)

**Interfaces:**
- Produces: `StationMetadata::formatted_quality(&self) -> String` — empty string when neither codec nor bitrate is known.

- [ ] **Step 1: Write the failing tests** in `src/api/station_metadata.rs`, module `tests`, using `StationMetadata { codec: .., bitrate: .., ..Default::default() }`:
  - `quality_codec_and_bitrate`: `"AAC"`, `128` → `"AAC · 128 kbit/s"`
  - `quality_codec_only`: `"MP3"`, `0` → `"MP3"`
  - `quality_bitrate_only`: `""`, `96` → `"96 kbit/s"`
  - `quality_nothing_known`: `""`, `0` → `""`
  - `quality_whitespace_codec_is_empty`: `"  "`, `0` → `""`

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --locked quality_` (UCRT64 shell, `CARGO_HOME` as above)
Expected: FAIL, `formatted_quality` not found.

- [ ] **Step 3: Implement `StationMetadata::formatted_quality(&self) -> String`** in the existing `impl StationMetadata` block with `formatted_tags`. Collect trimmed non-empty `codec` and, if `bitrate > 0`, `i18n_f("{} kbit/s", &[&self.bitrate.to_string()])` (import `crate::i18n::i18n_f`); join with `" · "`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --locked quality_`
Expected: 5 passed.

- [ ] **Step 5: Add the label to `data/gtk/station_row.ui`**: a `GtkLabel` with `id="quality_label"`, copied from `subtitle_label` (same `hexpand`, `label`, `ellipsize`, `max_width_chars`, `xalign`, `dimmed` class), as the next `<child>` of the vertical `GtkBox` after `subtitle_label`.

- [ ] **Step 6: Wire it up in `src/ui/station_row.rs`**: add `#[template_child] quality_label: TemplateChild<gtk::Label>` to `SwStationRow`; at the end of `set_metadata`, `let quality = metadata.formatted_quality();` then `set_text(&quality)` and `set_visible(!quality.is_empty())`. Note `metadata.tags` is moved earlier in `set_metadata`, so compute `quality` before that move (or from a borrow) to keep it compiling.

- [ ] **Step 7: Verify build, lint, and visual result**

Run: `cargo clippy --locked && cargo test --locked`, then `.\build-aux\windows\build.ps1 -Run`
Expected: no warnings, all tests pass; in Search results and Library, cards show the third dimmed line; a local station (no codec/bitrate) shows no extra line.

- [ ] **Step 8: Commit** (only when the user asks to commit)

```bash
git add src/api/station_metadata.rs src/ui/station_row.rs data/gtk/station_row.ui
git commit -m "feat(ui): show codec and bitrate on station cards"
```
