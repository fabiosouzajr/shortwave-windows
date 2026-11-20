# 09 — Dependency matrix (Windows x64, MSYS2 UCRT64)

✅ works on Windows · ⚠️ works but needs config or verification · ❌ needs replacement / gating.

Evidence (all collected 2026-11-20 on Windows 11 Pro, MSYS2 UCRT64):

- [evidence/cargo-check-unmodified.txt](evidence/cargo-check-unmodified.txt) —
  `cargo check --locked --keep-going` of the **unmodified** `Cargo.toml`,
  host `x86_64-pc-windows-gnu`, MSYS2 rustc 1.100.0.
- [evidence/cargo-tree.txt](evidence/cargo-tree.txt) — `cargo tree -i …`,
  default and `--target x86_64-pc-windows-gnu`.
- [evidence/msys2-packages.txt](evidence/msys2-packages.txt) — `pacman -Q`,
  `pkg-config --modversion`, `meson setup` result, `gst-inspect-1.0` sinks.

## Rust crates (`Cargo.toml`)

| Crate | Version | Status | Evidence / note |
|---|---|---|---|
| `anyhow` | 1.0 | ✅ | compiled |
| `ashpd` | 0.10 (`gtk4`, `async-std`) | ❌ | Its `gtk4` feature pulls `gdk4-wayland` + `gdk4-x11`; build scripts of `gdk4-wayland-sys v0.9.6` and `gdk4-x11-sys v0.9.6` **fail** (no `gtk4-wayland`/`gtk4-x11` pkg-config). Portal concept doesn't exist on Windows → gate to `cfg(target_os = "linux")`. |
| `async-channel` | 2.3 | ✅ | compiled |
| `async-compat` | 0.2 | ✅ | compiled |
| `async-io` | 2.4 | ✅ | compiled (IOCP/`polling` on Windows) |
| `async-std-resolver` | 0.24 | ⚠️ | compiled (`hickory-resolver 0.24.4`); reads Windows adapter DNS. Reverse-lookup behaviour to verify at runtime. |
| `cacache` | 13.1 | ✅ | compiled |
| `cast-sender` | 0.2 | ✅ | compiled; TLS via `async-native-tls` → `schannel` on Windows |
| `diesel` | 2.2 (`sqlite`, `r2d2`) | ✅ | compiled; `libsqlite3-sys 0.33` linked system `sqlite3` 3.54.1 via pkg-config |
| `diesel_migrations` | 2.2 | ✅ | compiled |
| `futures-util` | 0.3 | ✅ | compiled |
| `glycin` | 2.0 (`gdk4`) | ❌ | `libseccomp v0.3.0` fails: ``filter_context.rs:9:14: cannot find `unix` in `os` ``. Also pulls `zbus 5`. Replace the decode step with `gdk::Texture::from_bytes` on Windows. |
| `indexmap` | 2.7 | ✅ | compiled |
| `Inflector` | 0.11 | ✅ | compiled |
| `language-tags` | 0.3 | ✅ | compiled |
| `log` | 0.4 | ✅ | compiled |
| `mdns-sd` | 0.13 | ⚠️ | compiled; needs firewall rule for UDP 5353 (runtime) |
| `mpris-server` | 0.8 | ⚠️→❌ | **compiles** (`zbus 4.4.0` is portable) but there is no MPRIS consumer or session bus on Windows → gate to Linux, replace with SMTC |
| `pretty_env_logger` | 0.5 | ✅ | compiled |
| `rand` | 0.8 | ✅ | compiled |
| `regex` | 1.11 | ✅ | compiled |
| `reqwest` | 0.12 (default = native-tls) | ✅ | compiled; `native-tls → schannel`; `openssl-sys` not in the Windows tree |
| `sanitize-filename` | 0.6 | ✅ | compiled; applies Windows rules when built for Windows |
| `serde`, `serde_derive`, `serde_json`, `serde_urlencoded` | 1.0 / 0.7 | ✅ | compiled |
| `strum`, `strum_macros` | 0.26 | ✅ | compiled |
| `sys-locale` | 0.3 | ✅ | compiled (uses `GetUserPreferredUILanguages`) |
| `thiserror` | 1.0 | ✅ | compiled |
| `url` | 2.5 | ✅ | compiled |
| `uuid` | 1.11 (`v4`) | ✅ | compiled |
| `gstreamer`, `gstreamer-audio` | 0.23 | ✅ | `gstreamer-sys`, `gstreamer-audio-sys` build scripts OK against GStreamer 1.28.9 |
| `libshumate` (`shumate`) | 0.6 | ✅ | `libshumate-sys` OK against `shumate-1.0` 1.7.0 |
| `gettext-rs` | 0.7 (`gettext-system`) | ✅ | `gettext-sys 0.22.5` OK, links MSYS2 libintl |
| `libadwaita` (`adw`) | 0.7 (`v1_7`) | ✅ | `libadwaita-sys` OK against 1.10.0 |
| `gtk4` (`gtk`) | 0.9 (`gnome_46`) | ✅ | `gtk4-sys` OK against 4.24.1 |

