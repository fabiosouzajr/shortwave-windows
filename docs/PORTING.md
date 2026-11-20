# Porting design — Shortwave for Windows

Status: **proposal for Checkpoint B**. Based on the findings in
[docs/re/08-platform-coupling.md](re/08-platform-coupling.md) and
[docs/re/09-dependency-matrix.md](re/09-dependency-matrix.md).

Target: Windows 10/11 x64, built with MSYS2 **UCRT64** (GNU toolchain), Rust +
GTK4/libadwaita kept as-is. The Linux build must keep compiling and behaving
exactly as before.

## Decisions confirmed at Checkpoint A

| # | Decision |
|---|---|
| A1 | Default recording folder on **Windows only** is `<Music>\Shortwave` (created on first save). Linux keeps the Music folder. |
| A2 | Cover cache on Windows is `%LOCALAPPDATA%\Shortwave\cache` instead of GLib's `INetCache`. |
| A3 | Notifications and single instance: use GLib's built-in Win32 support first (`GWin32NotificationBackend`, GDBus session-bus autolaunch). Custom code (toasts, named mutex) only if runtime tests show they fall short. |
| A4 | Phase 2b (WSL runtime observation) is **skipped**. |
| A5 | No Cast device available → casting is built and checked as far as discovery/firewall allow, and reported as **untested end-to-end**. |

## 1. Platform layer

New module `src/platform/` with one public API and two implementations chosen
at compile time:

```text
src/platform/mod.rs      // pub use of the active backend + shared types
src/platform/linux.rs    // wraps the existing upstream code (moved, not rewritten)
src/platform/windows.rs  // Windows implementations
src/platform/windows/    // submodules if windows.rs grows: smtc.rs, tray.rs, paths.rs
```

`mod.rs` uses `#[cfg_attr(target_os = "linux", path = "linux.rs")]` /
`#[cfg_attr(windows, path = "windows.rs")]` so call sites stay `platform::…`.

| Function | Linux | Windows |
|---|---|---|
| `platform::init()` (called first in `main`) | no-op | set `GSETTINGS_SCHEMA_DIR` if unset; set `XDG_DATA_DIRS`-equivalent only if needed for icons; attach console in debug builds |
| `platform::resource_dirs() -> ResourceDirs { datadir, localedir }` | `config::DATADIR`, `config::LOCALEDIR` (unchanged) | `<exe dir>\..\share`, `<exe dir>\..\share\locale` |
| `platform::cache_dir() -> PathBuf` | `glib::user_cache_dir()/Shortwave` (unchanged) | `%LOCALAPPDATA%\Shortwave\cache` |
| `platform::default_recording_dir() -> PathBuf` | `user_special_dir(Music)` or home (unchanged) | `user_special_dir(Music)\Shortwave` or `%USERPROFILE%\Music\Shortwave` |
| `platform::default_audio_sink() -> &'static str` | `pulsesink` if available, else `autoaudiosink` (unchanged logic) | first available of `wasapi2sink`, `wasapisink`, `autoaudiosink` |
| `platform::media_controls::start()` | existing `MprisServer::start()` | SMTC (see §4) |
| `platform::background::setup(app)` / `request_permission()` | existing ashpd portal code | tray icon (see §5); permission always granted |
| `platform::inhibit_suspend(bool)` | existing `gtk::Application::inhibit` | `gtk::Application::inhibit` if it returns a non-zero cookie, else `SetThreadExecutionState` |
| `platform::load_cover_texture(file) -> Texture` | existing glycin path | `gdk::Texture::from_bytes` (gdk-pixbuf loaders) |

`path.rs`, `main.rs`, `gstreamer_backend.rs`, `cover_loader.rs`, `app.rs`,
`window.rs`, `utils.rs` and `player.rs` change only at those call sites. Upstream
code moves into `platform/linux.rs` verbatim so merges stay readable.

## 2. Cargo dependencies

```toml
[dependencies]
# …everything that compiles on both (unchanged)…

[target.'cfg(target_os = "linux")'.dependencies]
ashpd = { version = "0.10", default-features = false, features = ["gtk4", "async-std"] }
glycin = { version = "2.0", features = ["gdk4"] }
mpris-server = "0.8"

[target.'cfg(windows)'.dependencies]
souvlaki = "<latest compatible>"         # SMTC
gdk4-win32 = "0.9"                        # HWND from the GTK surface (matches gtk4 0.9)
tray-icon = "<latest compatible>"         # tray + menu (muda)
windows = { version = "<x>", features = ["Win32_System_Power", "Win32_Foundation", …] }

[target.'cfg(windows)'.build-dependencies]
winresource = "<latest compatible>"       # icon, version info, manifest
```

