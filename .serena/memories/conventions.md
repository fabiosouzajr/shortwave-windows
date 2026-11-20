# Conventions

- GObject subclasses: `mod imp` + `#[derive(Properties)]`, `glib::wrapper!`, names prefixed `Sw`.
- Async on GLib main context: `glib::spawn_future_local`; threads -> UI via `async-channel`; reqwest futures wrapped in `async_compat::Compat`.
- Settings only through `settings::settings_manager` + `Key` enum (strum kebab-case).
- Errors to UI via `handle_error(...)` / toasts (`display_error.rs`).
- New files: GPL-3.0-or-later header like upstream (`// Shortwave - <file>` + license block).
- Commits: Conventional Commits (`feat(win):`, `fix(audio):`, `docs(re):`), small, with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
