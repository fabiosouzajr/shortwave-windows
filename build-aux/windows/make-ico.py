#!/usr/bin/env python3
# Shortwave - build-aux/windows/make-ico.py
# Copyright (C) 2026  The Shortwave Windows port contributors
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

"""Build a multi-size Windows .ico from the app's scalable SVG icon.

Run from an MSYS2 UCRT64 shell (needs rsvg-convert from librsvg):

    python build-aux/windows/make-ico.py \
        data/icons/hicolor/scalable/apps/de.haeckerfelix.Shortwave.svg \
        build-aux/windows/shortwave.ico

Every image is stored as PNG, which Windows Vista and later support.
"""

import struct
import subprocess
import sys

SIZES = [16, 20, 24, 32, 40, 48, 64, 96, 128, 256]


def render(svg: str, size: int) -> bytes:
    return subprocess.run(
        ["rsvg-convert", "--width", str(size), "--height", str(size), svg],
        check=True,
        capture_output=True,
    ).stdout


def main() -> int:
    svg, out = sys.argv[1], sys.argv[2]
    images = [(size, render(svg, size)) for size in SIZES]

    header = struct.pack("<HHH", 0, 1, len(images))
    offset = len(header) + 16 * len(images)
    entries, data = b"", b""
    for size, png in images:
        dim = 0 if size >= 256 else size  # 0 means 256
        entries += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(png), offset)
        data += png
        offset += len(png)

    with open(out, "wb") as f:
        f.write(header + entries + data)
    print(f"wrote {out}: {', '.join(str(s) for s in SIZES)} px")
    return 0


if __name__ == "__main__":
    sys.exit(main())
