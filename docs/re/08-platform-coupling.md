# 08 — Platform coupling (Linux → Windows)

Every Linux-specific touchpoint found in Shortwave 5.0.0, with the proposed
Windows replacement. Found with Serena `fs.search_for_pattern` over `src/` for
`mpris|zbus|ashpd|portal|pulsesink|check_pulse|glycin|gio::Settings|user_data_dir|user_cache_dir|user_special_dir|bindtextdomain|LOCALEDIR|dbus|DBus|Notification|xdg|/usr/|/etc/`,
then followed with `lsp.find_symbol` (bodies) and `lsp.find_referencing_symbols`,
plus `cargo tree`, `pkg-config` and a `cargo check` of the unmodified tree under
MSYS2 UCRT64 (evidence in [evidence/](evidence/)).

Line numbers are 1-based against the `windows-port` baseline.

**Risk:** 🔴 blocks build or a required feature · 🟠 feature broken/degraded until fixed · 🟡 cosmetic or needs verification · 🟢 works as-is.

## Summary table

| # | Touchpoint | Location | What it does on Linux | Windows replacement | Risk |
|---|---|---|---|---|---|
| 1 | **`mpris-server` / zbus** | [src/audio/mpris.rs](../../src/audio/mpris.rs) (whole file); started from [src/audio/player.rs:139-144](../../src/audio/player.rs#L139-L144); `mod mpris` [src/audio/mod.rs:18,27](../../src/audio/mod.rs#L18) | Exposes `org.mpris.MediaPlayer2.Player` on the session bus: play/pause/stop, volume, raise, quit, metadata | **SMTC** (`Windows.Media.SystemMediaTransportControls`) through `souvlaki` (wraps MPRIS + SMTC; needs the HWND) or the `windows` crate. The crate **compiles** on Windows (zbus is portable) but has no bus to talk to; gate it to Linux. | 🟠 |
| 2 | **`ashpd` background portal** | [src/app.rs:21](../../src/app.rs#L21), [218-278](../../src/app.rs#L218-L278) (`setup_background_portal_proxy`, status text); [src/utils.rs:17](../../src/utils.rs#L17), [100-122](../../src/utils.rs#L100-L122) (`background_portal_permissions`); [src/ui/window.rs:211-243](../../src/ui/window.rs#L211-L243) | Flatpak "run in background" permission and status line | Drop on Windows (no sandbox). `background_portal_permissions()` already returns `true` when not sandboxed. **Build blocker:** `ashpd`'s `gtk4` feature pulls `gdk4-wayland-sys` + `gdk4-x11-sys`, whose build scripts fail (no `gtk4-wayland.pc`/`gtk4-x11.pc`). | 🔴 build |
| 3 | **Background playback = `app.hold()`** | [src/app.rs:207-216](../../src/app.rs#L207-L216); close handling [src/ui/window.rs:168-203](../../src/ui/window.rs#L168-L203) | Window closes, process keeps playing; the shell/MPRIS lets the user get back | **Tray icon** (Show/Hide, Play/Pause, Quit) so a hidden window is reachable; close → hide while `background-playback` is on | 🟠 |
| 4 | **Hard-coded `pulsesink` / `check_pulse_support()`** | [src/audio/gstreamer_backend.rs:82-89](../../src/audio/gstreamer_backend.rs#L82-L89), [504-507](../../src/audio/gstreamer_backend.rs#L504-L507) | Picks PulseAudio sink, falls back to `autoaudiosink` | `wasapi2sink` → `wasapisink` → `autoaudiosink` (all present, see [evidence/msys2-packages.txt](evidence/msys2-packages.txt)); skip the Pulse probe | 🟠 |
| 5 | **Volume only via element named `"pulsesink"`** | [gstreamer_backend.rs:124-162](../../src/audio/gstreamer_backend.rs#L124-L162), [275-306](../../src/audio/gstreamer_backend.rs#L275-L306) | Volume/mute read/write + external change notifications | Name the sink `audiosink` on every platform and look it up by that name; WASAPI sinks expose `volume`/`mute` | 🟠 (silent: volume stuck at 100 %) |
| 6 | **`glycin` image loading** | [src/api/cover_loader.rs:25](../../src/api/cover_loader.rs#L25), [89-110](../../src/api/cover_loader.rs#L89-L110) | Decodes favicons in sandboxed loader processes (bubblewrap + seccomp) | `gdk::Texture::from_bytes` (gdk-pixbuf loaders, incl. librsvg for SVG) behind the platform layer. **Build blocker:** `libseccomp` fails with ``cannot find `unix` in `os` ``; Meson also requires `libseccomp`/`lcms2` for it ([meson.build:22-23](../../meson.build#L22-L23)). | 🔴 build |
| 7 | **`gio::Settings` schema lookup** | [src/settings/settings_manager.rs:22-24](../../src/settings/settings_manager.rs#L22-L24); install [data/meson.build:58-69](../../data/meson.build#L58-L69) | Schema compiled into `/usr/share/glib-2.0/schemas` | Ship `share\glib-2.0\schemas\gschemas.compiled` next to `bin\`; set `GSETTINGS_SCHEMA_DIR` from `current_exe()` if unset. Backend becomes the registry (`HKCU\Software\GSettings\…`). A missing schema **aborts** the process. | 🟠 |
| 8 | **gettext `LOCALEDIR` baked in at compile time** | [src/main.rs:68-70](../../src/main.rs#L68-L70); [src/config.rs.in:8](../../src/config.rs.in#L8); [src/meson.build:10](../../src/meson.build#L10) | `bindtextdomain("shortwave", "/usr/share/locale")` | Resolve `<exe dir>\..\share\locale` at runtime on Windows. `setlocale(LC_ALL, "")` on Windows uses the user locale; GLib/GTK also honour `LANG` if set. `gettext-rs` with `gettext-system` links MSYS2's libintl (compiled OK). | 🟠 |
| 9 | **GResource path baked in** | [src/main.rs:73-80](../../src/main.rs#L73-L80) (`DATADIR/shortwave/<APP_ID>.gresource`) | Loads from `/usr/share/shortwave/` | Same runtime-relative resolution as #8 (`<exe dir>\..\share\shortwave\`); `expect()` panics otherwise | 🔴 runtime (app won't start) |
| 10 | **D-Bus `.service`, `.desktop`, metainfo** | [data/meson.build:4-56](../../data/meson.build#L4-L56), [82-92](../../data/meson.build#L82-L92); `gnome.post_install` [114-118](../../data/meson.build#L114-L118) | D-Bus activation, launcher, AppStream | Skip installing on Windows (`host_machine.system() == 'windows'`). Metainfo is still needed: it's compiled into the GResource as `metainfo.xml` for the About dialog ([data/de.haeckerfelix.Shortwave.gresource.xml.in:3](../../data/de.haeckerfelix.Shortwave.gresource.xml.in#L3)). Start Menu shortcut + AppUserModelID replace the `.desktop`. | 🟡 |
| 11 | **Meson `dependency('dbus-1')`** | [meson.build:13](../../meson.build#L13) | Required system dep (for the zbus/MPRIS stack's C side, historically) | Make it Linux-only. **Build blocker:** `meson setup` fails on it ([evidence/msys2-packages.txt](evidence/msys2-packages.txt)). | 🔴 build |
| 12 | **GApplication uniqueness** | [src/app.rs:305-324](../../src/app.rs#L305-L324) (`ApplicationFlags::empty()`, app id) | Single instance via the session bus name `de.haeckerfelix.Shortwave` | GLib 2.90's GDBus on Windows **autolaunches a private session bus** (`autolaunch:` + `g_win32_run_session_bus` strings found in `libgio-2.0-0.dll`), so uniqueness may work if `gdbus.exe` is bundled. Verify at runtime; fallback: named mutex + activate existing window. | 🟡 |
| 13 | **`GNotification`** | [src/audio/player.rs:296-300](../../src/audio/player.rs#L296-L300), [436-467](../../src/audio/player.rs#L436-L467) | Freedesktop/portal notifications with *Save Track* / *Don't Record* buttons | GLib 2.90 contains **`GWin32NotificationBackend`** (strings found). It is tray-balloon based; action buttons are likely unsupported → verify; fallback: Windows toast via `windows` crate + AUMID | 🟡 |
| 14 | **XDG Music dir default** | [src/audio/player.rs:154-163](../../src/audio/player.rs#L154-L163) | `~/Music` (or `$HOME`) | `glib::user_special_dir(Music)` already maps to `FOLDERID_Music` on Windows. Prompt wants `Music\Shortwave` → behaviour change, see "Decisions" | 🟢 / decision |
| 15 | **Data / cache dirs** | [src/path.rs:25-35](../../src/path.rs#L25-L35) | `~/.local/share/Shortwave`, `~/.cache/Shortwave` | Data → `%LOCALAPPDATA%\Shortwave` (GLib default, fine). Cache → GLib picks `%LOCALAPPDATA%\Microsoft\Windows\INetCache\Shortwave`; pin it to `%LOCALAPPDATA%\Shortwave\cache` | 🟡 |
| 16 | **`reqwest` TLS backend** | [Cargo.toml:30](../../Cargo.toml#L30); client [src/api/client.rs:45-58](../../src/api/client.rs#L45-L58) | native-tls → OpenSSL | native-tls → **SChannel** on Windows (`cargo tree --target x86_64-pc-windows-gnu -i schannel`; `openssl-sys` absent). Meson's `dependency('openssl')` ([meson.build:12](../../meson.build#L12)) becomes Linux-only. | 🟢 |
| 17 | **`async-std-resolver` system config** | [src/api/client.rs:89-97](../../src/api/client.rs#L89-L97) | `/etc/resolv.conf` | hickory reads Windows adapter DNS (compiled OK); fallback to built-in config exists. Reverse lookups need PTR answers from the local resolver. | 🟡 verify |
| 18 | **mDNS + Windows Firewall** | [src/device/device_discovery.rs:68-94](../../src/device/device_discovery.rs#L68-L94) | `mdns-sd` UDP 5353 | Works in principle (pure Rust, compiled OK); needs an inbound UDP 5353 firewall rule for `shortwave.exe` and sharing 5353 with Windows' Dnscache. See [06-casting.md](06-casting.md). | 🟡 |
| 19 | **Meson cargo wrapper** | [src/meson.build:19-24](../../src/meson.build#L19-L24), [41-57](../../src/meson.build#L41-L57) | `cp`, `env`, output name `shortwave` | MSYS2 provides `cp`/`env`, but the cargo output is `shortwave.exe` on Windows — `cp target/release/shortwave` fails. Use `meson.project_name() + exe_suffix`. `config.rs` is copied into the source tree (gitignored). | 🔴 build |
| 20 | **`GtkApplication::inhibit(SUSPEND)`** | [src/app.rs:342-360](../../src/app.rs#L342-L360) | Prevents suspend during playback via session manager / portal | GTK 4.24 Win32 may or may not implement it (strings present, backend unknown). Verify; fallback `SetThreadExecutionState(ES_CONTINUOUS \| ES_SYSTEM_REQUIRED)` | 🟡 |
| 21 | **`PULSE_PROP_*` env vars** | [src/main.rs:64-65](../../src/main.rs#L64-L65) | Name/icon in PulseAudio mixers | No-op on Windows; gate to Linux for clarity | 🟢 |
| 22 | **Window icon by theme name** | [src/main.rs:63](../../src/main.rs#L63) (`set_default_icon_name(APP_ID)`) | Found in `hicolor` theme | Ship `share\icons\hicolor\scalable\apps\de.haeckerfelix.Shortwave.svg` + librsvg loader; plus an embedded `.ico` resource for Explorer/taskbar | 🟡 |
| 23 | **`gtk::FileLauncher` for "play recorded track"** | [src/audio/track.rs:200-212](../../src/audio/track.rs#L200-L212) | Opens via portal / `xdg-open` | GTK 4.24 on Windows uses `ShellExecute`; expected to work, verify | 🟡 |
| 24 | **`sanitize-filename` + MAX_PATH** | [src/audio/track.rs:185-191](../../src/audio/track.rs#L185-L191) | Removes `/` and NUL | Crate already applies Windows rules on Windows (reserved names, `<>:"/\|?*`, trailing dots, 255-byte name). Total path > 260 not handled → add long-path manifest (`longPathAware`) or truncate | 🟡 |
| 25 | **libshumate map** | [src/ui/station_dialog.rs:24](../../src/ui/station_dialog.rs#L24), [75-78](../../src/ui/station_dialog.rs#L75-L78), [225-237](../../src/ui/station_dialog.rs#L225-L237) | OSM map of the station | **Available**: `mingw-w64-ucrt-x86_64-libshumate 1.7.0`, `shumate-1.0` found, `libshumate-sys` compiled. Keep. | 🟢 |
| 26 | **Debian packaging** | [debian/](../../debian/) | `.deb` for Mauna Linux | Not used on Windows; leave untouched for upstream merges | 🟢 |

## Decisions / questions for Checkpoint A

1. **Recording folder default** — upstream saves into the Music folder itself;
   the prompt says `%USERPROFILE%\Music\Shortwave`. Proposal: on Windows only,
   default to `Music\Shortwave` (created on first save); Linux unchanged.
2. **Cache location** — pin to `%LOCALAPPDATA%\Shortwave\cache` instead of
   GLib's INetCache default (#15).
3. **Notifications / single instance** — try GLib's built-in Win32 backends
   first (#12, #13) and only add toast / named-mutex code if they don't behave.
   This deviates from writing them up-front.

## What is *not* coupled (verified)

- GTK4 / libadwaita / libshumate / GStreamer bindings: all `*-sys` build scripts
  succeed against MSYS2 UCRT64 (`cargo check` evidence).
- Diesel + SQLite (system `sqlite3` 3.54.1 via pkg-config), cacache, reqwest,
  mdns-sd, cast-sender, hickory, gettext-rs: compile for `x86_64-pc-windows-gnu`.
- The recording branch (`vorbisenc ! oggmux ! filesink`) uses only
  gst-plugins-base/core elements present on Windows.
- No `/usr/` or `/etc/` literals in Rust sources; all paths go through GLib.
