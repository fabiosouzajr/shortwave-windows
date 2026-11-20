# 07 — UI inventory

All templates are plain GtkBuilder XML in [data/gtk/](../../data/gtk/) (no
Blueprint), bundled into `<APP_ID>.gresource` by
[data/de.haeckerfelix.Shortwave.gresource.xml.in](../../data/de.haeckerfelix.Shortwave.gresource.xml.in)
and loaded through `#[template(resource = "/de/haeckerfelix/Shortwave/gtk/…")]`.
Line numbers are 1-based against the `windows-port` baseline.

## Templates → widgets

| Template | Rust widget | Parent | Template attr |
|---|---|---|---|
| `window.ui` | `SwApplicationWindow` | `adw::ApplicationWindow` | [src/ui/window.rs:39](../../src/ui/window.rs#L39) |
| `library_page.ui` | `SwLibraryPage` | `adw::NavigationPage` | [src/ui/pages/library_page.rs:36](../../src/ui/pages/library_page.rs#L36) |
| `search_page.ui` | `SwSearchPage` | `adw::NavigationPage` | [src/ui/pages/search_page.rs:32](../../src/ui/pages/search_page.rs#L32) |
| `player_view.ui` | `SwPlayerView` (sidebar / sheet content) | `adw::Bin` | [src/ui/player/player_view.rs:36](../../src/ui/player/player_view.rs#L36) |
| `player_toolbar.ui` | `SwPlayerToolbar` (bottom bar in narrow layout) | `adw::Bin` | [src/ui/player/player_toolbar.rs:32](../../src/ui/player/player_toolbar.rs#L32) |
| `player_gadget.ui` | `SwPlayerGadget` (mini player) | `adw::Bin` | [src/ui/player/player_gadget.rs:32](../../src/ui/player/player_gadget.rs#L32) |
| `station_row.ui` | `SwStationRow` | `adw::Bin` | [src/ui/station_row.rs:36](../../src/ui/station_row.rs#L36) |
| `station_cover.ui` | `SwStationCover` | `adw::Bin` | [src/ui/station_cover.rs:35](../../src/ui/station_cover.rs#L35) |
| `station_dialog.ui` | `SwStationDialog` (details + **libshumate map**) | `adw::Dialog` | [src/ui/station_dialog.rs:35](../../src/ui/station_dialog.rs#L35) |
| `add_station_dialog.ui` | `SwAddStationDialog` (local station: name, URL, cover) | `adw::Dialog` | [src/ui/add_station_dialog.rs:33](../../src/ui/add_station_dialog.rs#L33) |
| `track_dialog.ui` | `SwTrackDialog` | `adw::Dialog` | [src/ui/track_dialog.rs:33](../../src/ui/track_dialog.rs#L33) |
| `track_row.ui` | `SwTrackRow` | `adw::ActionRow` | [src/ui/track_row.rs:35](../../src/ui/track_row.rs#L35) |
| `recording_indicator.ui` | `SwRecordingIndicator` | `gtk::Button` | [src/ui/recording_indicator.rs:31](../../src/ui/recording_indicator.rs#L31) |
| `volume_control.ui` | `SwVolumeControl` | `gtk::Widget` | [src/ui/volume_control.rs:32](../../src/ui/volume_control.rs#L32) |
| `device_dialog.ui` | `SwDeviceDialog` (Cast picker) | `adw::Dialog` | [src/ui/device_dialog.rs:33](../../src/ui/device_dialog.rs#L33) |
| `device_row.ui` | `SwDeviceRow` | `adw::ActionRow` | [src/ui/device_row.rs:35](../../src/ui/device_row.rs#L35) |
| `device_indicator.ui` | `SwDeviceIndicator` | `adw::Bin` | [src/ui/device_indicator.rs:31](../../src/ui/device_indicator.rs#L31) |
| `preferences_dialog.ui` | `SwSettingsDialog` | `adw::PreferencesDialog` | [src/ui/preferences_dialog.rs:29](../../src/ui/preferences_dialog.rs#L29) |
| `help_overlay.ui` (alias `gtk/help-overlay.ui`) | auto-loaded `GtkShortcutsWindow` (via `resource-base-path`) | — | — |
| `player.ui` | listed in the GResource but not referenced by any `#[template]` (leftover) | — | — |

The About dialog is built in code ([src/ui/about_dialog.rs](../../src/ui/about_dialog.rs))
from the bundled `metainfo.xml`.

## Layout and navigation

[data/gtk/window.ui](../../data/gtk/window.ui) uses an `AdwMultiLayoutView` with
two `AdwBreakpoint`s (`narrow_breakpoint`, `gadget_breakpoint`, lines 8 and 15):

- **Wide:** `AdwOverlaySplitView` (line 32): sidebar = `SwPlayerView`, content =
  `AdwNavigationView` (line 76) with `SwLibraryPage` → push `SwSearchPage`.
- **Narrow:** `AdwBottomSheet` (line 56) with `SwPlayerToolbar` as the bottom bar
  and `SwPlayerView` in the sheet.
- **Gadget:** a tiny always-on-top-style window showing only `SwPlayerGadget`;
  entering it saves the previous size (`window-previous-*`) and shrinks the
  window ([src/ui/window.rs:264-295](../../src/ui/window.rs#L264-L295)).
- Dialogs are `adw::Dialog`s presented over the window; toasts go through the
  `AdwToastOverlay`.

## Actions and shortcuts

App actions ([src/app.rs:72-140](../../src/app.rs#L72-L140)): `app.show-track(s)`,
`app.save-track(s)`, `app.cancel-recording(s)` (used by notification buttons),
`app.quit`.

Window actions ([src/ui/window.rs:66-131](../../src/ui/window.rs#L66-L131)):
`player.start-playback`, `player.stop-playback`, `player.toggle-playback`,
`player.show-device-connect`, `player.show-station-details`,
`win.open-radio-browser-info`, `win.add-local-station`,
`win.add-public-station`, `win.enable-gadget-player`,
`win.disable-gadget-player`, `win.show-preferences`, `win.about`. Library page:
`library.set-sorting`, `library.set-sorting-type`. Volume:
`volume.increase`, `volume.decrease`, `volume.toggle-mute`.

Accelerators ([src/app.rs:142-145](../../src/app.rs#L142-L145) and
[data/gtk/help_overlay.ui](../../data/gtk/help_overlay.ui)):

| Shortcut | Action |
|---|---|
| Ctrl+, | preferences |
| Ctrl+Q | quit |
| Ctrl+W | close window |
| Ctrl+Space | toggle playback |
| Ctrl+M / Ctrl++ / Ctrl+- | mute / volume up / down ([src/ui/volume_control.rs:136-150](../../src/ui/volume_control.rs#L136-L150)) |
| F10 | primary menu |
| Ctrl+? | shortcuts window (GTK default) |

`<primary>` maps to Ctrl on Windows, so no change is needed.

## CSS

- [data/gtk/style.css](../../data/gtk/style.css) (206 lines) and
  [style-dark.css](../../data/gtk/style-dark.css) (2 lines), loaded automatically by
  `AdwApplication` from `resource-base-path`.
- Background textures via `resource://…/assets/images/wood.png` and `metal.png`
  (style.css:104, 124) — resource URLs, so no file paths involved.
- libadwaita's `StyleManager` follows the Windows light/dark setting on
  libadwaita ≥ 1.6 through GTK's Win32 settings backend (to verify in Phase 4).

## Icons

- App icon: [data/icons/hicolor/scalable/apps/de.haeckerfelix.Shortwave.svg](../../data/icons/hicolor/scalable/apps/de.haeckerfelix.Shortwave.svg)
  (+ `.Devel.svg`, and `symbolic/apps/…-symbolic.svg`), installed to the hicolor
  theme; the window icon is set by name ([src/main.rs:63](../../src/main.rs#L63)).
- Custom symbolic icons are compiled into the GResource under `…/icons/scalable/…`
  (gresource lines 28-36): `cloud-disabled`, `check-plain`, `details`,
  `remove-custom-cover`, `streaming`, `window-grow`, `window-shrink`.
- Everything else comes from the **Adwaita** icon theme, which the Windows bundle
  must ship (symbolic SVGs need the librsvg gdk-pixbuf loader).
- Windows needs an `.ico` generated from the scalable SVG (Explorer, taskbar,
  installer).

## Screenshots (reference)

| | |
|---|---|
| ![1](../../data/screenshots/1.png) | ![2](../../data/screenshots/2.png) |
| ![3](../../data/screenshots/3.png) | ![4](../../data/screenshots/4.png) |
| ![5](../../data/screenshots/5.png) | |
