# Shortwave

![screenshot](data/screenshots/1.png)
![screenshot](data/screenshots/4.png)

Shortwave is an internet radio player that provides access to a station database with over 50,000 stations.

This is an **unofficial Windows 10/11 (x64) port** of [GNOME Shortwave](https://gitlab.gnome.org/World/Shortwave). It is not affiliated with the Shortwave authors or the GNOME project; see [NOTICE.md](NOTICE.md). Please report problems with the Windows build here, not upstream.

## Installation
Download the latest version from the [Releases page](https://github.com/fabiosouzajr/shortwave-windows/releases) (Windows 10/11, 64-bit):

- `Shortwave-<version>-windows-x64-setup.exe`: installer (per-user or per-machine), recommended.
- `Shortwave-<version>-windows-x64.zip`: portable, self-contained bundle. Unzip and run `Shortwave\bin\shortwave.exe`.

The files aren't code-signed, so Windows SmartScreen may ask you to confirm (**More info → Run anyway**). You can also build both yourself (see [Building](#building)). What changed in each version: [CHANGELOG.md](CHANGELOG.md).

## Features on Windows
- Search by station name or genre, filtered by country, codec and minimum bitrate.
- Station cards with country flags, station logos (also found on the station's homepage) and a codec/bitrate meter.
- Playback (HTTP/HTTPS/HLS), a heart in the player to save the playing station, library, recording (saved to `Music\Shortwave`).
- Media keys and the Windows media overlay, a tray icon for background playback, toast notifications, dark/light mode following Windows.
- App language selectable in Preferences.

Chromecast discovery works, but casting itself is untested. See [docs/WINDOWS-PORT-STATUS.md](docs/WINDOWS-PORT-STATUS.md) for details.

## FAQ
- **Why is it called 'Shortwave'?**

    Shortwave signals have a very long range because of their very good reflection properties. 
Due to their long range, they can be received almost anywhere in the world. 
The same applies to Internet radio stations, which can also be received almost anywhere in the world.
That's why we decided to call the project 'Shortwave', because internet radio stations and shortwave radio stations share many characteristics.

    If you want to know more about the naming process, you should read this [blog post](https://blogs.gnome.org/tbernard/2019/04/26/naming-your-app/)

- **Why I cannot edit stations anymore?**

    The edit feature is disabled because of vandalism. I cannot change this. [More information here](http://www.radio-browser.info/gui/#/) and [here](https://github.com/segler-alex/radiobrowser-api/issues/39).

- **Which database does Shortwave use?**

    [radio-browser.info](http://www.radio-browser.info/gui/#/). It's a community database, everybody can contribute information.

- **How I can get debug information?**

    Release builds have no console window. Build with `.\build-aux\windows\build.ps1 -BuildProfile development` (debug builds keep the console) and run it with `RUST_BACKTRACE=1` and `RUST_LOG=shortwave=debug` set.

## Translations
Translation of this project takes place on the GNOME translation platform,
[Damned Lies](https://l10n.gnome.org/module/shortwave). For further
information on how to join a language team, or even to create one, please see
[GNOME Translation Project wiki page](https://wiki.gnome.org/TranslationProject).

## Building
Shortwave builds natively on Windows with MSYS2 UCRT64 (GTK4, libadwaita, GStreamer).

```powershell
.\build-aux\windows\setup.ps1         # guided setup: checks your PC, installs MSYS2 and the build dependencies (once)
.\build-aux\windows\build.ps1 -Run    # build and start the app
.\build-aux\windows\bundle.ps1        # dist\Shortwave + portable zip
.\build-aux\windows\installer.ps1     # Inno Setup installer (needs Inno Setup 6)
```

New to the project? Run [build-aux/windows/setup.ps1](build-aux/windows/setup.ps1) first. It checks Windows, memory, disk space and conflicts, asks before installing anything, then runs `setup-msys2.ps1` (the non-interactive package installer CI uses). `-CheckOnly` just reports; `-Yes` accepts every default. If Windows says "running scripts is disabled", run it as `powershell -ExecutionPolicy Bypass -File .\build-aux\windows\setup.ps1` (see [the build docs](docs/BUILDING-WINDOWS.md#if-powershell-refuses-to-run-the-scripts)).

Full instructions: [docs/BUILDING-WINDOWS.md](docs/BUILDING-WINDOWS.md). Design notes: [docs/PORTING.md](docs/PORTING.md).

## Releasing
Releases are published by CI when a version tag is pushed. Versions are the upstream version plus `-win.N` for the port's own releases (e.g. `5.0.0-win.2`).

1. Set the version in `meson.build`.
2. Write the release notes in `docs/releases/<version>.md` (they become the GitHub Release text) and move the `Unreleased` entries in [CHANGELOG.md](CHANGELOG.md) under the new version.
3. Commit and push to `main`.
4. Tag and push the tag:

   ```powershell
   git tag -a v5.0.0-win.2 -m "Shortwave for Windows 5.0.0-win.2"
   git push origin v5.0.0-win.2
   ```

CI builds and tests on Windows and Linux, then publishes the release with the installer and the portable zip. It publishes nothing if the built files don't match the tag's version or the release notes are missing. Details: [docs/BUILDING-WINDOWS.md](docs/BUILDING-WINDOWS.md#releasing).

## Code Of Conduct
We follow the [GNOME Code of Conduct](https://conduct.gnome.org/). All
communications in project spaces are expected to follow it.
