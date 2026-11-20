# 03 — Data model (SQLite / Diesel)

Line numbers are 1-based against the `windows-port` baseline.

## Where the database lives

| Item | Value | Source |
|---|---|---|
| Data dir | `glib::user_data_dir()` + `config::NAME` (`"Shortwave"`) | [src/path.rs:25-29](../../src/path.rs#L25-L29) |
| DB file | `<data dir>/Shortwave.db` | [src/database/connection.rs:36-41](../../src/database/connection.rs#L36-L41) |
| Cache dir | `glib::user_cache_dir()` + `"Shortwave"` (cover cache, cacache) | [src/path.rs:31-35](../../src/path.rs#L31-L35) |
| Temp recordings | `<data dir>/recording/<track-uuid>.ogg`, wiped at start-up | [src/audio/track.rs:79-83](../../src/audio/track.rs#L79-L83), [src/audio/player.rs:146-151](../../src/audio/player.rs#L146-L151) |

Both directories are created by `path::init()` from `main()`
([src/path.rs:37-41](../../src/path.rs#L37-L41), [src/main.rs:53](../../src/main.rs#L53)).

**Resolved values**

| Platform | `user_data_dir()` | `user_cache_dir()` |
|---|---|---|
| Linux | `$XDG_DATA_HOME` or `~/.local/share` | `$XDG_CACHE_HOME` or `~/.cache` |
| Windows (GLib) | `CSIDL_LOCAL_APPDATA` → `%LOCALAPPDATA%` | `%LOCALAPPDATA%\Microsoft\Windows\INetCache` (GLib's Win32 choice) |

So on Windows the DB lands at `%LOCALAPPDATA%\Shortwave\Shortwave.db` with no
code change. The cache, however, would land inside the browser cache folder
(`INetCache\Shortwave`), which Windows' Disk Cleanup can purge. The port should
pin the cache to `%LOCALAPPDATA%\Shortwave\cache` (see
[08-platform-coupling.md](08-platform-coupling.md)). Note the prompt's
"`%LOCALAPPDATA%\shortwave`" is lower-case; NTFS is case-insensitive, and
keeping upstream's `Shortwave` avoids a second directory on case-sensitive
setups.

## Schema

Migrations are embedded at compile time with `diesel_migrations::embed_migrations!`
([src/database/connection.rs:31](../../src/database/connection.rs#L31))
from [data/database/migrations/](../../data/database/migrations/) and run on
every start-up by `init_connection_pool()`
([connection.rs:51-65](../../src/database/connection.rs#L51-L65)). The pool
holds **one** connection (`max_size(1)`).

History (all on a single table, `library`):

| Migration | Change |
|---|---|
| `2019-07-28-193715_init` | `library(id INTEGER PK AUTOINCREMENT, station_id INTEGER)` |
| `2020-01-18-182947_station_uuid` | `station_id INTEGER` → `stationuuid TEXT` (radio-browser moved to UUIDs) |
| `2021-05-07-232620_station_data` | Primary key becomes `uuid TEXT`; adds `is_local BOOLEAN DEFAULT FALSE`, `data TEXT` |
| `2021-11-09-134911_local_favicons` | Adds `favicon BLOB` |

Current shape — [src/database/schema.rs:17-24](../../src/database/schema.rs#L17-L24):

```sql
CREATE TABLE library (
    uuid     TEXT    NOT NULL PRIMARY KEY,  -- radio-browser stationuuid, or a random v4 UUID for local stations
    is_local BOOLEAN NOT NULL DEFAULT FALSE,
    data     TEXT,                          -- serde_json of StationMetadata (cache for RB stations, mandatory for local)
    favicon  BLOB                           -- PNG of a user-supplied cover (local stations only)
);
```

## Diesel model and queries

`StationEntry` (`Queryable`, `Insertable`) mirrors the table one-to-one —
[src/database/models.rs:23-38](../../src/database/models.rs#L23-L38).
`StationEntry::for_station()` serialises `SwStation::metadata()` to JSON and the
custom cover texture to PNG ([models.rs:42-55](../../src/database/models.rs#L42-L55)).

[src/database/queries.rs](../../src/database/queries.rs):

| Function | SQL | Lines |
|---|---|---|
| `stations()` | `SELECT * FROM library` | 28-32 |
| `insert_station()` | `INSERT INTO library …` | 34-40 |
| `update_station()` | `REPLACE INTO library …` | 42-48 |
| `delete_station(uuid)` | `DELETE FROM library WHERE uuid = ?` | 50-54 |

Every query calls `.unwrap()` on the pooled connection; a locked or unwritable
DB file panics.

## Library (`SwLibrary`)

[src/database/library.rs](../../src/database/library.rs)

- **Load** (`constructed`, [library.rs:51-107](../../src/database/library.rs#L51-L107)):
  reads all rows; local stations without `data` are deleted; RB stations without
  cached data get `StationMetadata::default()`; `favicon` bytes become a
  `gdk::Texture`.
- **Refresh** (`update_data`, [library.rs:116-164](../../src/database/library.rs#L116-L164)):
  after the server lookup, POSTs all non-local UUIDs to `json/stations/byuuid`,
  writes the fresh metadata back with `REPLACE`, and marks stations that the
  server no longer knows as **orphaned** (`set_is_orphaned(true)`).
- **Add / remove**: [library.rs:166-182](../../src/database/library.rs#L166-L182).
- **Status** (`SwLibraryStatus` = `Content | Empty | Offline`,
  [src/database/library_status.rs:20-29](../../src/database/library_status.rs#L20-L29)):
  `update_library_status()` only switches between `Content` and `Empty`
  ([library.rs:188-198](../../src/database/library.rs#L188-L198)).

Sorting (`library-sorting`, `library-sorting-type`) happens in the UI with
`SwStationSorter`, not in SQL.

## Windows notes

- `diesel` with the `sqlite` feature links `libsqlite3-sys`; under MSYS2 it finds
  `sqlite3.pc` from `mingw-w64-ucrt-x86_64-sqlite` through pkg-config (or use the
  crate's `bundled` feature). Evidence in [09-dependency-matrix.md](09-dependency-matrix.md).
- The DB path goes through `to_str().unwrap()` in a log line
  ([library.rs:58](../../src/database/library.rs#L58)) and as the
  `ConnectionManager` URL. A Windows profile path with characters outside the
  ANSI code page is still valid UTF-8, so this is safe; SQLite's Win32 VFS takes
  UTF-8 file names.
