# 00 — Tooling

How the reverse-engineering in `docs/re/` was done, and with which tools.

## Serena MCP

Serena 2.1.1.dev0, installed as a Claude Code plugin (`plugin:serena:serena`),
context `desktop-app`, modes `editing` + `interactive`, agent interface
**REPL**. Project `shortwave-windows` activated at `D:\git\shortwave-windows`
(config in `.serena/project.yml`, `language_servers: [rust]`).

### Tools exposed to the agent

The REPL interface exposes only three MCP tools; everything else is reached as
methods on facade objects inside `serena_repl`.

| MCP tool | Description |
|---|---|
| `initial_instructions` | Returns the Serena manual and the session id |
| `activate_project` | Activates a registered project or a directory path |
| `serena_repl` | Runs Python against the facade object `s`; results of the last expression are rendered |

| Facade | Methods (one line each) |
|---|---|
| `s.lsp` | `get_symbols_overview` (symbols of a file), `find_symbol` (by name path, optional body), `find_referencing_symbols` (callers/usages), `find_declaration`, `find_implementations`, `get_diagnostics_for_file`, `rename_symbol` (reference-aware), `safe_delete_symbol` |
| `s.fs` | `search_for_pattern` (regex over files, 0-based line numbers), `list_dir`, `find_file`, `read_file`, `create_text_file` |
| `s.edit` | `replace_content` (string/regex), `replace_in_files` (with dry-run), `replace_symbol_body`, `insert_before_symbol`, `insert_after_symbol` |
| `s.mem` | `onboarding`, `write_memory`, `read_memory`, `list_memories`, `edit_memory`, `rename_memory`, `delete_memory` |
| `s.shell` | `execute_shell_command` |
| `s.cfg` | `get_current_config` |

`get_current_config()` additionally lists these underlying tools as available
but not exposed directly (they back the facades): `create_text_file`,
`delete_lines`, `delete_memory`, `edit_memory`, `execute_shell_command`,
`find_declaration`, `find_file`, `find_implementations`,
`find_referencing_symbols`, `find_symbol`, `get_current_config`,
`get_diagnostics_for_file`, `get_diagnostics_for_symbol`,
`get_symbols_overview`, `insert_after_symbol`, `insert_at_line`,
`insert_before_symbol`, `jet_brains_*` (JetBrains backend only),
`list_dir`, `list_memories`, `list_queryable_projects`, `onboarding`,
`open_dashboard`, `query_project`, `read_file`, `read_memory`,
`remove_project`, `rename_memory`, `rename_symbol`, `replace_content`,
`replace_in_files`, `replace_lines`, `replace_symbol_body`,
`restart_language_server`, `safe_delete_symbol`, `search_for_pattern`,
`serena_info`, `write_memory`.

### Getting Serena connected (Phase 0 notes)

1. First registration was `claude mcp add serena …` at **local** scope. The VS
   Code extension session opened the folder as `d:\git` while the CLI recorded
   `D:\git`, and the server never appeared in the session.
2. The user re-registered it; in the next session Serena was available as the
   `plugin:serena:serena` server.

### Indexing

- `get_symbols_overview("src/app.rs")` and `find_symbol("SwApplication")` both
  return results → the workspace is indexed.
- **Degraded:** rust-analyzer reports `proc-macro not yet built` on every file
  that uses `glib::clone!`, `#[derive(Properties)]`, `CompositeTemplate` or
  Diesel derives. Cause: rustup's default toolchain is
  `stable-x86_64-pc-windows-msvc` and **no Visual Studio Build Tools** are
  installed (`vswhere` absent, no `link.exe`), so rust-analyzer can't compile
  proc-macro crates. Also `src/config.rs` doesn't exist until Meson configures
  (generated temporarily for probing).
- Effect: symbol overviews, bodies and pattern search work; references that
  pass through macro-expanded code are missing (e.g. `find_referencing_symbols`
  on `SwApplication` finds 2 of the many call sites).
