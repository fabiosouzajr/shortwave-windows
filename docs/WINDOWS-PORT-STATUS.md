# Windows port: status

State of the unofficial Windows port of Shortwave 5.0 on branch `main`
(public repository `fabiosouzajr/shortwave-windows`), as of 2026-11-24,
release `5.0.0-win.2`. The verification checklist below was run on
2026-11-20; features added after it are listed under "Not working / not
tested" until they have been checked in the running app. Design:
[PORTING.md](PORTING.md). Analysis: [re/](re/). Build instructions:
[BUILDING-WINDOWS.md](BUILDING-WINDOWS.md). Changes per release:
[CHANGELOG.md](../CHANGELOG.md).

## Verification checklist

Everything below was run on Windows 11 Pro (x64) with MSYS2 UCRT64, or in
GitHub Actions. "Evidence" names what was observed, not what is expected.

| Check | Result | Evidence |
|---|---|---|
| Clean build from a fresh clone (`setup-msys2.ps1` then `build.ps1`) | ✅ | CI run 44802845477, Windows job: fresh `windows-latest` runner, setup script, Meson release build, clippy, tests, bundle, installer, all green (24 min cold, 10 min cached in run 44806798067) |
| Linux build still compiles | ✅ | CI run 44806798067, Linux job (Fedora 42): `meson setup` + `meson compile`, clippy, tests green |
| Search returns stations, a station plays audibly, titles update | ✅ | Search for "Venenos", "MPBdoVinil", "Bandeirantes" listed results; WASAPI session meter for `shortwave.exe` peaked 0.3–0.6; user confirmed audio; `Stream title has changed to: …` in logs. HTTP MP3, HTTPS (zeno.fm) and HLS AAC+ (streamtheworld) all played |
| Favourites survive a restart, covers load | ✅ | Two stations added via the station dialog → `Loaded 2 item(s)` after restart; sort order persisted in the registry; covers incl. PNG-in-ICO favicons shown |
| Recording saves a playable file with a valid Windows name | ✅ | `Music\Shortwave\Ray Conniff - Superstar, Nature Boy.ogg`: gst-discoverer reports Vorbis 44.1 kHz stereo, decodes to EOS; file-name rules covered by unit tests (reserved names, invalid characters, MAX_PATH) |
| Media keys and the media overlay control playback and show station/track | ✅ (partly by proxy) | `GlobalSystemMediaTransportControlsSessionManager` showed status Playing, title = station, artist = track, thumbnail; toggling that session paused/resumed playback (events `Pause`/`Play` in the log). The physical media keys use the same SMTC button events but weren't pressed (another app owned a session) |
| Close-to-tray keeps playing, tray menu works | ✅ | Window closed while playing → process kept running, audio meter 0.19–0.37; tray menu Show Window (recreates the window, SMTC re-attaches), Stop, Quit all worked |
| Chromecast discovered and casting works | ⚠️ discovery only | No Cast device available. A fake `_googlecast._tcp` advertiser was discovered and listed; connection errors are reported cleanly. TLS handshake, receiver launch and media `LOAD` are **untested** |
| Bundle runs on a clean machine | ✅ (PATH-stripped, agreed) | Relocated copy, only System32 on `PATH`, fresh GStreamer registry: 0 modules from `C:\msys64`, both stream types audible, UI/fonts/icons/translations fine. Windows Sandbox/VM test not done (agreed) |
| Installer installs and uninstalls cleanly | ✅ per-user / ⚠️ per-machine | Silent per-user install: 1691 files, shortcut with AUMID, app runs from it; uninstall removes program, shortcut and uninstall entry, keeps user data. The per-machine path (UAC, firewall rule) was not run |
| Repository published, work pushed | ✅ | Private during development (`windows-port` pushed to `origin`); made public on 2026-11-22 for the first release after a history scan for secrets found none: `gh repo view --json visibility` → `PUBLIC`, release page reachable without login (HTTP 200). Development is on `main` |
| CI publishes a release when a version tag is pushed | ✅ | CI run 45453014882 for tag `v5.0.0-win.2`: Windows, Linux and `Publish release` jobs green; release "Shortwave for Windows 5.0.0-win.2" has `Shortwave-5.0.0-win.2-windows-x64-setup.exe` and `Shortwave-5.0.0-win.2-windows-x64.zip` |

