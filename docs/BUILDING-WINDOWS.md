# Building Shortwave on Windows

This is the unofficial Windows port of GNOME Shortwave. It builds natively on
Windows 10/11 x64 with MSYS2's **UCRT64** environment (GNU toolchain, GTK4,
libadwaita, GStreamer).

## Prerequisites

- [MSYS2](https://www.msys2.org/) installed at `C:\msys64`
  (`winget install MSYS2.MSYS2`)
- Git for Windows (`winget install Git.Git`)
- PowerShell 7 or Windows PowerShell 5.1

`build-aux/windows/setup.ps1` (step 1) checks all of this and offers to install
MSYS2 and Git for you, so you can start there.

Visual Studio and rustup are **not** needed to build. (They are only useful for
editor tooling such as rust-analyzer.)

## 1. Install the build dependencies (once)

```powershell
git clone <this repository> shortwave-windows
cd shortwave-windows
.\build-aux\windows\setup.ps1
```

`setup.ps1` is the guided way: it checks Windows, CPU, memory, disk space,
internet access and conflicts (for example a broken `C:\msys64`), asks before
installing MSYS2, Git or Inno Setup with winget, runs `setup-msys2.ps1`, and
verifies the toolchain. Use `-CheckOnly` to only report, `-Yes` to accept every
default, `-SkipUpdate` to skip the full `pacman -Syu`, and `-Msys2Root <path>`
for a non-default MSYS2 location (pass the same flag to `build.ps1`). It only
installs MSYS2 itself to `C:\msys64`, because the winget package has no
install-location option; for another folder, install MSYS2 yourself first.

If MSYS2 is already installed you can run the package step alone:
`.\build-aux\windows\setup-msys2.ps1`. It updates MSYS2 and installs the UCRT64
packages: toolchain, Rust, Meson, Ninja, GTK4, libadwaita, libshumate,
GStreamer (base/good/bad/libav), SQLite, gettext, librsvg, the WebP pixbuf
loader, glib-networking and ntldd. It is safe to re-run; add `-SkipUpdate` to
skip the full `pacman -Syu`.

### If PowerShell refuses to run the scripts

A fresh Windows PowerShell 5.1 blocks local scripts ("running scripts is
disabled on this system"), and a repository downloaded as a zip is also blocked
by Mark-of-the-Web. Either run the script for this session only:

```powershell
powershell -ExecutionPolicy Bypass -File .\build-aux\windows\setup.ps1
```

or allow local scripts for your user once, and unblock a zip download:

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
Get-ChildItem -Recurse . | Unblock-File
```


## 2. Build

```powershell
.\build-aux\windows\build.ps1            # release build into _build\install
.\build-aux\windows\build.ps1 -Run       # build, then start the app
.\build-aux\windows\build.ps1 -BuildProfile development   # debug build, app id *.Devel
.\build-aux\windows\build.ps1 -Reconfigure                # wipe and re-run meson setup
```

The script runs `meson setup`, `meson compile` and `meson install` inside
MSYS2 UCRT64. The executable ends up in `_build\install\bin\shortwave.exe`.

Until the self-contained bundle exists, `shortwave.exe` needs MSYS2's
`C:\msys64\ucrt64\bin` on `PATH` for the GTK and GStreamer DLLs; `-Run` sets
that up for you.

## Building by hand

From an **MSYS2 UCRT64** shell:

```sh
meson setup _build --prefix="$PWD/_build/install"
meson compile -C _build
meson install -C _build
```

To run clippy or the tests by hand, use MSYS2's cargo **and** a `CARGO_HOME`
other than `%USERPROFILE%\.cargo`. Cargo looks up subcommands such as
`cargo-clippy` in `$CARGO_HOME/bin` first, so with rustup installed it would
run the MSVC toolchain's clippy instead:

```sh
export CARGO_HOME=$PWD/_build/cargo-home
cargo clippy --locked && cargo test --locked
```

## Packaging

```powershell
.\build-aux\windows\bundle.ps1      # dist\Shortwave + portable zip
.\build-aux\windows\installer.ps1   # Inno Setup installer (needs Inno Setup 6)
```

If Meson can't find `git`, either install it in MSYS2
(`pacman -S git`) or start the shell with `MSYS2_PATH_TYPE=inherit`.

## Releasing

Releases are published by CI when a `v<version>` tag is pushed:

1. Set the new version in `meson.build` (e.g. `5.0.0-win.2`: upstream
   version, then `-win.N` for the port's own releases).
2. Write the release notes in `docs/releases/<version>.md` (they become the
   text of the GitHub Release) and move the `Unreleased` entries in
   `CHANGELOG.md` under the new version and date.
3. Commit and push to `main`.
4. Tag and push the tag:

   ```powershell
   git tag -a v5.0.0-win.2 -m "Shortwave for Windows 5.0.0-win.2"
   git push origin v5.0.0-win.2
   ```

The CI run for the tag builds and tests on Windows and Linux; the `release`
job then publishes "Shortwave for Windows <version>" with the installer and
the portable zip attached. It fails without publishing anything if the built
files don't carry the tag's version or the release notes file is missing; fix
it, then delete and re-push the tag. To check the script locally:
`bash build-aux/test-publish-release.sh`.

## How the Windows build differs

- `meson.build` skips the D-Bus, OpenSSL, lcms2 and libseccomp checks on
  Windows (HTTPS uses SChannel; there is no D-Bus session bus or glycin).
- The cargo step goes through `build-aux/windows/cargo-build.py`, because Ninja
  on Windows runs commands without a shell and the binary is `shortwave.exe`.
- The `.desktop` file and D-Bus service aren't installed; the AppStream
  metainfo is still generated because it's bundled into the GResource.
- Platform-specific code lives in `src/platform/` (`linux.rs` keeps upstream's
  behaviour for every non-Windows target, `windows.rs` holds the port).

See [PORTING.md](PORTING.md) for the design and [re/](re/) for the analysis.
