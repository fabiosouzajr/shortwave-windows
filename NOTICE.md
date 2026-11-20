# Notice

This repository is an **unofficial Windows port** of
[Shortwave](https://gitlab.gnome.org/World/Shortwave), the internet radio
player for GNOME written by **Felix Häcker** and contributors.

The source this port started from is
[Mauna Linux's packaging of Shortwave 5.0](https://github.com/maunalinux/shortwave),
which tracks GNOME upstream.

- It is not affiliated with, endorsed by, or supported by the Shortwave
  authors, the GNOME project, or Mauna Linux. Please report problems with the
  Windows build here, not upstream.
- The application name, app ID (`de.haeckerfelix.Shortwave`), icons and the
  Google Cast receiver ID are upstream's and are kept unchanged.
- All original copyright headers and author credits are preserved. Files added
  for the port carry the same license header.

## License

Shortwave, and therefore this port, is licensed under the
**GNU General Public License v3.0 or later**. See [COPYING.md](COPYING.md).

The Windows distribution bundles third-party libraries (GTK, libadwaita,
GLib, GStreamer and its plugins, libshumate, SQLite, librsvg and their
dependencies, from MSYS2). Each is distributed under its own license, mostly
LGPL-2.1-or-later; their license texts ship with the bundle.

Station data comes from the community-run [radio-browser.info](https://www.radio-browser.info/) API.
