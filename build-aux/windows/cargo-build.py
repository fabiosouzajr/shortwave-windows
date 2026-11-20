#!/usr/bin/env python3
# Shortwave - build-aux/windows/cargo-build.py
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

"""Run `cargo build` and copy the produced executable to Meson's output.

Used by src/meson.build on Windows only, where Ninja runs commands without a
shell (so the upstream `env … cargo build && cp …` command can't work).
"""

import argparse
import os
import shutil
import subprocess
import sys


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cargo", required=True)
    parser.add_argument("--cargo-home", required=True)
    parser.add_argument("--built", required=True, help="path of the executable cargo produces")
    parser.add_argument("--output", required=True, help="Meson's @OUTPUT@")
    parser.add_argument("cargo_args", nargs=argparse.REMAINDER)
    args = parser.parse_args()

    cargo_args = args.cargo_args
    if cargo_args and cargo_args[0] == "--":
        cargo_args = cargo_args[1:]

    # Strip release binaries (no effect on debug builds); Linux builds don't use
    # this helper, so upstream's profile is unchanged there.
    env = dict(os.environ, CARGO_HOME=args.cargo_home, CARGO_PROFILE_RELEASE_STRIP="symbols")
    result = subprocess.run([args.cargo, "build", *cargo_args], env=env)
    if result.returncode != 0:
        return result.returncode

    shutil.copy2(args.built, args.output)
    return 0


if __name__ == "__main__":
    sys.exit(main())