### New crates the port will add (to verify in Phase 4)

| Crate | Purpose | Target gate |
|---|---|---|
| `souvlaki` (or `windows` with `Media_*` features) | SMTC media controls | `cfg(windows)` |
| `gdk4-win32` | HWND of the GTK window (`GdkWin32Surface`) | `cfg(windows)` |
| `tray-icon` (or `windows` `Shell_NotifyIcon`) | tray icon for background playback | `cfg(windows)` |
| `windows` | toasts / `SetThreadExecutionState` / named mutex if GLib's built-ins aren't enough | `cfg(windows)` |
| `winresource` or `embed-resource` | icon, version info, manifest | `[target.'cfg(windows)'.build-dependencies]` |

## `cargo tree` highlights

```text
$ cargo tree -e normal --target x86_64-pc-windows-gnu -i zbus@4.4.0
zbus v4.4.0
└── mpris-server v0.8.1
    └── shortwave v0.0.0

$ cargo tree -e normal --target x86_64-pc-windows-gnu -i zbus@5.7.1
zbus v5.7.1
├── ashpd v0.10.3
├── glycin v2.1.1
└── glycin-utils v3.0.1

$ cargo tree -e normal --target x86_64-pc-windows-gnu -i libseccomp-sys
libseccomp-sys v0.2.1
└── libseccomp v0.3.0
    ├── glycin v2.1.1
    └── glycin-utils v3.0.1

$ cargo tree -e normal --target x86_64-pc-windows-gnu -i openssl-sys
warning: nothing to print.

$ cargo tree -e normal --target x86_64-pc-windows-gnu -i schannel
schannel v0.1.27
└── native-tls v0.2.14
    ├── async-native-tls v0.5.0 └── cast-sender v0.2.0
    ├── hyper-tls v0.6.0        └── reqwest v0.12.16
    └── tokio-native-tls v0.3.1
```

Gating `ashpd`, `glycin` and `mpris-server` to Linux removes **both** zbus
versions, `libseccomp` and the Wayland/X11 GDK crates from the Windows build.

## System libraries / Meson dependencies

