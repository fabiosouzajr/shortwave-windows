# Shortwave - Windows port
# Copyright 2026 The Shortwave Windows port contributors
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
#
# SPDX-License-Identifier: GPL-3.0-or-later

<#
.SYNOPSIS
    Installs the MSYS2 UCRT64 packages needed to build Shortwave on Windows.

.DESCRIPTION
    Updates MSYS2, then installs the toolchain, Rust, GTK4, libadwaita,
    GStreamer, SQLite, gettext and the build tools with `pacman -S --needed`.
    Safe to re-run.

.PARAMETER Msys2Root
    MSYS2 install location. Defaults to C:\msys64.

.PARAMETER SkipUpdate
    Skip the full system upgrade (pacman -Syu) and only install packages.
#>
[CmdletBinding()]
param(
    [string]$Msys2Root = 'C:\msys64',
    [switch]$SkipUpdate
)

$ErrorActionPreference = 'Stop'

$bash = Join-Path $Msys2Root 'usr\bin\bash.exe'
if (-not (Test-Path $bash)) {
    throw "MSYS2 not found at $Msys2Root. Install it with: winget install MSYS2.MSYS2"
}

# Package names were checked with `pacman -Si` / `pacman -Ss` (see docs/re/09-dependency-matrix.md).
# Note: in current MSYS2, sqlite3 is `sqlite` and gettext is split into -runtime/-tools.
$packages = @(
    'toolchain'            # gcc, binutils, make, ... (package group)
    'rust'
    'pkgconf'
    'meson'
    'ninja'
    'gtk4'
    'libadwaita'
    'libshumate'           # station map
    'gstreamer'
    'gst-plugins-base'
    'gst-plugins-good'
    'gst-plugins-bad'      # wasapi2sink, hls
    'gst-libav'
    'sqlite'
    'gettext-runtime'
    'gettext-tools'
    'blueprint-compiler'
    'desktop-file-utils'
    'appstream'
    'librsvg'              # SVG icons through gdk-pixbuf
    'ntldd'                # DLL dependency resolution for bundling
) | ForEach-Object { "mingw-w64-ucrt-x86_64-$_" }

function Invoke-Msys([string]$Command) {
    $env:MSYSTEM = 'UCRT64'
    $env:CHERE_INVOKING = '1'
    $env:LANG = 'C'
    & $bash -lc $Command
    if ($LASTEXITCODE -ne 0) { throw "MSYS2 command failed ($LASTEXITCODE): $Command" }
}

if (-not $SkipUpdate) {
    Write-Host '==> Updating MSYS2 (pacman -Syu). Run it twice: the first pass may only update the core.'
    Invoke-Msys 'pacman -Syu --noconfirm'
    Invoke-Msys 'pacman -Syu --noconfirm'
}

Write-Host "==> Installing $($packages.Count) packages"
Invoke-Msys ("pacman -S --needed --noconfirm " + ($packages -join ' '))

Write-Host '==> Installed versions'
Invoke-Msys 'rustc --version; cargo --version; meson --version; pkg-config --modversion gtk4 libadwaita-1 gstreamer-1.0 sqlite3 shumate-1.0'
