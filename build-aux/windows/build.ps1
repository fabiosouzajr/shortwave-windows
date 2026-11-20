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
    Builds Shortwave for Windows with Meson inside MSYS2 UCRT64.

.DESCRIPTION
    Runs `meson setup` (or reconfigure), `meson compile` and `meson install`
    in the MSYS2 UCRT64 environment. Run setup-msys2.ps1 once beforehand.

.PARAMETER BuildProfile
    'default' (release build) or 'development' (debug build, app id *.Devel).

.PARAMETER BuildDir
    Meson build directory, relative to the repository root. Default: _build

.PARAMETER Prefix
    Install prefix. Default: <BuildDir>\install

.PARAMETER Reconfigure
    Wipe and re-run `meson setup`.

.PARAMETER Run
    Start the installed shortwave.exe after building.
#>
[CmdletBinding()]
param(
    [ValidateSet('default', 'development')]
    [string]$BuildProfile = 'default',
    [string]$BuildDir = '_build',
    [string]$Prefix,
    [string]$Msys2Root = 'C:\msys64',
    [switch]$Reconfigure,
    [switch]$Run
)

$ErrorActionPreference = 'Stop'

$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$bash = Join-Path $Msys2Root 'usr\bin\bash.exe'
if (-not (Test-Path $bash)) { throw "MSYS2 not found at $Msys2Root (run setup-msys2.ps1)" }

$buildPath = Join-Path $repo $BuildDir
if (-not $Prefix) { $Prefix = Join-Path $buildPath 'install' }

# Meson (a native UCRT64 Python program) accepts Windows paths with forward slashes.
function ConvertTo-MesonPath([string]$Path) { $Path -replace '\\', '/' }

function Invoke-Msys([string]$Command) {
    $env:MSYSTEM = 'UCRT64'
    $env:CHERE_INVOKING = '1'
    # Append the Windows PATH after MSYS2's own entries so Meson can find
    # Git for Windows (used for the VCS tag); MSYS2's cargo/rustc stay first.
    $env:MSYS2_PATH_TYPE = 'inherit'
    Push-Location $repo
    try {
        & $bash -lc $Command
        if ($LASTEXITCODE -ne 0) { throw "Command failed ($LASTEXITCODE): $Command" }
    } finally {
        Pop-Location
    }
}

$b = ConvertTo-MesonPath $BuildDir
$p = ConvertTo-MesonPath $Prefix

if ($Reconfigure -and (Test-Path $buildPath)) {
    Invoke-Msys "meson setup --wipe '$b' -Dprofile=$BuildProfile --prefix='$p'"
} elseif (-not (Test-Path (Join-Path $buildPath 'build.ninja'))) {
    Invoke-Msys "meson setup '$b' -Dprofile=$BuildProfile --prefix='$p'"
} else {
    Invoke-Msys "meson configure '$b' -Dprofile=$BuildProfile --prefix='$p'"
}

Invoke-Msys "meson compile -C '$b'"
Invoke-Msys "meson install -C '$b' --quiet"

# CA certificates for GIO's TLS backends (see platform::init); same layout as the bundle
$caSource = Join-Path $Msys2Root 'ucrt64\etc\ssl\certs\ca-bundle.crt'
$caTarget = Join-Path $Prefix 'etc\ssl\certs'
New-Item -ItemType Directory -Force -Path $caTarget | Out-Null
Copy-Item $caSource $caTarget -Force

$exe = Join-Path $Prefix 'bin\shortwave.exe'
Write-Host "==> Installed: $exe"

if ($Run) {
    # Until the bundle exists (Phase 5), the GTK/GStreamer DLLs come from MSYS2.
    $env:PATH = (Join-Path $Msys2Root 'ucrt64\bin') + ';' + $env:PATH
    & $exe
}