From [meson.build:11-30](../../meson.build#L11-L30), checked with `pkg-config --modversion` under UCRT64:

| Meson dependency | Required | Found | Status | MSYS2 package |
|---|---|---|---|---|
| `sqlite3` | ≥ 3.20 | 3.54.1 | ✅ | `mingw-w64-ucrt-x86_64-sqlite` (the prompt's `sqlite3` name no longer exists) |
| `openssl` | ≥ 1.0 | 4.1.0 | ⚠️ | not needed on Windows (SChannel); make the Meson check Linux-only |
| `dbus-1` | any | **missing** | ❌ | none; `meson setup` fails here → Linux-only |
| `glib-2.0`, `gio-2.0` | ≥ 2.66 | 2.90.1 | ✅ | `glib2` (pulled by gtk4) |
| `gtk4` | ≥ 4.16 | 4.24.1 | ✅ | `gtk4` (`gtk4-win32` 4.24.1 also present) |
| `libadwaita-1` | ≥ 1.7 | 1.10.0 | ✅ | `libadwaita` |
| `shumate-1.0` | ≥ 1.3 | 1.7.0 | ✅ | `libshumate` |
| `lcms2` | ≥ 2.12 | 2.20 | ⚠️ | present, but only needed by glycin → Linux-only |
| `libseccomp` | ≥ 2.5 | **missing** | ❌ | Linux-only (glycin) |
| `gstreamer-1.0`, `-base-1.0`, `-audio-1.0` | ≥ 1.16 | 1.28.9 | ✅ | `gstreamer`, `gst-plugins-base` |
| `gstreamer-plugins-base-1.0` | ≥ 1.16 | 1.28.9 | ✅ | `gst-plugins-base` |
| `gstreamer-plugins-bad-1.0`, `gstreamer-bad-audio-1.0` | ≥ 1.16 | 1.28.9 | ✅ | `gst-plugins-bad-libs` |

## MSYS2 packages (`build-aux/windows/setup-msys2.ps1`)

Every name checked with `pacman -Si` / `pacman -Ss` before installing
(`LANG=C` is required: with the Portuguese locale pacman prints localised field
names). Installed versions:

| Prompt name | Actual MSYS2 name | Version | Notes |
|---|---|---|---|
| toolchain | `mingw-w64-ucrt-x86_64-toolchain` (group) | gcc 16.2.0 | |
| rust | `…-rust` | 1.100.0-2 | |
| pkgconf | `…-pkgconf` | 3.0.11 | |
| meson | `…-meson` | 1.12.1 | |
| ninja | `…-ninja` | 1.13.2 | |
| gtk4 | `…-gtk4` | 4.24.1 | |
| libadwaita | `…-libadwaita` | 1.10.0 | |
| gstreamer | `…-gstreamer` | 1.28.9 | |
| gst-plugins-base / good / bad | `…-gst-plugins-{base,good,bad}` | 1.28.9 | `bad` provides `wasapi2sink`, `wasapisink`; `directsoundsink` also present |
| gst-libav | `…-gst-libav` | 1.28.9 | |
| **sqlite3** | **`…-sqlite`** | 3.54.1 | renamed |
| **gettext** | **`…-gettext-runtime` + `…-gettext-tools`** | 1.0 | split package |
| libshumate | `…-libshumate` | 1.7.0 | available → map stays |
| blueprint-compiler | `…-blueprint-compiler` | 0.22.2 | not used by Shortwave 5.0 (plain `.ui`), installed for completeness |
| desktop-file-utils, appstream | as named | 0.29 / 1.2.3 | optional validators |
| *(added)* librsvg | `…-librsvg` | 2.63.4 | SVG icons via gdk-pixbuf |
| *(added)* ntldd | `…-ntldd` | r19 | DLL closure for the bundle |
| *(pulled in)* libsoup3 | `…-libsoup3` | 3.8.1 | needed at runtime by `souphttpsrc` |

## GStreamer runtime sinks

```text
$ gst-inspect-1.0 | grep -iE "wasapi|directsound|autoaudiosink|pulse"
autodetect:  autoaudiosink: Auto audio sink
directsound:  directsoundsink: Direct Sound Audio Sink
wasapi:  wasapisink: WasapiSrc
wasapi2:  wasapi2sink: Wasapi2Sink
```

`pulsesink` is (correctly) absent.

## Toolchain note for analysis

rustup's default host is `x86_64-pc-windows-msvc` and no Visual Studio Build
Tools are installed, so the rustup-provided rust-analyzer (used by Serena) can't
build proc-macros (`proc-macro not yet built`). Builds use MSYS2's GNU
toolchain and are unaffected. See [00-tooling.md](00-tooling.md).
