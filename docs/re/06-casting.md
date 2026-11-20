# 06 — Google Cast

Line numbers are 1-based against the `windows-port` baseline.
Code: [src/device/](../../src/device/) plus the `cast-sender 0.2.0` and
`mdns-sd 0.13` crates.

## Objects

| Type | File | Role |
|---|---|---|
| `SwDeviceDiscovery` | [device_discovery.rs](../../src/device/device_discovery.rs) | mDNS browse, fills an `SwDeviceModel` |
| `SwDevice` | [device.rs](../../src/device/device.rs) | id, kind, name, model, address |
| `SwDeviceKind` | [device_kind.rs](../../src/device/device_kind.rs) | only `Cast` today |
| `SwCastSender` | [cast_sender.rs](../../src/device/cast_sender.rs) | Cast connection, media controller, volume |
| `SwDeviceDialog` / `SwDeviceRow` / `SwDeviceIndicator` | [src/ui/](../../src/ui/) | picker dialog, list rows, header indicator |

`SwPlayer` owns `device_discovery` and `cast_sender`
([src/audio/player.rs:80-83](../../src/audio/player.rs#L80-L83)) and binds its
`volume` property bidirectionally to `SwCastSender:volume`
([player.rs:133-137](../../src/audio/player.rs#L133-L137)).

## Discovery

`discover_cast_devices()` — [device_discovery.rs:68-94](../../src/device/device_discovery.rs#L68-L94)

- `mdns_sd::ServiceDaemon::new()` then `browse("_googlecast._tcp.local.")`
  ([device_discovery.rs:35](../../src/device/device_discovery.rs#L35)).
- On every `ServiceResolved` event, the first resolved address becomes the host;
  TXT records give `id` (falls back to the host), `fn` (friendly name, falls back
  to *"Google Cast Device"*) and `md` (model, falls back to *"Unknown Model"*).
- `scan()` drives this while the device dialog is open
  ([device_discovery.rs:107-125](../../src/device/device_discovery.rs#L107-L125)).

`mdns-sd` is a pure-Rust daemon: it opens its **own** UDP sockets on port 5353
(multicast 224.0.0.251 / ff02::fb) on every interface. It does not use Avahi
or Bonjour, so it works on Windows without extra services.

## Connection and playback hand-over

`SwCastSender::connect(ip)` — [cast_sender.rs:127-149](../../src/device/cast_sender.rs#L127-L149):

1. `Receiver::connect(ip)` → TCP **8009**, TLS through `async-native-tls` with
   `danger_accept_invalid_certs(true)` (Cast devices use self-signed certs) —
   `cast-sender-0.2.0/src/client.rs:35-39`.
2. `launch_app(AppId::Custom("E3F31F9F"))` — Shortwave's own **custom web
   receiver** app, not the Default Media Receiver. That app ID is registered
   with Google by upstream; the port must keep it unchanged.
3. Create a `MediaController` on the launched app's transport.

`load()` ([cast_sender.rs:80-110](../../src/device/cast_sender.rs#L80-L110)) reads
the receiver volume, then sends a media `LOAD` with:

```json
{
  "contentId": "<station stream URL>",
  "streamType": "LIVE",
  "contentType": "audio/*",
  "metadata": { "metadataType": 3, "title": "<station title>",
                "images": [ { "url": "<station favicon URL>" } ] }
}
```

`metadataType: 3` is `MusicTrackMediaMetadata`. The receiver fetches the stream
itself, so the sender's network only carries control messages.

Namespaces used (from `cast-sender-0.2.0/src/namespace/mod.rs`):
`urn:x-cast:com.google.cast.tp.connection`, `…tp.heartbeat` (PING/PONG),
`…receiver` (LAUNCH, GET_STATUS, SET_VOLUME), `…media` (LOAD, PLAY, PAUSE/STOP).

`SwPlayer::connect_device()` / `disconnect_device()`
([player.rs:618-664](../../src/audio/player.rs#L618-L664)) stop local playback,
hand the current station to the sender and back. Volume changes go through
`receiver.set_volume(level, false)` ([cast_sender.rs:61-78](../../src/device/cast_sender.rs#L61-L78)).

## TLS backend

`cargo tree -e normal --target x86_64-pc-windows-gnu -i schannel`:
`schannel → native-tls → async-native-tls → cast-sender`. So on Windows the Cast
TLS is **SChannel** and no OpenSSL is needed. SChannel supports
`danger_accept_invalid_certs`. Evidence in [09-dependency-matrix.md](09-dependency-matrix.md).

## Windows risks

| Risk | Detail | Mitigation |
|---|---|---|
| Firewall blocks mDNS replies | Inbound multicast UDP 5353 to `shortwave.exe` is blocked until allowed; Windows shows the "allow access" prompt the first time the socket binds (often only for Private networks) | Installer adds an inbound UDP 5353 rule for the exe (prompt requirement); document it |
| Port 5353 shared with Windows' own mDNS (Dnscache) | `mdns-sd` binds with `SO_REUSEADDR`, which on Windows allows sharing | Verify in Phase 4/9 on the real device |
| Multiple interfaces (Hyper-V, WSL, VPN adapters) | mdns-sd queries all of them; the first address in `ServiceResolved` may belong to a virtual adapter's subnet | Low; verify, prefer an address on the same subnet as a local interface if needed |
| Outbound TCP 8009 | Normally allowed by default policy | — |

## Windows verification (Phase 4, workstream 9)

No real Cast device was available. What was checked on Windows 11 (Hyper-V
and WSL adapters present):

- **Firewall:** the first runs of `shortwave.exe` triggered the Windows
  Firewall prompt, which created inbound *Allow* rules for the exe (seen with
  `Get-NetFirewallApplicationFilter`). Without them, mDNS replies are dropped →
  the installer must add the rule.
- **Discovery:** a scratch program using the same `mdns-sd 0.13.9` advertised
  `_googlecast._tcp.local.` (`fn=Test Cast Device`, `md=Fake Chromecast`) from
  this machine's LAN address. Shortwave's device dialog listed
  *Test Cast Device / Fake Chromecast*: multicast send/receive and sharing
  UDP 5353 with Windows' own mDNS work, and TXT records parse.
- **Connection error path:** selecting the fake device (nothing listening on
  8009) shows the toast *Unable to connect with device … (os error 10061)*; the
  app keeps running.
- **Not tested:** the TLS handshake with a real receiver (SChannel with
  `danger_accept_invalid_certs`), launching the custom receiver app
  `E3F31F9F`, media `LOAD`, volume sync and hand-back to local playback.