- The prompt's suggested fallback — MSYS2 bootstrap first — was done
  (`build-aux/windows/setup-msys2.ps1`). It gives `pkg-config` and the GTK
  libraries, but does **not** fix proc-macros, because Serena still launches
  the rustup (MSVC) rust-analyzer. Fixing that needs one of:
  1. `rustup toolchain install stable-x86_64-pc-windows-gnu` +
     `rustup component add rust-analyzer --toolchain stable-x86_64-pc-windows-gnu`,
     and run Serena with `RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-gnu` and
     `C:\msys64\ucrt64\bin` on `PATH`; or
  2. install Visual Studio Build Tools (C++ workload).
  Both install system software, so they are left for the user to decide
  (Checkpoint A).
- **Resolved (Phase 4):** the user installed the VS 2026 Build Tools C++
  workload (MSVC 14.52). After the next `Cargo.toml` change rust-analyzer
  reloaded, proc-macro diagnostics disappeared, and
  `find_referencing_symbols("integer", "src/settings/settings_manager.rs")`
  returns 7 references — the same as ripgrep, including the call inside a
  `glib::clone!` block (`src/audio/player.rs:188`). What worked: **option 2**.
  Serena still runs rust-analyzer for the MSVC host; `*-sys` build scripts that
  need `pkg-config` fail there, which only affects generated sys bindings, not
  the app's own symbols.

### Which tool produced which report

| Report | Primary tools |
|---|---|
| [01-architecture.md](01-architecture.md) | `lsp.get_symbols_overview`, `lsp.find_symbol(include_body)`, ripgrep for `const NAME` / `ParentType`, `git diff` vs GNOME tag |
| [02-api-protocol.md](02-api-protocol.md) | `lsp.find_symbol` bodies of `client.rs`, `cover_loader.rs`; file reads of the serde structs |
| [03-data-model.md](03-data-model.md) | full reads of SQL migrations and `schema.rs` (prompt allows), `lsp.find_symbol` for `SwLibrary`, `connection.rs` |
| [04-settings.md](04-settings.md) | full read of the gschema; ripgrep `Key::\w+` for read/write sites |
| [05-audio-pipeline.md](05-audio-pipeline.md) | `lsp.find_symbol` bodies of `GstreamerBackend`, `SwPlayer`; `gst-inspect-1.0` |
| [06-casting.md](06-casting.md) | `lsp.find_symbol` bodies in `device/`; reading `cast-sender 0.2.0` sources in the cargo registry |
| [07-ui-inventory.md](07-ui-inventory.md) | ripgrep `#[template(resource=…)]`, `install_action`; reads of `window.ui`, gresource XML |
| [08-platform-coupling.md](08-platform-coupling.md) | `fs.search_for_pattern` with the prompt's coupling regex, then `lsp.find_symbol`; `strings` on `libgio-2.0-0.dll` / `libgtk-4-1.dll` |
| [09-dependency-matrix.md](09-dependency-matrix.md) | `cargo tree -i`, `cargo check --keep-going` under MSYS2, `pkg-config`, `pacman -Si/-Ss/-Q` |

Line-number caveat: Serena's `search_for_pattern` and symbol locations are
0-based; all references in these reports were converted to 1-based and spot
checked with ripgrep.

## Other tools

| Tool | Version | Used for |
|---|---|---|
| MSYS2 UCRT64 | pacman 6.1.0 | toolchain + GTK/GStreamer stack (see [09](09-dependency-matrix.md)) |
| cargo (MSYS2) | 1.100.0 | `cargo check` probe of the unmodified tree |
| cargo (rustup, MSVC host) | 1.100.0 | `cargo tree` (resolution only, no build) |
| meson | 1.12.1 | `meson setup` probe |
| binutils `strings` (MSYS2) | — | checking GLib/GTK Win32 backends in DLLs |
| gh | 2.105.0 | private repo creation |

Phase 2b (runtime observation under WSL2) has **not** been run; it needs the
user's approval.