## What works on Windows

- Search, browse, play (HTTP/HTTPS/HLS; MP3, AAC/HE-AAC, Opus, Vorbis, FLAC),
  volume, buffering, track titles, past tracks.
- Library (favourites, local stations, sorting, metadata refresh), covers and
  cover cache.
- Recording: detection, min/max duration, recording modes, save/discard,
  track dialog, Windows-safe file names, default folder `Music\Shortwave`.
- Media overlay / SMTC, tray icon with background playback, single instance,
  suspend inhibition while playing, toast notifications (installed build).
- Settings in the registry, translations (incl. pt_BR), dark/light following
  Windows, HiDPI manifest, app icon and version info, no console window.
- Self-contained bundle (124 MB, 50 MB zip), Inno Setup installer (33 MB), CI.

Added since the checklist (2026-11-21/22; unit-tested, see below for
in-app status):

- Station cards: bigger logos, logos found on the station's homepage, bold
  name, codec and bitrate meter, country flags (bundled, flag-icons v7.6.0).
- Search by name or genre with country, codec and minimum-bitrate filters.
- Heart button in the player view that saves the playing station.
- Larger Browse headings; app language selectable in Preferences.

## Not working / not tested

| Item | Status | Issue |
|---|---|---|
| Station map | Hidden on Windows (libshumate never requests tiles in this dialog); a "Location" row links to openstreetmap.org instead | [#3](https://github.com/fabiosouzajr/shortwave-windows/issues/3) |
| Toast buttons (Save Track / Don't Record) | Shown, translated; **clicking them is unverified**: the live toast timed out before the test click, and a click in the Notification Centre relaunches the app instead of running the action | [#7](https://github.com/fabiosouzajr/shortwave-windows/issues/7) |
| Chromecast end-to-end | Untested (no device) | [#11](https://github.com/fabiosouzajr/shortwave-windows/issues/11) |
| Per-machine install + firewall rule | Untested (needs UAC) | [#4](https://github.com/fabiosouzajr/shortwave-windows/issues/4) |
| Hardware media keys | Not pressed physically (SMTC path verified) | [#5](https://github.com/fabiosouzajr/shortwave-windows/issues/5) |
| Windows Sandbox / VM clean test | Not done; PATH-stripped test used instead | — |
| Features added 2026-11-21/22 (see "What works") | Unit tests and builds pass; not yet recorded as checked in the running app: flag rendering, heart states, search filters and the country search, dropdown labels, language switch | [#9](https://github.com/fabiosouzajr/shortwave-windows/issues/9) |
| Translations of the 2026-11-22 strings | Search filter, heart and "Station added" strings are untranslated in all languages | [#8](https://github.com/fabiosouzajr/shortwave-windows/issues/8) |
| Dark theme switch | Light verified; libadwaita reports system colour-scheme support | [#10](https://github.com/fabiosouzajr/shortwave-windows/issues/10) |
| System proxy | GIO proxy modules aren't bundled; `http_proxy`-style environment variables only | [#6](https://github.com/fabiosouzajr/shortwave-windows/issues/6) |
| Translations of the new strings | "Show Window", "Hide Window", "Quit", "Location" appear in English where upstream has no translation | [#8](https://github.com/fabiosouzajr/shortwave-windows/issues/8) |

## Known issues

Upstream behaviour, unchanged (same on Linux):

- Saved recordings miss the last ~4 s (the file is copied before the encoder
  is flushed).
- The last played station isn't restored if it isn't in the library.
- libadwaita 1.10 warns that `style-dark.css` is deprecated.
- `openssl-sys 0.9.108` in upstream's `Cargo.lock` doesn't build against
  OpenSSL 4 (newer Linux distributions); CI pins Fedora 42.

Port-specific:

- Clippy reports 17 warnings, all in upstream code (newer lints); left for parity.
- Leftover inbound firewall rules for development paths can appear on a
  developer machine (Windows prompts on first run of each exe path).

## Deviations from the original prompt

1. Gating uses `cfg(not(windows))` instead of `cfg(target_os = "linux")`, so
   other Unix builds keep upstream's code.
2. `gdk4-win32` was dropped (doesn't link against GTK 4.24);
   `gdk_win32_surface_get_handle` is declared directly.
3. Single instance and suspend inhibition use GLib/GTK's built-in Windows
   support; no named mutex or `SetThreadExecutionState` code was needed.
4. GNotification stays as the fallback; toasts are used when the installer's
   AppUserModelID shortcut exists.
5. The recorder bin gained an `audioconvert` on every platform (needed for
   WASAPI; a pass-through on Linux).
6. AAC is decoded by `faad` instead of `gst-libav` (saves ~90 MB); `fdkaac` is
   not bundled because its license isn't GPL-compatible.
7. A CA bundle ships with the app and `SSL_CERT_FILE` is set at start-up (GIO
   TLS otherwise misses roots Windows hasn't fetched yet).
8. The station map is replaced by an OpenStreetMap link on Windows (agreed).
9. MSYS2 package names: `sqlite` (not `sqlite3`), `gettext-runtime` +
   `gettext-tools` (not `gettext`).
10. Data directory keeps upstream's casing: `%LOCALAPPDATA%\Shortwave`.
11. The CI Linux job uses Fedora 42 instead of the latest Fedora (OpenSSL 4).

## Release process

Versions are the upstream version plus `-win.N` for the port's own releases;
the tag is `v` + version. History:

| Release | Published | How |
|---|---|---|
| `5.0.0-win.1` (2026-11-22) | [Releases page](https://github.com/fabiosouzajr/shortwave-windows/releases/tag/v5.0.0-win.1) | By hand: built locally (`build.ps1`, `bundle.ps1`, `installer.ps1`), tagged, `gh release create` with the installer and zip |

Later releases are published by CI: pushing a `v<version>` tag runs the
workflow, and its `release` job publishes the installer and zip from the
Windows job once the Windows and Linux jobs pass. It refuses to publish if the
built files don't carry the tag's version or `docs/releases/<version>.md` is
missing (`build-aux/publish-release.sh`, tested by
`build-aux/test-publish-release.sh`). Steps: set the version in `meson.build`,
write the release notes, move the `Unreleased` changelog entries under the new
version, push to `main`, then tag and push the tag. See
[BUILDING-WINDOWS.md](BUILDING-WINDOWS.md#releasing).

Release notes list anything in the "Not working / not tested" table above
that a user would hit as a known issue; keep the two in step.

## Merging future upstream releases

```powershell
git fetch upstream          # Mauna Linux packaging (re-imports releases)
git fetch gnome --tags      # GNOME upstream (the actual development)
git checkout main
git merge upstream/main     # or: git merge <gnome tag>
```

Conflicts are expected only where the port touched upstream files (all
small, marked with `platform::` calls or `cfg(windows)`): `Cargo.toml`,
`meson.build`, `src/meson.build`, `data/meson.build`, `src/main.rs`,
`src/app.rs`, `src/path.rs`, `src/utils.rs`, `src/audio/{mod,player,track,
gstreamer_backend}.rs`, `src/api/cover_loader.rs`, `src/ui/{window,
station_dialog}.rs`. Platform code lives in `src/platform/`. After a merge, run
the CI workflow (or `build.ps1` + `bundle.ps1`) and re-check the items above.
Push only to `origin`; both `upstream` and `gnome` have pushing disabled.
