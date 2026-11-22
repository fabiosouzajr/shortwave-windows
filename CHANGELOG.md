# Changelog

Changes in the unofficial Windows port of
[Shortwave](https://gitlab.gnome.org/World/Shortwave). Each release names the
upstream version it is based on; `-win.N` counts the port's own releases.

## [5.0.0-win.1] - 2026-11-22

First release of the Windows port, based on Shortwave 5.0.0.

### Added

**Windows support**
- Runs natively on Windows 10 and 11 (x64), built with GTK 4, libadwaita
  and GStreamer from MSYS2.
- Audio through WASAPI, with a volume control that works on any output.
- HTTPS streams work out of the box (a CA bundle ships with the app).
- Media keys and the Windows media overlay show the station and track and
  control playback.
- Tray icon: closing the window keeps the radio playing; Show Window, Stop
  and Quit from the tray menu.
- Toast notifications for new tracks, with Save Track / Don't Record
  buttons (installed version; the portable zip uses plain notifications).
- Recording saves to `Music\Shortwave` with file names that are valid on
  Windows.
- Settings in the registry, data and cover cache in
  `%LOCALAPPDATA%\Shortwave`.
- Dark and light mode follow Windows; app icon, version information,
  HiDPI support, no console window.
- Installer (per-user or per-machine) and a portable zip.

**New features**
- Search by station name *or* genre, and filter by country, codec and
  minimum bitrate. Filters work without typing anything, e.g. all AAC+
  stations in Brazil.
- Heart button in the player: one click saves the playing station to your
  library.
- Country flags on station cards.
- Station cards show bigger logos, a bold name, and a codec and bitrate
  meter.
- Station logos are found on the station's homepage when the directory has
  none.
- Choose the app language in Preferences (takes effect after a restart).

### Changed
- Larger "Popular Stations" and "Discover New Stations" headings.
- The station map is replaced by a link to OpenStreetMap on Windows.
- AAC is decoded with `faad`, which keeps the download about 90 MB smaller.

### Fixed
- Recording works with WASAPI.
- PNG-compressed `.ico` station logos are displayed.
- Logo lookups give up after a bounded time, and temporary network failures
  are retried later instead of being remembered as "no logo".

### Known issues
- The features added last (search filters, the heart button, country flags,
  the language picker and the larger headings) pass the automated tests but
  haven't yet been checked by hand in the running app.
- Dark mode following Windows: only light mode has been tested.
- Hardware media keys weren't pressed in testing; the Windows media overlay,
  which uses the same controls, works.
- Clicking the Save Track / Don't Record buttons on a toast is untested.
- Chromecast: devices are discovered, but casting itself is untested.
- The per-machine install (with its firewall rule) is untested; per-user
  install and uninstall are verified.
- System proxy settings aren't used; only `http_proxy`-style environment
  variables are.
- A few new menu entries ("Show Window", "Hide Window", "Quit", "Location")
  appear in English in most languages; the new search, filter and heart
  labels appear in English in all languages.
- From upstream: saved recordings miss their last ~4 seconds; the last played
  station isn't restored if it isn't in your library.
- The bitrate filter leaves out stations that don't report a bitrate.

[5.0.0-win.1]: https://github.com/fabiosouzajr/shortwave-windows/releases/tag/v5.0.0-win.1
