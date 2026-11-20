# Task completion

- Windows: build under MSYS2 UCRT64 (Meson / `build-aux/windows/build.ps1` once it exists), `cargo clippy`, smoke-test the app.
- Linux build must not regress (CI Linux job once added).
- Claims need evidence (command output) — record in `docs/` when relevant.
- Commit + push `origin windows-port`.
