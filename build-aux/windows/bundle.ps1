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
    Creates a self-contained Shortwave folder (dist\Shortwave) and a portable zip.

.DESCRIPTION
    Builds a release (build.ps1), copies the install prefix and adds everything
    the app needs from MSYS2 UCRT64 at runtime: the DLL closure (resolved with
    ntldd -R), selected GStreamer plugins, gdk-pixbuf loaders, the GIO TLS
    module, GLib/GTK schemas, icon themes, translations, fontconfig config,
    CA certificates and the licenses of every bundled MSYS2 package.

.PARAMETER SkipBuild
    Reuse the existing _build\install instead of rebuilding.
#>
[CmdletBinding()]
param(
    [string]$Msys2Root = 'C:\msys64',
    [string]$OutDir,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$ucrt = Join-Path $Msys2Root 'ucrt64'
$bash = Join-Path $Msys2Root 'usr\bin\bash.exe'
if (-not $OutDir) { $OutDir = Join-Path $repo 'dist' }
$dist = Join-Path $OutDir 'Shortwave'
$install = Join-Path $repo '_build\install'

function Invoke-Msys([string]$Command) {
    $env:MSYSTEM = 'UCRT64'; $env:CHERE_INVOKING = '1'; $env:LANG = 'C'
    $out = & $bash -lc $Command
    if ($LASTEXITCODE -ne 0) { throw "MSYS2 command failed ($LASTEXITCODE): $Command" }
    $out
}
function ConvertTo-MsysPath([string]$Path) { (Invoke-Msys "cygpath -u '$($Path -replace '\\', '/')'") }
function Copy-Into([string]$Source, [string]$TargetDir) {
    New-Item -ItemType Directory -Force -Path $TargetDir | Out-Null
    Copy-Item -Path $Source -Destination $TargetDir -Force -Recurse
}

# --- What to bundle ---------------------------------------------------------

# GStreamer plugins (see docs/re/05-audio-pipeline.md)
$gstPlugins = @(
    'coreelements', 'playback', 'typefindfunctions', 'audioconvert', 'audioresample',
    'autodetect', 'volume', 'wasapi', 'wasapi2',                 # output
    'soup', 'icydemux', 'id3demux', 'apetag', 'audioparsers',    # sources, metadata, parsers
    # Decoders (+ vorbisenc/oggmux for recording). AAC/HE-AAC uses faad (faad2,
    # GPL-2.0-or-later) instead of gst-libav, which pulls in ~90 MB of FFmpeg
    # video codecs; fdkaac is avoided because its license isn't GPL-compatible.
    'mpg123', 'flac', 'opus', 'vorbis', 'ogg', 'faad',
    'adaptivedemux2', 'hls', 'mpegtsdemux', 'isomp4'             # HLS / DASH
) | ForEach-Object { "libgst$_.dll" }

# gdk-pixbuf loaders (GTK decodes PNG/JPEG/TIFF itself; these cover favicons and icons)
$pixbufLoaders = @(
    'libpixbufloader-png.dll', 'libpixbufloader-jpeg.dll', 'libpixbufloader-gif.dll',
    'libpixbufloader-ico.dll', 'libpixbufloader-bmp.dll', 'libpixbufloader-tiff.dll',
    'libpixbufloader-webp.dll', 'pixbufloader_svg.dll'
)

# Translation domains
$localeDomains = @('shortwave', 'gtk40', 'libadwaita', 'glib20', 'glib-networking', 'shumate1',
    'gstreamer-1.0', 'gst-plugins-base-1.0', 'gst-plugins-good-1.0', 'gst-plugins-bad-1.0')

# --- Build --------------------------------------------------------------------

if (-not $SkipBuild) {
    & (Join-Path $PSScriptRoot 'build.ps1') -BuildProfile default
}
if (-not (Test-Path (Join-Path $install 'bin\shortwave.exe'))) { throw "No build found in $install" }

Write-Host "==> Creating $dist"
if (Test-Path $dist) { Remove-Item -Recurse -Force $dist }
New-Item -ItemType Directory -Force -Path $dist | Out-Null
Copy-Item -Path (Join-Path $install '*') -Destination $dist -Recurse -Force

# --- Executables, plugins and modules -------------------------------------------

$bin = Join-Path $dist 'bin'
Copy-Into (Join-Path $ucrt 'bin\gdbus.exe') $bin    # session bus for single instance
Copy-Into (Join-Path $ucrt 'bin\gdk-pixbuf-query-loaders.exe') $bin

$gstDir = Join-Path $dist 'lib\gstreamer-1.0'
foreach ($p in $gstPlugins) { Copy-Into (Join-Path $ucrt "lib\gstreamer-1.0\$p") $gstDir }
Copy-Into (Join-Path $ucrt 'libexec\gstreamer-1.0\gst-plugin-scanner.exe') (Join-Path $dist 'libexec\gstreamer-1.0')

$loaderDir = Join-Path $dist 'lib\gdk-pixbuf-2.0\2.10.0\loaders'
foreach ($l in $pixbufLoaders) { Copy-Into (Join-Path $ucrt "lib\gdk-pixbuf-2.0\2.10.0\loaders\$l") $loaderDir }

Copy-Into (Join-Path $ucrt 'lib\gio\modules\libgiognutls.dll') (Join-Path $dist 'lib\gio\modules')

# --- DLL closure ----------------------------------------------------------------

Write-Host '==> Resolving DLL dependencies (ntldd -R)'
$roots = @(Get-ChildItem $bin -Filter *.exe) + @(Get-ChildItem (Join-Path $dist 'lib') -Recurse -Filter *.dll) +
    @(Get-ChildItem (Join-Path $dist 'libexec') -Recurse -Filter *.exe)
$ucrtBin = (Join-Path $ucrt 'bin').ToLower()
$dlls = @{}
foreach ($root in $roots) {
    $lines = Invoke-Msys "ntldd -R '$(ConvertTo-MsysPath $root.FullName)'"
    foreach ($line in $lines) {
        if ($line -match '=>\s+(.+?\.dll)\s+\(0x') {
            $path = $Matches[1]
            if ($path.ToLower().StartsWith($ucrtBin)) { $dlls[$path.ToLower()] = $path }
        }
    }
}
foreach ($dll in $dlls.Values) { Copy-Item $dll $bin -Force }
Write-Host "    $($dlls.Count) DLLs"

# --- Data -----------------------------------------------------------------------

Write-Host '==> Schemas, icons, translations, fonts, certificates'
$schemaDir = Join-Path $dist 'share\glib-2.0\schemas'
Get-ChildItem (Join-Path $ucrt 'share\glib-2.0\schemas') -Filter 'org.gtk.gtk4.Settings.*.gschema.xml' |
    ForEach-Object { Copy-Item $_.FullName $schemaDir -Force }
Remove-Item (Join-Path $schemaDir 'gschemas.compiled') -ErrorAction SilentlyContinue
Invoke-Msys "glib-compile-schemas '$(ConvertTo-MsysPath $schemaDir)'" | Out-Null

$iconDir = Join-Path $dist 'share\icons'
Copy-Into (Join-Path $ucrt 'share\icons\Adwaita') $iconDir
Remove-Item -Recurse -Force (Join-Path $iconDir 'Adwaita\cursors') -ErrorAction SilentlyContinue
Copy-Item (Join-Path $ucrt 'share\icons\hicolor\index.theme') (Join-Path $iconDir 'hicolor') -Force
foreach ($theme in 'Adwaita', 'hicolor') {
    Invoke-Msys "gtk4-update-icon-cache -q -t -f '$(ConvertTo-MsysPath (Join-Path $iconDir $theme))'" | Out-Null
}

foreach ($lang in Get-ChildItem (Join-Path $ucrt 'share\locale') -Directory) {
    foreach ($domain in $localeDomains) {
        $mo = Join-Path $lang.FullName "LC_MESSAGES\$domain.mo"
        if (Test-Path $mo) { Copy-Into $mo (Join-Path $dist "share\locale\$($lang.Name)\LC_MESSAGES") }
    }
}

Copy-Into (Join-Path $ucrt 'etc\fonts') (Join-Path $dist 'etc')
Copy-Into (Join-Path $ucrt 'etc\ssl\certs\ca-bundle.crt') (Join-Path $dist 'etc\ssl\certs')

# --- Caches (paths relative to the prefix, so the folder stays relocatable) --------

$cache = Invoke-Msys "cd '$(ConvertTo-MsysPath $dist)' && GDK_PIXBUF_MODULEDIR=lib/gdk-pixbuf-2.0/2.10.0/loaders bin/gdk-pixbuf-query-loaders.exe"
$prefixWin = ($dist -replace '\\', '\\') + '\\'
$cache = $cache | ForEach-Object { $_ -replace [regex]::Escape($prefixWin), '' -replace '^"lib/', '"lib\\' }
Set-Content -Path (Join-Path $dist 'lib\gdk-pixbuf-2.0\2.10.0\loaders.cache') -Value $cache -Encoding ascii
Remove-Item (Join-Path $bin 'gdk-pixbuf-query-loaders.exe')
Invoke-Msys "gio-querymodules '$(ConvertTo-MsysPath (Join-Path $dist 'lib\gio\modules'))'" | Out-Null

# --- Licenses -------------------------------------------------------------------

Write-Host '==> Licenses'
Copy-Item (Join-Path $repo 'COPYING.md'), (Join-Path $repo 'NOTICE.md') $dist -Force
$owned = Get-ChildItem $dist -Recurse -File | Where-Object { $_.Extension -in '.dll', '.exe' } | ForEach-Object {
    $src = Join-Path $ucrt ($_.FullName.Substring($dist.Length + 1))
    $alt = Join-Path $ucrt ('bin\' + $_.Name)
    if (Test-Path $src) { $src } elseif (Test-Path $alt) { $alt }
}
$packages = @{}
foreach ($file in $owned) {
    $p = Invoke-Msys "pacman -Qoq '$(ConvertTo-MsysPath $file)' 2>/dev/null || true"
    if ($p) { $packages[$p] = $true }
}
$licenseDir = Join-Path $dist 'share\licenses'
foreach ($pkg in $packages.Keys) {
    $name = $pkg -replace '^mingw-w64-ucrt-x86_64-', ''
    $src = Join-Path $ucrt "share\licenses\$name"
    if (Test-Path $src) { Copy-Into $src $licenseDir }
}
$packages.Keys | Sort-Object | Set-Content (Join-Path $licenseDir 'PACKAGES.txt')
Write-Host "    $($packages.Count) MSYS2 packages"

# --- Zip --------------------------------------------------------------------------

$version = (Get-Item (Join-Path $bin 'shortwave.exe')).VersionInfo.ProductVersion
$zip = Join-Path $OutDir "Shortwave-$version-windows-x64.zip"
Remove-Item $zip -ErrorAction SilentlyContinue
Compress-Archive -Path $dist -DestinationPath $zip -CompressionLevel Optimal
$size = '{0:N0} MB' -f ((Get-ChildItem $dist -Recurse -File | Measure-Object Length -Sum).Sum / 1MB)
Write-Host "==> $dist ($size)"
Write-Host "==> $zip ($('{0:N0} MB' -f ((Get-Item $zip).Length / 1MB)))"
