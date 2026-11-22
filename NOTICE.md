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

## Country flags

The flag images in `data/flags/` are the two-letter country flags from
[flag-icons](https://github.com/lipis/flag-icons) v7.6.0 (`flags/4x3/`),
under the MIT License:

```
The MIT License (MIT)

Copyright (c) 2013 Panayiotis Lipiridis

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
