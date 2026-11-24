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
    Guided setup of a Shortwave development environment on Windows.

.DESCRIPTION
    Checks that this machine can build Shortwave (Windows version, CPU
    architecture, memory, disk space, internet access, winget, MSYS2, Git),
    looks for conflicts, installs what is missing after asking you, runs
    setup-msys2.ps1 to install the MSYS2 UCRT64 packages, and verifies the
    toolchain. Nothing on your machine changes without a confirmation, unless
    you pass -Yes. Safe to re-run.

.PARAMETER Yes
    Accept the default answer to every question (non-interactive).

.PARAMETER CheckOnly
    Only report the state of the machine; change nothing.

.PARAMETER SkipUpdate
    Passed to setup-msys2.ps1: skip the full MSYS2 system upgrade.

.PARAMETER Msys2Root
    MSYS2 install location. Defaults to C:\msys64.

.EXAMPLE
    .\build-aux\windows\setup.ps1 -CheckOnly
    Shows what is missing without installing anything.

.EXAMPLE
    .\build-aux\windows\setup.ps1 -Yes
    Installs everything that is missing without asking.
#>
[CmdletBinding()]
param(
    [switch]$Yes,
    [switch]$CheckOnly,
    [switch]$SkipUpdate,
    [string]$Msys2Root = 'C:\msys64'
)

$ErrorActionPreference = 'Stop'

# Print errors as one readable line instead of a PowerShell stack trace.
trap {
    Write-Host ("ERROR: " + $_.Exception.Message) -ForegroundColor Red
    exit 1
}

