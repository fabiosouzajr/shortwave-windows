# Guided setup script for new contributors

Status: approved by the user in conversation (2026-11-24). Size: small change.

## Intent and why

New contributors who clone the repo should be able to run one script and end up
with a working build environment, without reading `docs/BUILDING-WINDOWS.md`
first. The maintainer did these steps by hand on a fresh machine (install
MSYS2, run `setup-msys2.ps1`, verify versions); the script automates and
explains them, and asks before anything that changes the machine.

## Goals

- `build-aux/windows/setup.ps1`: an interactive, guided setup that
  1. checks the OS (Windows 10 1809+/11, x64), hardware (RAM, free disk on the
     MSYS2 drive and the repo drive) and software (winget, MSYS2, Git, optional
     Inno Setup), plus internet access and admin rights;
  2. detects conflicts (MSYS2 in a non-default path, a running pacman, MSYS2
     directory with files but no `usr\bin\bash.exe`, an older MSYS2) and asks
     the user how to proceed when a check fails or warns;
  3. installs missing tools through winget, each after confirmation;
  4. runs the existing `setup-msys2.ps1` for the package list;
  5. verifies tool versions and offers a first development build.
- Reference the script from `README.md` (Building section) and
  `docs/BUILDING-WINDOWS.md`.

## Non-goals and anti-goals

- No second copy of the MSYS2 package list: `setup-msys2.ps1` stays the single
  source (CI uses it) and is not changed.
- Not a replacement for `build.ps1`; it only offers to call it.
- Must never change the machine without a prompt (unless `-Yes` is passed), and
  must not install Visual Studio or rustup (the docs say they are not needed).

## Constraints

- Works in Windows PowerShell 5.1 and PowerShell 7 (the docs promise both).
- Same GPL header and comment-based help style as the other `build-aux/windows`
  scripts.
- `build.ps1` takes `-Msys2Root`; a non-default MSYS2 path is passed through to
  `setup-msys2.ps1` and the script prints the matching `build.ps1` flag.

## Approaches considered

1. **Wrapper around `setup-msys2.ps1`** (chosen, my call): checks and winget
   installs first, then delegates package installation. No duplicated package
   list; CI and the script stay in step.
2. Extend `setup-msys2.ps1` itself: fewer files, but it is used by CI
   non-interactively, so the risk of breaking CI is higher and the script gets
   two jobs.
3. Standalone script with its own package list: self-contained but drifts from
   CI.

## Decisions I made (not asked)

- Minimums: 8 GB RAM recommended (warn below), 15 GB free disk (the MSYS2
  packages plus a Rust build; warn below, ask to continue).
- Switches: `-Yes` (accept defaults, non-interactive), `-SkipUpdate` (passed
  through), `-Msys2Root`, `-CheckOnly` (report only, change nothing).
- Inno Setup is optional and defaults to "no".

## Left to the builder

Exact wording of prompts, the check output layout, and the version-detection
details.
