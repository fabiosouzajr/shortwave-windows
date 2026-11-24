# Guided Setup Script Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** One interactive script, `build-aux/windows/setup.ps1`, that checks a new contributor's machine, installs missing tools after confirmation, runs the existing package setup and verifies the result.

**Architecture:** A wrapper in front of `setup-msys2.ps1` (unchanged, still the only package list, still used by CI). Checks are small functions that each return one result object (`Status` of `Pass`/`Warn`/`Fail`, `Message`, `Hint`); a report loop prints them and prompts on `Warn`/`Fail`. Installs go through winget.

**Tech Stack:** PowerShell 5.1 and 7 compatible, winget, MSYS2 UCRT64, existing `setup-msys2.ps1` / `build.ps1`.

**Spec:** `docs/superpowers/specs/2026-11-24-guided-setup-script-design.md`

## Global Constraints

- Must run in Windows PowerShell 5.1 and PowerShell 7 (no `??`, `?.`, ternary, `&&`).
- Same GPL-3.0-or-later header and comment-based help style as `build-aux/windows/build.ps1`.
- `setup-msys2.ps1` is not modified; the package list is not copied.
- Minimums: RAM warn below 8 GB; free disk warn below 15 GB (MSYS2 drive and repo drive); OS Windows 10 build 17763+ (1809) and x64.
- Parameters: `-Yes`, `-CheckOnly`, `-SkipUpdate`, `-Msys2Root` (default `C:\msys64`).
- Nothing changes the machine without a prompt unless `-Yes`; `-CheckOnly` changes nothing at all.
- Do not install Visual Studio or rustup. Inno Setup is optional, default "no".
- Default MSYS2 path stays `C:\msys64`; a different one is passed to `setup-msys2.ps1` and the matching `build.ps1 -Msys2Root` is printed.

## Review Focus

- Non-interactive host (CI, redirected stdin) without `-Yes`: must not hang on `Read-Host`; stop with a clear message telling the user to pass `-Yes` or `-CheckOnly`.
- `C:\msys64` exists but has no `usr\bin\bash.exe` (broken or partial install): report a conflict, never install over it silently.
- Repo path containing spaces or non-ASCII characters (e.g. `C:\Users\José Silva\...`): warn, since Meson/MSYS2 builds are fragile there.
- winget missing or its sources unreachable: say how to get it (App Installer) instead of a stack trace.
- Re-run on an already-set-up machine: everything reports `Pass`, nothing is reinstalled, exit code 0.

---

### Task 1: Checks and report (`-CheckOnly`)

**Files:**
- Create: `build-aux/windows/setup.ps1`

**Interfaces:**
- Produces: `New-CheckResult([string]$Name, [ValidateSet('Pass','Warn','Fail')][string]$Status, [string]$Message, [string]$Hint) -> PSCustomObject`; check functions `Test-WindowsVersion`, `Test-Architecture`, `Test-Memory`, `Test-DiskSpace([string]$Path, [string]$Label, [int]$MinGB)`, `Test-RepoPath`, `Test-Internet`, `Test-Winget`, `Test-AdminRights`, `Test-Msys2([string]$Root)`, `Test-Git`, `Test-InnoSetup`, `Test-Conflicts([string]$Root)` (running pacman/msys2 processes, MSYS2 dir without `bash.exe`, other MSYS2 at a non-default path); `Show-CheckResults([object[]]$Results)`; `Confirm-Continue([string]$Question, [bool]$Default) -> bool` (returns `$Default` under `-Yes`, throws a clear error when the host is non-interactive and `-Yes` is absent).

