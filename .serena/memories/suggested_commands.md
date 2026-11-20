# Suggested commands (Windows)

- MSYS2 shell commands from PowerShell: set `$env:MSYSTEM='UCRT64'; $env:CHERE_INVOKING='1'` then `C:\msys64\usr\bin\bash.exe -lc '<cmd>'` (single-quote the bash script so PowerShell doesn't expand `$`).
- Toolchain bootstrap: `build-aux\windows\setup-msys2.ps1` (idempotent; `-SkipUpdate` skips pacman -Syu).
- Dependency analysis: `cargo tree -e normal --target x86_64-pc-windows-gnu -i <crate>` (ambiguous crates need `name@version`).
- Git: never push `upstream`/`gnome`; push `origin windows-port`. `.gitattributes` forces LF (core.autocrlf=true on this machine).