Exact versions get pinned in Phase 4 after a compile test; each one is recorded
in `docs/re/09-dependency-matrix.md`. Gating these three crates removes both
`zbus` versions, `libseccomp` and the Wayland/X11 GDK crates from the Windows
build (`cargo tree` evidence).

## 3. Audio

- Pipeline string becomes `… ! queue ! {sink} name=audiosink`, with `{sink}`
  from `platform::default_audio_sink()`.
- All volume/mute lookups use `by_name("audiosink")` instead of
  `by_name("pulsesink")`. On Linux the element is still `pulsesink`, so behaviour
  is identical; on Windows WASAPI's `volume`/`mute` properties are used.
- The "PulseAudio is required" warnings become "audio sink has no volume
  property" and only fire when that's actually true (e.g. `autoaudiosink`).
- Recording branch unchanged (`queue ! vorbisenc ! oggmux ! filesink`).
- `PULSE_PROP_*` env vars gated to Linux.

## 4. Media keys / SMTC

- `souvlaki::MediaControls::new(PlatformConfig { hwnd: Some(hwnd), … })`, with
  the HWND taken from the main window's `gdk4_win32::Win32Surface` after it is
  realized. Created lazily on first `realize`; recreated if the window is
  recreated.
- Metadata: **title = station name**, **artist = current track title**,
  **cover = station favicon URL** (or a file URL to the cached PNG if the remote
  URL isn't usable). Playback state from `SwPlayer::state`.
- Events: Play / Pause / Toggle / Stop → `SwPlayer` actions; volume keys are
  handled by Windows itself for the system volume, and `SetVolume` (if SMTC
  delivers it) maps to `SwPlayer::set_volume`. souvlaki calls back from another
  thread → forward through an `async_channel` to the GLib main context.
- Fallback if souvlaki doesn't fit: `windows` crate with
  `ISystemMediaTransportControlsInterop::GetForWindow` (same design).

## 5. Background playback / tray

- When `background-playback` is on, a tray icon (`tray-icon` + `muda` menu:
  **Show/Hide**, **Play/Pause**, **Quit**) is shown; left-click toggles the
  window.
- `close_request`: if background playback is on **and** playing, hide the window
  (upstream already hides; the portal permission check becomes "always
  granted"). The existing `app.hold()` keeps the process alive; the tray is the
  way back.
- Quit from the tray calls `app.quit()`.
- Tray events arrive via `tray-icon`'s channel; drained on the GLib main context
  with an idle/timeout source (GTK's Win32 backend pumps the thread's message
  queue, which `tray-icon`'s hidden window needs).

## 6. Paths, settings, i18n, resources

- `main.rs`: GResource path from `platform::resource_dirs().datadir`;
  `bindtextdomain` with `resource_dirs().localedir`.
- `gio::Settings`: keep. Schema compiled into `share\glib-2.0\schemas`; GLib
  finds `<prefix>\share` from the DLL location, and `platform::init()` also sets
  `GSETTINGS_SCHEMA_DIR` (relative to `current_exe()`) when unset so dev runs
  from the build dir work. Registry backend on Windows.
- Data dir unchanged (`%LOCALAPPDATA%\Shortwave`); cache per A2.
- App ID and name **unchanged** (`de.haeckerfelix.Shortwave`, "Shortwave").

## 7. Covers

`cover_bytes()` calls `platform::load_cover_texture(file)`; on Windows that
reads the downloaded bytes and calls `gdk::Texture::from_bytes`. GTK decodes
PNG/JPEG/TIFF itself; ICO, GIF, BMP and SVG go through gdk-pixbuf loaders
(bundle `pixbufloader_svg` from librsvg and the WebP loader from
`webp-pixbuf-loader`). Anything still unsupported falls back to the placeholder
cover, the same as a glycin error today.
Snapshotting, caching and pruning stay shared.

## 8. Recording file names

`sanitize-filename` already applies Windows rules when built for Windows.
Added on Windows: truncate the title part so the full path stays ≤ 259 chars,
and declare `longPathAware` in the manifest (only effective when the
`LongPathsEnabled` policy is on). Same-title overwrites are upstream behaviour
and stay unchanged.

## 9. Notifications, single instance, inhibit (A3)

- Keep `GNotification` / `send_notification`. Verify on Windows: does the
  balloon show, do *Save Track* / *Don't Record* buttons work? If buttons don't,
  implement a toast backend with the `windows` crate (AppUserModelID
  `de.haeckerfelix.Shortwave`, set on the Start Menu shortcut by the installer).
- Keep GApplication uniqueness; bundle `gdbus.exe` for the session-bus
  autolaunch. Verify a second launch activates the first window; fallback is a
  named mutex + `FindWindow`/`SetForegroundWindow`.
- Inhibit per §1.

## 10. Casting

Keep `mdns-sd` + `cast-sender` unchanged (SChannel TLS). The installer adds an
inbound UDP 5353 rule for `shortwave.exe`; `docs/BUILDING-WINDOWS.md` explains
the manual command for portable-zip users. End-to-end casting is untested (A5).

## 11. Map

`libshumate` 1.7 is packaged by MSYS2 → keep the map. Tiles come from OSM over
HTTPS through libsoup3. HTTPS radio streams (`souphttpsrc`) use the same
libsoup3, so the bundle must include `lib\gio\modules\` from `glib-networking`
(`libgioopenssl.dll` or `libgiognutls.dll`, installed) plus a CA bundle — without
it every `https://` stream and tile fails. To verify in Phase 4.

## 12. Windows integration

- `build.rs` (new, Windows-only effect) with `winresource`: `.ico`, version info
  from `CARGO_PKG_VERSION`/Meson, manifest (`PerMonitorV2` DPI awareness,
  `asInvoker`, `longPathAware`, Common Controls v6).
- `.ico` generated once from `data/icons/hicolor/scalable/apps/de.haeckerfelix.Shortwave.svg`
  (16–256 px, via `rsvg-convert` + a small packing script) and committed under
  `build-aux/windows/` (binary per `.gitattributes`).
- `#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]`
  in `main.rs`; debug builds keep the console for logs.
- Dark mode: libadwaita `StyleManager` default (`prefer-light` → follow system).
  Verify it follows the Windows setting; if not, read
  `HKCU\…\Themes\Personalize\AppsUseLightTheme` at start-up.

## 13. Build system

Meson stays primary:

- `meson.build`: `dependency('dbus-1')`, `openssl`, `lcms2`, `libseccomp` only
  when `host_machine.system() == 'linux'`.
- `data/meson.build`: `.desktop`, D-Bus `.service`, `gnome.post_install` only on
  Linux. The metainfo is still generated (it's bundled into the GResource). On
  Windows, compile the schema at install time with `glib-compile-schemas`.
- `src/meson.build`: copy `target/<profile>/shortwave` **+ exe suffix**; config
  generation unchanged.
- `build-aux/windows/build.ps1`: runs `meson setup` / `meson compile` /
  `meson install --destdir` inside MSYS2 UCRT64, prefix `/` into
  `_build\install`. `bundle.ps1` (Phase 5) builds `dist\Shortwave\` from that.

## 14. Order of work (Phase 4)

As in the prompt: (1) toolchain — done; (2) compile with cfg-gating;
(3) paths/settings/i18n/resources; (4) audio; (5) library + covers;
(6) recording; (7) SMTC; (8) tray, notifications, single instance;
(9) Chromecast (discovery + firewall only, A5); (10) Windows polish. Each ends
with a build, a smoke test, a commit and a push.

## Deviations from the prompt (for approval)

1. Notifications / single instance: GLib built-ins first, custom code only if
   needed (A3).
2. `sqlite3` / `gettext` MSYS2 package names differ (renamed upstream).
3. Data dir keeps upstream's casing `Shortwave` (`%LOCALAPPDATA%\Shortwave`), not
   `shortwave`.
4. Linux keeps `mpris-server` rather than switching to souvlaki's MPRIS backend,
   to keep the Linux code identical to upstream.
5. Future merges come from GNOME upstream (`gnome` remote) as well as Mauna
   (`upstream`), because Mauna's tree tracks GNOME main.
