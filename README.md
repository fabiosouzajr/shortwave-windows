# Shortwave

![screenshot](data/screenshots/1.png)
![screenshot](data/screenshots/4.png)

Shortwave is an internet radio player that provides access to a station database with over 50,000 stations.

This is an **unofficial Windows 10/11 (x64) port** of [GNOME Shortwave](https://gitlab.gnome.org/World/Shortwave). It is not affiliated with the Shortwave authors or the GNOME project; see [NOTICE.md](NOTICE.md). Please report problems with the Windows build here, not upstream.

## Installation
Build the installer or the portable zip yourself (see [Building](#building)):

- `dist\Shortwave-<version>-windows-x64-setup.exe`: Inno Setup installer (per-user or per-machine).
- `dist\Shortwave-<version>-windows-x64.zip`: portable, self-contained bundle. Unzip and run `shortwave.exe`.

## Features on Windows
Search, playback (HTTP/HTTPS/HLS), library, recording (saved to `Music\Shortwave`), media keys and the Windows media overlay, a tray icon for background playback, toast notifications, and dark/light mode following Windows. Chromecast discovery works, but casting itself is untested. See [docs/WINDOWS-PORT-STATUS.md](docs/WINDOWS-PORT-STATUS.md) for details.

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
.\build-aux\windows\setup-msys2.ps1   # install build dependencies (once)
.\build-aux\windows\build.ps1 -Run    # build and start the app
.\build-aux\windows\bundle.ps1        # dist\Shortwave + portable zip
.\build-aux\windows\installer.ps1     # Inno Setup installer (needs Inno Setup 6)
```

Full instructions: [docs/BUILDING-WINDOWS.md](docs/BUILDING-WINDOWS.md). Design notes: [docs/PORTING.md](docs/PORTING.md).

## Code Of Conduct
We follow the [GNOME Code of Conduct](https://conduct.gnome.org/). All
communications in project spaces are expected to follow it.
