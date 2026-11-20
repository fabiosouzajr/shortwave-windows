# 02 — radio-browser.info API protocol

How Shortwave 5.0.0 finds a radio-browser server, which endpoints it calls,
what the payloads look like, and how station covers are fetched and cached.
Line numbers are 1-based and refer to the `windows-port` baseline (upstream
commit `ce4be13`).

## Server selection

`client::lookup_rb_server()` — [src/api/client.rs:87-139](../../src/api/client.rs#L87-L139)

1. Reads the GSettings key `api-lookup-domain` (default
   `all.api.radio-browser.info`, see [04-settings.md](04-settings.md)).
2. Builds a DNS resolver with `async_std_resolver::resolver_from_system_conf()`.
   If that fails it logs `Unable to use dns resolver from system conf` and falls
   back to `ResolverConfig::default()` (Google public DNS) —
   [client.rs:89-97](../../src/api/client.rs#L89-L97).
3. **Forward lookup** of the lookup domain → list of A/AAAA records; the list
   is shuffled with `rand::thread_rng()` so load spreads across servers
   ([client.rs:100-104](../../src/api/client.rs#L100-L104)).
4. For each IP: **reverse lookup** (PTR) to get the server's hostname, strip the
   trailing `.` (otherwise TLS hostname verification fails), then probe it with
   `GET https://<host>/json/stats`. The first host that answers becomes the API
   base URL `https://<host>/` ([client.rs:106-136](../../src/api/client.rs#L106-L136)).
5. The result is stored in `SwApplication.rb_server` and announced with
   `notify("rb-server")`; on success the library is refreshed
   ([src/app.rs:280-294](../../src/app.rs#L280-L294)). This runs once from
   `ApplicationImpl::startup` ([src/app.rs:151-175](../../src/app.rs#L151-L175)).

`build_url()` returns `Error::NoServerAvailable` while `rb_server` is `None`
([client.rs:141-158](../../src/api/client.rs#L141-L158)), so every request made
before the lookup finishes fails with that error.

**Windows note:** `resolver_from_system_conf()` on Windows reads the adapter DNS
configuration through `ipconfig` (hickory-resolver's Windows backend), so no
`/etc/resolv.conf` is needed. The fallback path also works. Reverse lookups need
PTR records from the network's resolver; some corporate/VPN resolvers don't
answer them, in which case every server is skipped (same behaviour as on Linux).

## Endpoints

Defined in [src/api/mod.rs:18-20](../../src/api/mod.rs#L18-L20):

| Constant | Path | Method | Caller | Body / query |
|---|---|---|---|---|
| `STATS` | `json/stats` | GET | `server_stats()` [client.rs:160-167](../../src/api/client.rs#L160-L167) | none |
| `STATION_SEARCH` | `json/stations/search` | GET | `station_request()` [client.rs:60-72](../../src/api/client.rs#L60-L72) | query string = url-encoded `StationRequest` |
| `STATION_BY_UUID` | `json/stations/byuuid` | POST | `station_metadata_by_uuid()` [client.rs:74-85](../../src/api/client.rs#L74-L85) | JSON `{"uuids":[...]}` |

No click/vote endpoints are called.

### Request: `StationRequest`

[src/api/station_request.rs:18-40](../../src/api/station_request.rs#L18-L40): every
field is `Option<_>` and is serialised with `serde_urlencoded` (`None` fields
are omitted). Fields: `name`, `name_exact`, `country`, `country_exact`,
`countrycode`, `state`, `state_exact`, `language`, `language_exact`, `tag`,
`tag_exact`, `bitrate_min`, `bitrate_max`, `has_geo_info`, `has_extended_info`,
`is_https`, `order`, `reverse`, `offset`, `limit`, `hidebroken`.

The search page uses `StationRequest::search_for_name(name, limit)` →
`?name=<q>&order=votes&reverse=true&limit=<n>`
([station_request.rs:43-51](../../src/api/station_request.rs#L43-L51)).

### Response: `StationMetadata`

[src/api/station_metadata.rs:25-69](../../src/api/station_metadata.rs#L25-L69).
A `glib::Boxed` (`SwStationMetadata`) that mirrors radio-browser's station
object: `changeuuid`, `stationuuid`, `name`, `url`, `url_resolved`, `homepage`,
`favicon` (all URLs parsed leniently: invalid → `None`), `tags`, `country`,
`countrycode`, `state`, `language`, `languagecodes`, `votes`,
`lastchangetime(_iso8601)`, `codec`, `bitrate`, `hls`, `lastcheckok`,
`lastchecktime(_iso8601)`, `lastcheckoktime(_iso8601)`, `lastlocalchecktime`,
`clicktimestamp(_iso8601)`, `clickcount`, `clicktrend`, `ssl_error`, `geo_lat`,
`geo_long`, `has_extended_info`.

The same struct is serialised to JSON and stored in the `library.data` column
for favourites (see [03-data-model.md](03-data-model.md)).

### Response: `Stats`

[src/api/stats.rs:17-29](../../src/api/stats.rs#L17-L29): `supported_version`,
`software_version`, `status`, `stations`, `stations_broken`, `tags`,
`clicks_last_hour`, `clicks_last_day`, `languages`, `countries`. Only
`software_version` and `stations` are used, for a debug log line.

## HTTP client

[src/api/client.rs:36-58](../../src/api/client.rs#L36-L58)

- `reqwest 0.12` with default features → **native-tls**. On Linux that is
  OpenSSL; on Windows `cargo tree --target x86_64-pc-windows-gnu -i schannel`
  shows it resolves to **SChannel** (no OpenSSL), see
  [09-dependency-matrix.md](09-dependency-matrix.md).
- User-Agent: `"{PKGNAME}/{VERSION}-{PROFILE}"`, e.g. `shortwave/5.0.0-default`
  ([client.rs:36-43](../../src/api/client.rs#L36-L43)).
- Default header `content-type: application/json`, 15 s timeout.
- Requests run under `async_compat::Compat` because reqwest needs a Tokio
  reactor while the app runs futures on the GLib main context
  (`send_request_compat`, [client.rs:184-186](../../src/api/client.rs#L184-L186)).

### Error handling

`send_request()` ([client.rs:169-182](../../src/api/client.rs#L169-L182)) reads
the body as text and deserialises it; on failure it logs the raw body and
returns `Error::Deserializer`. Transport errors are wrapped as `Rc<reqwest::Error>`.
Errors bubble up to the UI through `display_error::ToastWindow` toasts.

## Covers

`CoverLoader` — [src/api/cover_loader.rs](../../src/api/cover_loader.rs)

- Separate `reqwest::Client` with a 5 s timeout and **no** custom User-Agent
  ([cover_loader.rs:33-38](../../src/api/cover_loader.rs#L33-L38)).
- Requests go through an unbounded `async_channel`; at most
  `max(num_processors/2, 2)` covers load concurrently
  ([cover_loader.rs:143-154](../../src/api/cover_loader.rs#L143-L154)).
- Cache key: `"{favicon_url}@{size}"` in a **cacache** store rooted at
  `path::CACHE` (`glib::user_cache_dir()/Shortwave`) —
  [cover_loader.rs:71-87](../../src/api/cover_loader.rs#L71-L87).
- Cache miss: download the favicon to a temporary `gio::File`, decode it with
  **glycin** (`glycin::Loader::new(file).load()`), draw it into a square
  thumbnail (`snapshot_thumbnail`, blurred letterbox when the aspect ratio isn't
  1:1) and render with `gsk::CairoRenderer`; the result is stored as PNG
  ([cover_loader.rs:89-110](../../src/api/cover_loader.rs#L89-L110),
  [215-273](../../src/api/cover_loader.rs#L215-L273)).
- Cache hit: PNG bytes → `gdk::Texture::from_bytes` (gdk-pixbuf / GTK's
  built-in loaders, no glycin) — [cover_loader.rs:71-77](../../src/api/cover_loader.rs#L71-L77).
- `prune_cache()` at shutdown removes the pre-4.0 `favicons/` directory and
  entries older than 30 days ([cover_loader.rs:156-176](../../src/api/cover_loader.rs#L156-L176)).

**Windows impact:** glycin runs image decoders in sandboxed helper processes
using Linux-only `libseccomp`/bubblewrap, so it can't build for Windows (see
[08-platform-coupling.md](08-platform-coupling.md)). Only the decode step in
`cover_bytes()` needs a replacement; everything around it (cacache, snapshot,
Cairo renderer, PNG export) is portable.
