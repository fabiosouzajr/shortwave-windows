# 04 — Settings (GSettings)

Schema: [data/de.haeckerfelix.Shortwave.gschema.xml.in](../../data/de.haeckerfelix.Shortwave.gschema.xml.in),
schema id `@APP_ID@` (`de.haeckerfelix.Shortwave`, or `….Devel` for the
development profile), path `/de/haeckerfelix/Shortwave/`. Configured and
installed to `<datadir>/glib-2.0/schemas` by
[data/meson.build:58-69](../../data/meson.build#L58-L69); compiled by
`gnome.post_install(glib_compile_schemas: true)` ([data/meson.build:114-118](../../data/meson.build#L114-L118)).

Access goes through `settings_manager` ([src/settings/settings_manager.rs](../../src/settings/settings_manager.rs)),
which creates a **new** `gio::Settings::new(config::APP_ID)` on every call
([settings_manager.rs:22-24](../../src/settings/settings_manager.rs#L22-L24)).
The `Key` enum maps kebab-case names via `strum` ([src/settings/key.rs:16-45](../../src/settings/key.rs#L16-L45)).

## Keys

| Key | Type | Default | Read / written by |
|---|---|---|---|
| `api-lookup-domain` | `s` | `"all.api.radio-browser.info"` | read: `client::lookup_rb_server` [src/api/client.rs:88](../../src/api/client.rs#L88) |
| `library-sorting` | `s` | `"name"` | bound to the sort menu: [src/ui/pages/library_page.rs:75](../../src/ui/pages/library_page.rs#L75) |
| `library-sorting-type` | `s` | `"ascending"` | bound: [library_page.rs:76](../../src/ui/pages/library_page.rs#L76) |
| `playback-last-station` | `s` | `""` | written on station change [src/audio/player.rs:493](../../src/audio/player.rs#L493); read by `restore_state` [player.rs:581](../../src/audio/player.rs#L581). Stores the serialised station. |
| `playback-past-tracks-count` | `i` | `10` | read at player construction [player.rs:167](../../src/audio/player.rs#L167) |
| `playback-volume` | `d` | `1.0` | written on volume change [player.rs:225](../../src/audio/player.rs#L225); read by `restore_state` [player.rs:576](../../src/audio/player.rs#L576) |
| `recording-maximum-duration` | `i` (s) | `900` | 1 s timer [player.rs:188](../../src/audio/player.rs#L188); spin row [src/ui/preferences_dialog.rs:98](../../src/ui/preferences_dialog.rs#L98) |
| `recording-minimum-duration` | `i` (s) | `30` | `stop_recording` [player.rs:384](../../src/audio/player.rs#L384); spin row [preferences_dialog.rs:115](../../src/ui/preferences_dialog.rs#L115) |
| `recording-mode` | `s` | `"decide"` (`nothing`/`decide`/`everything`) | bound to `SwPlayer:recording-mode` [player.rs:171](../../src/audio/player.rs#L171); combo row [preferences_dialog.rs:78](../../src/ui/preferences_dialog.rs#L78) |
| `recording-track-directory` | `s` | `""` → set at first run to the XDG Music dir, or `$HOME` | [player.rs:154-163](../../src/audio/player.rs#L154-L163); save path [src/audio/track.rs:185](../../src/audio/track.rs#L185); folder chooser [preferences_dialog.rs:84](../../src/ui/preferences_dialog.rs#L84), [149](../../src/ui/preferences_dialog.rs#L149) |
| `window-width` | `i` | `975` | [src/ui/window.rs:159-160](../../src/ui/window.rs#L159-L160) (restore), [173-174](../../src/ui/window.rs#L173-L174) (save on close) |
| `window-height` | `i` | `675` | same as above |
| `window-previous-width` | `i` | `1000` | size before switching to the mini "gadget" player [window.rs:272-281](../../src/ui/window.rs#L272-L281) |
| `window-previous-height` | `i` | `650` | same |
| `background-playback` | `b` | `true` | bound to `SwApplication:background-playback` [src/app.rs:171](../../src/app.rs#L171); switch row [preferences_dialog.rs:66](../../src/ui/preferences_dialog.rs#L66) |
| `notifications` | `b` | `false` | checked on every track change [player.rs:296](../../src/audio/player.rs#L296); switch row [preferences_dialog.rs:72](../../src/ui/preferences_dialog.rs#L72) |

## Windows behaviour

- **Backend.** GLib on Windows defaults to the **registry backend**
  (`HKCU\Software\GSettings\de\haeckerfelix\Shortwave\…`), so values persist
  without extra work. An installer uninstall should offer to remove that key.
- **Schema lookup.** `gio::Settings::new()` *aborts the process* if the schema
  isn't found. GLib searches `GSETTINGS_SCHEMA_DIR`, then
  `<XDG_DATA_DIRS>/glib-2.0/schemas`. On Windows, GLib derives the default
  `XDG_DATA_DIRS` from the DLL location (`<prefix>\share`), so a bundle laid out
  as `bin\` + `share\glib-2.0\schemas\gschemas.compiled` is found automatically;
  setting `GSETTINGS_SCHEMA_DIR` from `current_exe()` (prompt default) is a
  belt-and-braces fallback for development runs from `_build`.
- **Recording directory default.** `glib::user_special_dir(Music)` maps to the
  Known Folder `FOLDERID_Music` on Windows (usually `%USERPROFILE%\Music`,
  redirected to OneDrive on some machines). Upstream saves straight into the
  Music folder; the prompt asks for `Music\Shortwave`. This is a behaviour change
  to confirm (see [08-platform-coupling.md](08-platform-coupling.md)).
- **Per-call `Settings::new()`** costs a schema lookup each time on every
  platform; it is not Windows-specific and is left alone.