- [ ] **Step 1: Create the script skeleton** with the license header, comment-based help (`.SYNOPSIS`, `.DESCRIPTION`, one `.PARAMETER` per parameter in Global Constraints, `.EXAMPLE` for `-CheckOnly` and `-Yes`), `[CmdletBinding()] param(...)`, `$ErrorActionPreference = 'Stop'`, and a banner that explains in three lines what the script will do and that nothing changes without confirmation.
- [ ] **Step 2: Implement the helpers and check functions** listed in Interfaces. Data sources: `Get-CimInstance Win32_OperatingSystem` (build number, `OSArchitecture`), `Win32_ComputerSystem.TotalPhysicalMemory`, `Get-PSDrive` free space for the drive of the given path, `Resolve-Path (Join-Path $PSScriptRoot '..\..')` for the repo, `[Net.Dns]::GetHostAddresses('github.com')` plus a HEAD request to `https://repo.msys2.org` with a 5 s timeout, `Get-Command winget`, `WindowsPrincipal.IsInRole(Administrator)` (Warn only: winget and MSYS2 install per user work without it), `git --version` for Git, `Inno Setup 6\ISCC.exe` under `Program Files (x86)` for Inno Setup (Warn, not Fail, when absent). `Test-Msys2` is `Pass` when `usr\bin\bash.exe` exists, `Fail` with hint `will be installed` when the directory is absent, and `Fail` with a conflict hint when the directory exists without `bash.exe`.
- [ ] **Step 3: Wire the main flow for checks only:** run every check, print with `Show-CheckResults` (colored `[ OK ]`, `[WARN]`, `[FAIL]` lines, hint under each non-pass), print a summary count, and `exit 0` when `-CheckOnly` is set regardless of failures (it is a report).
- [ ] **Step 4: Verify parsing in both shells**

Run: `powershell -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('build-aux/windows/setup.ps1',[ref]$null,[ref]$e); $e; if(-not $e){'PARSE OK'}"` and the same with `pwsh`
Expected: `PARSE OK` in both.

- [ ] **Step 5: Verify behavior with `-CheckOnly`**

Run: `pwsh -NoProfile -File build-aux/windows/setup.ps1 -CheckOnly` (on this already set-up machine), then `... -CheckOnly -Msys2Root D:\does-not-exist`
Expected: first run shows all `[ OK ]` for MSYS2, Git, winget and OS checks; second shows `[FAIL]` for MSYS2 with the "will be installed" hint; exit code 0 and no files created in both.

- [ ] **Step 6: Verify the non-interactive guard (Review Focus)**

Run: `pwsh -NoProfile -File build-aux/windows/setup.ps1 -Msys2Root D:\does-not-exist < NUL`
Expected: stops with the message telling the user to pass `-Yes` or `-CheckOnly`; no prompt hang; MSYS2 not installed.

- [ ] **Step 7: Commit**

```bash
git add build-aux/windows/setup.ps1
git commit -m "feat(setup): add guided prerequisite checks"
```

### Task 2: Install, delegate, verify, offer build

**Files:**
- Modify: `build-aux/windows/setup.ps1`

**Interfaces:**
- Consumes: Task 1's `Confirm-Continue`, `Test-*` functions, `New-CheckResult`.
- Produces: `Install-WithWinget([string]$Id, [string]$Name)` (runs `winget install --id $Id -e --accept-package-agreements --accept-source-agreements --silent`, throws on non-zero exit); `Invoke-PackageSetup()` (calls `& "$PSScriptRoot\setup-msys2.ps1" -Msys2Root $Msys2Root` plus `-SkipUpdate` when set); `Test-Toolchain()` (runs, via MSYS2 bash with `MSYSTEM=UCRT64`, `rustc --version`, `meson --version`, `ninja --version`, `blueprint-compiler --version`, `pkg-config --modversion gtk4 libadwaita-1 gstreamer-1.0 sqlite3 shumate-1.0` and prints them; returns `$true` when all exit 0).

