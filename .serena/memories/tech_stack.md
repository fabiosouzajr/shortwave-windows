# Tech stack

- Rust edition 2021; build via Meson (wraps `cargo build` as custom_target, CARGO_HOME in build dir).
- Windows toolchain: MSYS2 UCRT64 only (rust 1.100, gcc 16, pkgconf, meson 1.12, GTK 4.24, libadwaita 1.10, GStreamer 1.28, libshumate 1.7, sqlite 3.54). Installed by `build-aux/windows/setup-msys2.ps1`.
- rustup on this machine is MSVC host without VS Build Tools: rustup cargo is fine for `cargo tree`, NOT for builds. Serena's rust-analyzer can't build proc-macros (glib::clone!, Properties, CompositeTemplate, diesel) -> macro-expanded references are missing.
- MSYS2 package renames vs. common docs: `sqlite3` -> `sqlite`; `gettext` -> `gettext-runtime` + `gettext-tools`.
- pacman output is localised (pt locale): always `LANG=C` when parsing.
- Linux-only crates that break the Windows build: `ashpd` (gtk4 feature -> gdk4-wayland/x11-sys), `glycin` (libseccomp). `mpris-server` compiles but is useless on Windows. TLS: native-tls -> SChannel on Windows (no OpenSSL).
- Meson blockers on Windows: `dependency('dbus-1')`, `libseccomp`; cargo output is `shortwave.exe` (cp step).
