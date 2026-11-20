# Shortwave - Windows port
# Copyright 2026 The Shortwave Windows port contributors
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# SPDX-License-Identifier: GPL-3.0-or-later

<#
.SYNOPSIS
    Builds the Inno Setup installer from dist\Shortwave.

.PARAMETER Bundle
    Run bundle.ps1 first (otherwise the existing dist\Shortwave is used).
#>
[CmdletBinding()]
param([switch]$Bundle)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

if ($Bundle) { & (Join-Path $PSScriptRoot 'bundle.ps1') }

$exe = Join-Path $repo 'dist\Shortwave\bin\shortwave.exe'
if (-not (Test-Path $exe)) { throw "No bundle found, run bundle.ps1 first ($exe)" }
$version = (Get-Item $exe).VersionInfo.ProductVersion

$iscc = @(
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $iscc) { throw 'Inno Setup 6 not found (winget install JRSoftware.InnoSetup)' }

& $iscc /Q "/DAppVersion=$version" (Join-Path $repo 'packaging\windows\shortwave.iss')
if ($LASTEXITCODE -ne 0) { throw "ISCC failed ($LASTEXITCODE)" }
Get-Item (Join-Path $repo "dist\Shortwave-$version-windows-x64-setup.exe")