- [ ] **Step 1: Add the decision flow after the checks.** On any `Fail`/`Warn`, explain each in plain words and call `Confirm-Continue 'Continue anyway?'` for warnings (default yes) and for unfixable failures (default no, exit 1 when declined). Fixable failures (MSYS2, Git absent; Inno Setup if the user opts in, default no) are handled in the next steps; a conflict (MSYS2 dir without `bash.exe`) is never auto-fixed: print what was found and exit 1 telling the user to move or remove it.
- [ ] **Step 2: Implement `Install-WithWinget`** and use it for `MSYS2.MSYS2`, `Git.Git` and `JRSoftware.InnoSetup`, each behind its own `Confirm-Continue` that names what will be installed and roughly how large. If winget is missing, print how to get it (install "App Installer" from the Microsoft Store) and exit 1. After installing Git, refresh `$env:PATH` from the machine and user registry values so the same session sees it.
- [ ] **Step 3: Run the package setup:** explain that it updates MSYS2 and installs about 23 packages and can take 10 to 30 minutes, confirm, then `Invoke-PackageSetup`.
- [ ] **Step 4: Verify and finish:** `Test-Toolchain`, then print a "Next steps" block with the exact commands (`.\build-aux\windows\build.ps1 -BuildProfile development`, with `-Msys2Root` when non-default, and the `-Run` variant) and offer to start the development build now (`Confirm-Continue`, default no).
- [ ] **Step 5: Verify idempotency (Review Focus)**

Run: `pwsh -NoProfile -File build-aux/windows/setup.ps1 -Yes -SkipUpdate` on this machine
Expected: all checks `[ OK ]`, `pacman --needed` reports nothing to install or only reinstalls nothing, toolchain versions printed, exit code 0. Answer "no" to the build offer under `-Yes` (default is no).

- [ ] **Step 6: Verify the conflict path (Review Focus)**

Run: create a temp dir containing a file, then `pwsh -NoProfile -File build-aux/windows/setup.ps1 -Yes -Msys2Root <that dir>`
Expected: a conflict message, exit code 1, nothing installed into that directory.

- [ ] **Step 7: Verify the spaced/non-ASCII repo path warning (Review Focus)**

Run: copy the script to a temp directory `...\sp ace é\build-aux\windows\` and `-CheckOnly` from there (the repo path check runs against `$PSScriptRoot\..\..`).
Expected: a `[WARN]` on the repo path check.

- [ ] **Step 8: Commit**

```bash
git add build-aux/windows/setup.ps1
git commit -m "feat(setup): install missing tools, run package setup, verify"
```

### Task 3: Documentation, then push

**Files:**
- Modify: `README.md` (Building section, near line 59)
- Modify: `docs/BUILDING-WINDOWS.md` (Prerequisites / step 1)
- Modify: `CHANGELOG.md` (one line under `## [Unreleased]`, matching the existing entry style)
- Add: the design doc and this plan (already in `docs/superpowers/`)

- [ ] **Step 1: README:** in the Building section, show `.\build-aux\windows\setup.ps1` as the recommended first step (guided: checks the machine, installs MSYS2 and the packages), and keep `setup-msys2.ps1` as the non-interactive alternative used by CI. Mention `-CheckOnly` and `-Yes` in one line.
- [ ] **Step 2: `docs/BUILDING-WINDOWS.md`:** add a short "Guided setup" paragraph at the start of step 1 pointing to `setup.ps1` and the same switches; leave the manual steps intact.
- [ ] **Step 3: CHANGELOG:** add the entry under Unreleased. Do not change versions or release notes (no release is being cut).
- [ ] **Step 4: Verify links and wording**

Run: `grep -n "setup.ps1" README.md docs/BUILDING-WINDOWS.md CHANGELOG.md`
Expected: each file references `build-aux/windows/setup.ps1` (README and docs also keep their `setup-msys2.ps1` mentions).

- [ ] **Step 5: Run the repo checks that exist for scripts**

Run: `bash build-aux/checks.sh` if it covers docs or scripts (read it first); otherwise skip and say so.
Expected: passes, or the skip is reported.

- [ ] **Step 6: Commit and push** (no tag; pushing a tag publishes a release and is not part of this task)

```bash
git add README.md docs/BUILDING-WINDOWS.md CHANGELOG.md docs/superpowers
git commit -m "docs: reference the guided setup script"
git push origin main
```
Expected: push succeeds; `git status` clean; `gh run list --limit 1` shows the CI run for the push (report its result, do not wait on release jobs).
