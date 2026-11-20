# 01 — Architecture

Shortwave 5.0.0 (Mauna Linux packaging of GNOME Shortwave by Felix Häcker).
Single Rust binary crate, GTK4 + libadwaita UI, GStreamer playback, SQLite via
Diesel, radio-browser.info REST API, Google Cast. Line numbers are 1-based
against the `windows-port` baseline (upstream `ce4be13`).

## Module map (`src/`)

| Module | Files | Responsibility |
|---|---|---|
| `main.rs` | — | Process entry: logger, paths, GTK/GStreamer init, gettext, GResource, `SwApplication::run()` ([src/main.rs:48-84](../../src/main.rs#L48-L84)) |
| `app.rs` | — | `SwApplication` (`adw::Application`): owns library, player, cover loader, rb server, background-playback hold, app actions ([src/app.rs](../../src/app.rs)) |
| `config.rs` | generated from `config.rs.in` by Meson | `NAME`, `PKGNAME`, `APP_ID`, `PATH_ID`, `VERSION`, `PROFILE`, `VCS_TAG`, `LOCALEDIR`, `DATADIR` ([src/config.rs.in](../../src/config.rs.in)) |
| `path.rs` | — | `DATA` / `CACHE` dirs from GLib ([src/path.rs](../../src/path.rs)) |
| `i18n.rs` | — | `i18n()`, `i18n_f()`, `ni18n…` gettext helpers |
| `utils.rs` | — | channel send helper, duration formatting, ellipsizing, background portal check |
| `api/` | `client`, `cover_loader`, `station`, `station_metadata`, `station_model`, `station_request`, `station_sorter`, `stats`, `error` | radio-browser client, `SwStation` GObject, list model + sorter, cover loading/cache — see [02-api-protocol.md](02-api-protocol.md) |
| `audio/` | `gstreamer_backend`, `player`, `mpris`, `track`, `track_model`, `recording_state`, `recording_mode`, `playback_state` | playback pipeline, recording, MPRIS — see [05-audio-pipeline.md](05-audio-pipeline.md) |
| `database/` | `connection`, `queries`, `models`, `schema`, `library`, `library_status` | Diesel pool + migrations, `SwLibrary` — see [03-data-model.md](03-data-model.md) |
| `device/` | `device_discovery`, `cast_sender`, `device`, `device_kind`, `device_model` | mDNS discovery and Google Cast — see [06-casting.md](06-casting.md) |
| `settings/` | `key`, `settings_manager` | typed GSettings access — see [04-settings.md](04-settings.md) |
| `ui/` | window, pages, player views, dialogs, rows | GTK templates — see [07-ui-inventory.md](07-ui-inventory.md) |

## GObject types

All subclasses use the `glib::subclass` + `#[derive(Properties)]` pattern with
an inner `imp` module.

| Type | Parent | Defined at |
|---|---|---|
| `SwApplication` | `adw::Application` | [src/app.rs:61-62](../../src/app.rs#L61-L62) |
| `SwApplicationWindow` | `adw::ApplicationWindow` | [src/ui/window.rs:58-59](../../src/ui/window.rs#L58-L59) |
| `SwPlayer` | `glib::Object` | [src/audio/player.rs:91](../../src/audio/player.rs#L91) |
| `SwTrack`, `SwTrackModel` | `glib::Object` (model implements `gio::ListModel`) | [src/audio/track.rs:65](../../src/audio/track.rs#L65), [src/audio/track_model.rs:41](../../src/audio/track_model.rs#L41) |
| `SwLibrary` | `glib::Object` | [src/database/library.rs:45](../../src/database/library.rs#L45) |
| `SwStation`, `SwStationModel` | `glib::Object` | [src/api/station.rs:52](../../src/api/station.rs#L52), [src/api/station_model.rs:36](../../src/api/station_model.rs#L36) |
| `SwStationSorter` | `gtk::Sorter` | [src/api/station_sorter.rs:40-42](../../src/api/station_sorter.rs#L40-L42) |
| `SwDevice`, `SwDeviceModel`, `SwDeviceDiscovery`, `SwCastSender` | `glib::Object` | [src/device/](../../src/device/) |
| `SwLibraryPage`, `SwSearchPage` | `adw::NavigationPage` | [src/ui/pages/](../../src/ui/pages/) |
| `SwPlayerView`, `SwPlayerToolbar`, `SwPlayerGadget` | `adw::Bin` | [src/ui/player/](../../src/ui/player/) |
| `SwStationDialog`, `SwTrackDialog`, `SwAddStationDialog`, `SwDeviceDialog` | `adw::Dialog` | [src/ui/](../../src/ui/) |
| `SwSettingsDialog` | `adw::PreferencesDialog` | [src/ui/preferences_dialog.rs:48-49](../../src/ui/preferences_dialog.rs#L48-L49) |
| `SwStationRow`, `SwStationCover`, `SwDeviceIndicator` | `adw::Bin` | [src/ui/](../../src/ui/) |
| `SwTrackRow`, `SwDeviceRow` | `adw::ActionRow` | [src/ui/](../../src/ui/) |
| `SwRecordingIndicator` | `gtk::Button` | [src/ui/recording_indicator.rs:43-44](../../src/ui/recording_indicator.rs#L43-L44) |
| `SwVolumeControl` | `gtk::Widget` | [src/ui/volume_control.rs:52-53](../../src/ui/volume_control.rs#L52-L53) |
| Enums (`glib::Enum`) | `SwPlaybackState`, `SwRecordingState`, `SwRecordingMode`, `SwLibraryStatus`, `SwDeviceKind` | [src/audio/](../../src/audio/), [src/database/library_status.rs](../../src/database/library_status.rs) |

`SwApplication::default()` returns the running application singleton and is
the service locator used throughout (e.g. `SwApplication::default().player()`).

## Async model

- **Main loop:** everything runs on the GLib main context. Futures are spawned
  with `glib::spawn_future_local` (UI-thread, `!Send` allowed).
- **Channels:** `async-channel` connects GStreamer's streaming threads to the
  UI (`GstreamerChange`, bounded 10 — [src/audio/player.rs:101-118](../../src/audio/player.rs#L101-L118))
  and feeds the cover loader (unbounded, `buffer_unordered` —
  [src/api/cover_loader.rs:143-154](../../src/api/cover_loader.rs#L143-L154)).
- **Tokio bridging:** `reqwest` needs a Tokio reactor, so HTTP futures are
  wrapped in `async_compat::Compat` ([src/api/client.rs:184-186](../../src/api/client.rs#L184-L186)).
- **async-std:** `async-std-resolver` (DNS) and `ashpd` (`async-std` feature)
  run on async-std's global executor; `mdns-sd` runs its own daemon thread and
  delivers events via `recv_async()`; `cast-sender` uses `async-net` /
  `async-native-tls`.
- **Timers:** `glib::timeout_add_seconds_local(1, …)` polls the recording
  duration ([player.rs:177-201](../../src/audio/player.rs#L177-L201)).

## Start-up sequence

```mermaid
sequenceDiagram
  participant M as main()
  participant A as SwApplication
  participant P as SwPlayer
  participant L as SwLibrary
  participant W as SwApplicationWindow
  M->>M: pretty_env_logger::init()
  M->>M: path::init() (create DATA, CACHE)
  M->>M: gtk::init(), gstreamer::init()
  M->>M: set_application_name, set_default_icon_name(APP_ID), PULSE_PROP_* env
  M->>M: setlocale, bindtextdomain(PKGNAME, LOCALEDIR), textdomain
  M->>M: gio::Resource::load(DATADIR/shortwave/APP_ID.gresource) + register
  M->>A: SwApplication::run() (application-id, flags=empty, resource-base-path)
  Note over A: GObject construction builds default fields:<br/>SwLibrary (loads SQLite), SwPlayer (GStreamer pipeline,<br/>MPRIS, recording cleanup), CoverLoader
  A->>L: constructed(): DB pool + migrations, load stations
  A->>P: constructed(): GstreamerBackend::new(), MprisServer::start(), 1 s timer
  A->>A: constructed(): app actions (show-track, save-track, cancel-recording, quit), accels
  A->>A: startup(): spawn [lookup_rb_server → library.update_data(); setup_background_portal_proxy]
  A->>P: restore_state() (last station, volume)
  A->>A: bind background-playback setting
  A->>A: activate()
  A->>W: application_window() → SwApplicationWindow::new(); present()
  W->>W: constructed(): restore window size
```

References: [src/main.rs:48-84](../../src/main.rs#L48-L84);
[src/app.rs:68-147](../../src/app.rs#L68-L147) (`constructed`),
[151-190](../../src/app.rs#L151-L190) (`startup` / `activate` / `shutdown`),
[305-324](../../src/app.rs#L305-L324) (`run`),
[326-336](../../src/app.rs#L326-L336) (`application_window`);
[src/ui/window.rs:149-162](../../src/ui/window.rs#L149-L162).

Shutdown: `ApplicationImpl::shutdown` prunes the cover cache
([src/app.rs:184-189](../../src/app.rs#L184-L189)). Window close saves geometry
and, if background playback is on and something is playing, hides instead of
closing ([src/ui/window.rs:168-203](../../src/ui/window.rs#L168-L203)).

## Build system

Meson is the entry point ([meson.build](../../meson.build)): it checks system
dependencies, generates `src/config.rs` (and **copies it into the source tree**
so plain `cargo` also works — [src/meson.build:13-24](../../src/meson.build#L13-L24)),
compiles the GResource bundle and schema, then runs `cargo build` as a
`custom_target` with `CARGO_HOME` inside the build dir
([src/meson.build:27-57](../../src/meson.build#L27-L57)). The `development`
profile appends `.Devel` to the app id and builds debug.

## Baseline: Mauna vs. GNOME upstream

`git log` of `maunalinux/shortwave` has only five commits (`6474cdb Initial
commit` … `ce4be13 5.0.0`); it's an import, not a fork with history. Comparing
against GNOME's tag (`git fetch gnome refs/tags/5.0.0:refs/tags/gnome-5.0.0`,
remote `https://gitlab.gnome.org/World/Shortwave.git`, push disabled):

```text
$ git diff --stat gnome-5.0.0 ce4be13 -- . ':!debian' ':!po'
23 files changed, 926 insertions(+), 957 deletions(-)
```

The non-translation differences are all **upstream post-5.0.0 changes**, not
Mauna patches: libadwaita feature `v1_6` → `v1_7` (Cargo.toml, meson.build),
`cacache::write_with_algo(Xxh3, …)`, `prune_cache()` made synchronous at
shutdown, `update_mpris_plaback_status` typo fixed, log-format cleanups, small
`.ui` tweaks, a refreshed `Cargo.lock`, and updated translations under `po/`.
Mauna's own additions are `debian/` only (whose `control` still lists GTK3-era
build deps and isn't used by the Meson build).

**Consequence for merges:** treat the baseline as "GNOME Shortwave main shortly
after 5.0.0". Future updates should be merged from GNOME upstream (remote
`gnome`) as well as from `upstream` (Mauna), which only re-imports releases.