# Windows PowerShell 5.1 may default to old TLS versions.
try { [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12 } catch { }

$MinBuild = 17763   # Windows 10 1809
$MinRamGB = 8
$MinDiskGB = 15
$DefaultMsys2Root = 'C:\msys64'

# ---------------------------------------------------------------- helpers

function New-CheckResult {
    param(
        [string]$Name,
        [ValidateSet('Pass', 'Warn', 'Fail')][string]$Status,
        [string]$Message,
        [string]$Hint
    )
    [pscustomobject]@{ Name = $Name; Status = $Status; Message = $Message; Hint = $Hint }
}

function Show-CheckResults {
    param([object[]]$Results)
    foreach ($r in $Results) {
        switch ($r.Status) {
            'Pass' { $tag = '[ OK ]'; $color = 'Green' }
            'Warn' { $tag = '[WARN]'; $color = 'Yellow' }
            'Fail' { $tag = '[FAIL]'; $color = 'Red' }
        }
        Write-Host $tag -ForegroundColor $color -NoNewline
        Write-Host (' {0,-18} {1}' -f $r.Name, $r.Message)
        if ($r.Status -ne 'Pass' -and $r.Hint) {
            Write-Host ('       -> ' + $r.Hint) -ForegroundColor DarkGray
        }
    }
}

function Confirm-Continue {
    param([string]$Question, [bool]$Default = $true)
    if ($Yes) { return $Default }
    $interactive = [Environment]::UserInteractive -and -not [Console]::IsInputRedirected
    if (-not $interactive) {
        throw "'$Question' needs an answer but this session is not interactive. Re-run with -Yes to accept the defaults, or with -CheckOnly to only report."
    }
    $suffix = if ($Default) { '[Y/n]' } else { '[y/N]' }
    while ($true) {
        $answer = (Read-Host "$Question $suffix").Trim().ToLowerInvariant()
        if ($answer -eq '') { return $Default }
        if ($answer -in 'y', 'yes') { return $true }
        if ($answer -in 'n', 'no') { return $false }
        Write-Host 'Please answer y or n.'
    }
}

function Get-RepoRoot { (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path }

function Get-InnoSetupPath {
    foreach ($base in @(${env:ProgramFiles(x86)}, $env:ProgramFiles, "$env:LOCALAPPDATA\Programs")) {
        if (-not $base) { continue }
        $iscc = Join-Path $base 'Inno Setup 6\ISCC.exe'
        if (Test-Path $iscc) { return $iscc }
    }
    return $null
}

# ----------------------------------------------------------------- checks

function Test-WindowsVersion {
    $os = Get-CimInstance Win32_OperatingSystem
    $build = [int]$os.BuildNumber
    $label = '{0} (build {1})' -f $os.Caption.Trim(), $build
    if ($build -ge $MinBuild) { return New-CheckResult 'Windows' 'Pass' $label }
    New-CheckResult 'Windows' 'Fail' "$label is too old" "Windows 10 1809 (build $MinBuild) or newer is required."
}

function Test-Architecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    try { $arch = [string][Runtime.InteropServices.RuntimeInformation]::OSArchitecture } catch { }
    switch -Regex ($arch) {
        '^(X64|AMD64)$' { return New-CheckResult 'CPU architecture' 'Pass' 'x64' }
        '^ARM64$' { return New-CheckResult 'CPU architecture' 'Warn' 'ARM64' 'MSYS2 UCRT64 targets x64; it works through Windows x64 emulation but is slower and less tested.' }
        default { return New-CheckResult 'CPU architecture' 'Fail' $arch 'A 64-bit x64 Windows is required.' }
    }
}

function Test-Memory {
    $bytes = (Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory
    $gb = [math]::Round($bytes / 1GB, 1)
    # An "8 GB" machine reports slightly less (reserved memory), so allow 0.5 GB of slack.
    if ($gb -ge ($MinRamGB - 0.5)) { return New-CheckResult 'Memory' 'Pass' "$gb GB RAM" }
    New-CheckResult 'Memory' 'Warn' "$gb GB RAM" "$MinRamGB GB or more is recommended; linking the Rust build can be slow or fail with less."
}

function Test-DiskSpace {
    param([string]$Path, [string]$Label, [int]$MinGB)
    $name = "Disk ($Label)"
    $root = [IO.Path]::GetPathRoot($Path)
    $drive = Get-PSDrive -PSProvider FileSystem | Where-Object { $_.Root -eq $root } | Select-Object -First 1
    if (-not $drive) { return New-CheckResult $name 'Fail' "drive $root not found" 'Choose a path on an existing drive.' }
    $gb = [math]::Round($drive.Free / 1GB, 1)
    if ($gb -ge $MinGB) { return New-CheckResult $name 'Pass' "$gb GB free on $root" }
    New-CheckResult $name 'Warn' "$gb GB free on $root" "$MinGB GB or more is recommended (MSYS2 packages and a Rust build are large)."
}

function Test-RepoPath {
    $repo = Get-RepoRoot
    if ($repo -match '[^\x21-\x7E]') {
        return New-CheckResult 'Repository path' 'Warn' $repo 'Spaces or non-ASCII characters in the path can break Meson and MSYS2 builds; consider cloning to a path like C:\src\shortwave.'
    }
    New-CheckResult 'Repository path' 'Pass' $repo
}

function Test-Internet {
    try {
        [void][Net.Dns]::GetHostAddresses('github.com')
    } catch {
        return New-CheckResult 'Internet' 'Fail' 'cannot resolve github.com' 'Check your network, proxy or VPN; packages are downloaded during setup.'
    }
    try {
        [void](Invoke-WebRequest -Uri 'https://repo.msys2.org' -Method Head -TimeoutSec 5 -UseBasicParsing)
    } catch {
        # Any HTTP answer (even an error status) means the server is reachable.
        if (-not $_.Exception.Response) {
            return New-CheckResult 'Internet' 'Fail' 'cannot reach repo.msys2.org' 'The MSYS2 package server must be reachable (firewall, proxy or VPN?).'
        }
    }
    New-CheckResult 'Internet' 'Pass' 'github.com and repo.msys2.org reachable'
}

function Test-Winget {
    if (Get-Command winget -ErrorAction SilentlyContinue) { return New-CheckResult 'winget' 'Pass' 'available' }
    New-CheckResult 'winget' 'Warn' 'not found' 'Only needed to install missing tools. Get it by installing "App Installer" from the Microsoft Store.'
}

function Test-AdminRights {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    $admin = ([Security.Principal.WindowsPrincipal]$id).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if ($admin) { return New-CheckResult 'Administrator' 'Pass' 'running as administrator' }
    New-CheckResult 'Administrator' 'Pass' 'standard user (not required)' 'Installers may show a UAC prompt.'
}

function Test-Msys2 {
    param([string]$Root)
    $bash = Join-Path $Root 'usr\bin\bash.exe'
    if (Test-Path $bash) { return New-CheckResult 'MSYS2' 'Pass' "installed at $Root" }
    if (Test-Path $Root) {
        return New-CheckResult 'MSYS2' 'Fail' "$Root exists but has no usr\bin\bash.exe" 'Conflict: a broken or foreign directory is in the way. Move or delete it, or use -Msys2Root with another path.'
    }
    New-CheckResult 'MSYS2' 'Fail' "not found at $Root" 'Will be installed with winget (MSYS2.MSYS2).'
}

function Test-Git {
    $git = Get-Command git -ErrorAction SilentlyContinue
    if ($git) { return New-CheckResult 'Git' 'Pass' (& $git.Source --version) }
    New-CheckResult 'Git' 'Fail' 'not found' 'Will be installed with winget (Git.Git). Meson needs it for the version tag.'
}

function Test-InnoSetup {
    $iscc = Get-InnoSetupPath
    if ($iscc) { return New-CheckResult 'Inno Setup' 'Pass' 'installed' }
    New-CheckResult 'Inno Setup' 'Pass' 'not installed (optional)' 'Only needed to build the installer with installer.ps1.'
}

function Test-Conflicts {
    param([string]$Root)
    $results = @()
    $running = @(Get-Process -ErrorAction SilentlyContinue | Where-Object {
            $_.Name -eq 'pacman' -or ($_.Path -and $_.Path.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase))
        })
    if ($running.Count -gt 0) {
        $names = ($running | Select-Object -ExpandProperty Name -Unique) -join ', '
        $results += New-CheckResult 'Running processes' 'Warn' "MSYS2 processes running: $names" 'Close MSYS2 shells and wait for any running pacman before updating packages.'
    } else {
        $results += New-CheckResult 'Running processes' 'Pass' 'no MSYS2 processes running'
    }
    $others = @()
    foreach ($c in @("$env:SystemDrive\msys64", "$env:SystemDrive\msys2", 'D:\msys64', "$env:USERPROFILE\msys64")) {
        $full = [IO.Path]::GetFullPath($c)
        if ($full.TrimEnd('\') -ne ([IO.Path]::GetFullPath($Root)).TrimEnd('\') -and (Test-Path (Join-Path $full 'usr\bin\bash.exe'))) { $others += $full }
    }
    if ($others.Count -gt 0) {
        $results += New-CheckResult 'Other MSYS2' 'Warn' ('also found at ' + ($others -join ', ')) 'Use -Msys2Root <path> to pick one; build.ps1 takes the same -Msys2Root switch.'
    } else {
        $results += New-CheckResult 'Other MSYS2' 'Pass' 'no other installation found'
    }
    $results
}

# --------------------------------------------------------------- actions

function Install-WithWinget {
    param([string]$Id, [string]$Name, [string]$Location)
    Write-Host "==> Installing $Name with winget ($Id)" -ForegroundColor Cyan
    $wingetArgs = @('install', '--id', $Id, '-e', '--accept-package-agreements', '--accept-source-agreements', '--silent')
    if ($Location) { $wingetArgs += @('--location', $Location) }
    & winget @wingetArgs
    if ($LASTEXITCODE -ne 0) { throw "winget could not install $Name (exit code $LASTEXITCODE). Install it manually and re-run this script." }
}

function Update-SessionPath {
    $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $user = [Environment]::GetEnvironmentVariable('Path', 'User')
    $env:PATH = "$machine;$user"
}

function Invoke-PackageSetup {
    $setupArgs = @{ Msys2Root = $Msys2Root }
    if ($SkipUpdate) { $setupArgs['SkipUpdate'] = $true }
    & (Join-Path $PSScriptRoot 'setup-msys2.ps1') @setupArgs
}

function Test-Toolchain {
    $bash = Join-Path $Msys2Root 'usr\bin\bash.exe'
    $env:MSYSTEM = 'UCRT64'
    $env:CHERE_INVOKING = '1'
    $env:LANG = 'C'
    $script = 'set -e; rustc --version; cargo --version; meson --version; ninja --version; blueprint-compiler --version; ' +
        'for m in gtk4 libadwaita-1 gstreamer-1.0 sqlite3 shumate-1.0; do v=$(pkg-config --modversion $m); echo $m $v; done'
    # Out-Host keeps bash's output out of the function's return value (a single boolean).
    & $bash -lc $script | Out-Host
    $LASTEXITCODE -eq 0
}

# ------------------------------------------------------------------- main

function Invoke-AllChecks {
    $repo = Get-RepoRoot
    @(
        Test-WindowsVersion
        Test-Architecture
        Test-Memory
        Test-DiskSpace -Path $Msys2Root -Label 'MSYS2' -MinGB $MinDiskGB
        Test-DiskSpace -Path $repo -Label 'repository' -MinGB $MinDiskGB
        Test-RepoPath
        Test-Internet
        Test-Winget
        Test-AdminRights
        Test-Msys2 -Root $Msys2Root
        Test-Git
        Test-InnoSetup
        Test-Conflicts -Root $Msys2Root
    )
}

Write-Host ''
Write-Host 'Shortwave for Windows - guided setup' -ForegroundColor Cyan
Write-Host '  1. check this machine   2. install what is missing   3. install the MSYS2 packages   4. verify'
Write-Host '  Nothing is changed without asking you first (-CheckOnly changes nothing at all).'
Write-Host ''

$results = @(Invoke-AllChecks)
Show-CheckResults $results

$fails = @($results | Where-Object { $_.Status -eq 'Fail' }).Count
$warns = @($results | Where-Object { $_.Status -eq 'Warn' }).Count
Write-Host ''
Write-Host ("Summary: {0} checks, {1} warning(s), {2} to fix." -f $results.Count, $warns, $fails)

if ($CheckOnly) { exit 0 }

function Stop-Setup([string]$Message) {
    Write-Host $Message -ForegroundColor Red
    exit 1
}

# 1. Conflicts are never fixed automatically.
$bashPath = Join-Path $Msys2Root 'usr\bin\bash.exe'
if ((Test-Path $Msys2Root) -and -not (Test-Path $bashPath)) {
    Stop-Setup "Conflict: $Msys2Root exists but is not a working MSYS2 install. Move or delete it, or pass -Msys2Root with another path. Nothing was changed."
}

# 2. Problems this script cannot fix, then warnings.
$fixable = @('MSYS2', 'Git')
$blocking = @($results | Where-Object { $_.Status -eq 'Fail' -and $fixable -notcontains $_.Name })
if ($blocking.Count -gt 0) {
    Write-Host ''
    Write-Host 'These problems cannot be fixed by this script:' -ForegroundColor Red
    $blocking | ForEach-Object { Write-Host ("  - {0}: {1}" -f $_.Name, $_.Message) }
    if (-not (Confirm-Continue 'Continue anyway? The build is likely to fail.' $false)) { Stop-Setup 'Stopped. Nothing was changed.' }
} elseif ($warns -gt 0) {
    if (-not (Confirm-Continue 'Some checks have warnings. Continue?' $true)) { Stop-Setup 'Stopped. Nothing was changed.' }
}

# 3. Install missing tools.
$needMsys2 = -not (Test-Path $bashPath)
$needGit = -not (Get-Command git -ErrorAction SilentlyContinue)
if (($needMsys2 -or $needGit) -and -not (Get-Command winget -ErrorAction SilentlyContinue)) {
    Stop-Setup 'winget is needed to install the missing tools. Install "App Installer" from the Microsoft Store (or install MSYS2 and Git yourself) and re-run this script.'
}

if ($needMsys2) {
    if (Confirm-Continue "Install MSYS2 (about 400 MB) to $Msys2Root with winget?" $true) {
        $location = $null
        if ($Msys2Root.TrimEnd('\') -ne $DefaultMsys2Root) { $location = $Msys2Root }
        Install-WithWinget -Id 'MSYS2.MSYS2' -Name 'MSYS2' -Location $location
        if (-not (Test-Path $bashPath)) { Stop-Setup "MSYS2 was installed but $bashPath was not found. Pass -Msys2Root with its real location." }
    } else {
        Stop-Setup 'MSYS2 is required. Stopped. Nothing else was changed.'
    }
}

if ($needGit) {
    if (Confirm-Continue 'Install Git for Windows with winget?' $true) {
        Install-WithWinget -Id 'Git.Git' -Name 'Git for Windows'
        Update-SessionPath
    } else {
        Write-Warning 'Skipping Git; Meson will not be able to embed the version tag.'
    }
}

if (-not (Get-InnoSetupPath)) {
    if (Confirm-Continue 'Install Inno Setup 6 (only needed to build the installer)?' $false) {
        Install-WithWinget -Id 'JRSoftware.InnoSetup' -Name 'Inno Setup 6'
    }
}

# 4. MSYS2 packages (the package list lives in setup-msys2.ps1).
Write-Host ''
Write-Host 'Next: update MSYS2 and install about 23 UCRT64 packages (GTK4, libadwaita, GStreamer, Rust, Meson, ...).'
Write-Host 'This downloads roughly 2 GB and can take 10 to 30 minutes.'
if (-not (Confirm-Continue 'Run the package setup now?' $true)) { Stop-Setup 'Stopped before installing packages. Re-run this script when ready.' }
Invoke-PackageSetup

# 5. Verify.
Write-Host ''
Write-Host '==> Verifying the toolchain' -ForegroundColor Cyan
if (-not (Test-Toolchain)) { Stop-Setup 'Verification failed: a tool is missing or broken. Re-run this script, or run setup-msys2.ps1 and read its output.' }

$rootFlag = ''
if ($Msys2Root.TrimEnd('\') -ne $DefaultMsys2Root) { $rootFlag = " -Msys2Root '$Msys2Root'" }
Write-Host ''
Write-Host 'Setup complete. Next steps:' -ForegroundColor Green
Write-Host "  .\build-aux\windows\build.ps1 -BuildProfile development$rootFlag        # build"
Write-Host "  .\build-aux\windows\build.ps1 -BuildProfile development$rootFlag -Run   # build and start"
if (Confirm-Continue 'Run the development build now?' $false) {
    & (Join-Path $PSScriptRoot 'build.ps1') -BuildProfile development -Msys2Root $Msys2Root
}
