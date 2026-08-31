# Residual — splora production indexer

This file is the Open leftover list for the approved production indexer work.
Chat status is not this list. Dual honesty: finished slices are not Open;
sibling paths that are still unfixed stay here.

The last review pass (`/tmp/grok-1000/grok-review-splora-prod.md`) has **0
open issues**. Queue HTTP writes only `/var/lib/splora/queue`. The allowlist
stays `/var/lib/splora/allow-npubs`. Those review items are not leftover.

## Already in the tree (do not re-open)

The crate is named `splora`. Crane builds `splora` and `splora-liquid`.
`.cargo/config.toml` rewrites crates.io to the Menhera 7-day sparse index.
`cargo-deny.toml` denies unknown registries and git sources. Novel Surmount
files use the Unlicense. Inherited electrs stays MIT.

`rust-toolchain` is `1.98.0`. `Cargo.toml` is `edition = "2024"` and
`rust-version = "1.98"`. The flake uses `pkgs.rust-bin.stable."1.98.0".default`.
`hyper = "0.14"` is kept (lockfile **0.14.32**). The clap default for `--db-block-cache-mb` is 24.
`.cargo/config.toml` still rewrites crates.io to Menhera 7-day. The 2026-08-31
lock refresh pinned `nostr` 0.44.8 and `prometheus` 0.14.0 with default
features off. `idna` 1.0.3 and `idna_adapter` 1.1.0 stayed. `rocksdb` stayed
0.24.0. NIP-98 caps encoded header size before Base64. Remaining
unmaintained rustsec warnings (clap 2 / `ansi_term` / `atty` / bincode /
nostr-`instant`) are listed in [doc/supply-chain.md](doc/supply-chain.md).
They are not leftover lock work.

Authorization is two files. Pending queue is CSV `npub,email` with no status
column (`src/queue.rs`, `tests/queue_csv.rs`). Approved allowlist is one npub
per line. NIP-98 allowlist load, reload, and verify live in `src/auth.rs`.
That verifier caps the encoded `Authorization` payload before Base64
allocates, then caps decoded JSON at 64 KiB (`MAX_NIP98_AUTH_EVENT_BYTES`,
matching nostr 0.44.8 / RUSTSEC-2026-0229). Queue HTTP is the
`splora-queue` binary. Import is `splora-import` with `approve`, `reject`,
and `remove`. The indexer clap app has no `queue` subcommand. Queue disk
writes serialize on a `Mutex`.

`--db-block-cache-mb`, `--db-parallelism`, `--enable-mining-rest`,
`--cookie-file`, `testnet4`, `getblocktemplate` / `getnewblockhex`, and
`POST /electrum` are in the tree. Electrum does not bind TCP when
`--rpc-socket-file` is omitted.

MWCK JSON lives in `src/mwck.rs`. `rest.rs` gates HTTP with the live
allowlist, upgrades `GET /api/v1/ws`, and serves `POST /electrum`. Named
unit tests cover the gate, the 101 upgrade *decision*, and JSON
`track-addresses` buckets. `live_hyper_ws_101_handshake_allowlist` is a
tiny hyper 0.14 server fixture (not a full indexer) for empty-allowlist
401, listed-npub 101, and socket close after an allowlist reload drop.
`Allowlist::load` is already `Arc`; the fixture compares tungstenite http
1 status as `u16` against hyper 0.14 401/101.

`MwckHub` holds a `Query` and fills subscribe snapshots on first
`track-addresses` / `track-scriptpubkeys`. Empty arrays until a later
notify are not the only path. Named test
`handle_socket_track_addresses_fills_known_history` is green on that
contract (fixture history; production fills from `Query`). Block notify
emits confirmed txs for every new height after the previous tip (named
test `notify_block_emits_confirmed_from_two_new_heights`). Dropped
mempool txs keep a full object in `removed` when that body is still
available (named tests `prefer_full_removed_uses_available_body_not_txid_stub`
and `notify_mempool_removed_emits_full_object_when_available`).

`ErrorKind::DaemonBusy` and `ErrorKind::DaemonUnavailable` exist.
REST daemon-proxy paths map occupancy to HTTP 503 and a missing or
timed-out daemon to HTTP 504. Named test
`daemon_proxy_failures_map_to_503_and_504` covers that mapping.
Electrum JSON-RPC stays JSON on the wire, not those HTTP statuses.

`nix/module.nix` ships five-instance NixOS options, isolated queue
`ReadWritePaths`, and an assertion that the queue directory is not the
allowlist directory. Queue listen is `--socket-file` XOR `--bind`. Default
listen is unix `/run/splora/queue.sock`. TCP needs `queueSocketFile =
null`. `systemd.timers.splora-popular-scripts` and
`systemd.services.splora-popular-scripts` are in the module when
`services.splora.popularScripts.enable` is set. The oneshot writes
`/var/lib/splora/popular-scripts/popular-scripts.txt` and
`ReadWritePaths` is that directory only. `flake.nix` exports
`nixosModules.splora`, `apps.popular-scripts`, overlays, the
`nixosFiveInstances` eval check, `nixosQueueListenXorSocket` (default
socket plus `queueListen` must fail the module assertion), and named
check `rocksdbMoldLink` (`splora-nixpkgs-rocksdb-mold`). README matches
the production argv (CLI versus module). There is no nginx in this tree.
Public TLS, HTTP/2, HTTP/3, and cipher suites belong on the first-party
Axum edge in surmount-server. splora on the unix socket is local
cleartext HTTP/1.1. QUIC cannot sit on a Unix domain socket. The README
and [FORK.md](FORK.md) section 5 state that socket contract. This
session did not edit surmount-server.

The indexer was not rewritten. `mempool/mempool` was not vendored.

Repo-root [FORK.md](FORK.md) names lineage, the Mempool schema lock, Blockstream ports that were copied without merging trees, Surmount-only modules, and the HTTP/2 HTTP/3 split (section 5). README points at that file. Do not list `FORK.md` as Open.

## Open

### Operator-owned gates

The operator must run `just check-local` and then `just check-remote`.
This wave ran `cargo deny check --config cargo-deny.toml` and `cargo
audit` (both exit 0) after the lock refresh. Agents did not prove
crate-wide cargo fmt, clippy, nextest, or `nix flake check` on this
laptop. Named unit tests are in the tree. They are not a substitute
for those two recipes.

The flake sets `useSystemRocksdb = true` and bindgen against nixpkgs
headers (rocksdb 10.10.1, SONAME `librocksdb.so.10`). The crate vendor
stays `librocksdb-sys` 0.17.3+10.4.2. Named check `rocksdbMoldLink`
(`splora-nixpkgs-rocksdb-mold`) requires both crane binaries to `NEEDED`
`librocksdb` and show mold in `.comment`. Agents did not run
`just check-remote`. The system RocksDB plus mold link is therefore
unproven on the builder. A laptop `cargo` ELF with `ROCKSDB_*` from
nixpkgs did `NEEDED` `librocksdb.so.10`; that ELF `.comment` is Wild
0.10.0, not mold, and is not the crane check. The bundled `rocksdb`
0.24.0 fallback, and how to set `useSystemRocksdb = false`, is in
`doc/supply-chain.md`.

Nix flakes only see git-tracked files. `flake.nix`, `flake.lock`,
`nix/module.nix`, and other new packaging files stay invisible to
`just check-remote` until the operator stages them. Agents do not stage
those files.

### HTTP/2 and HTTP/3 (other tree)

HTTP/2 and HTTP/3 are not served on the splora unix socket. They
terminate on surmount-server Axum (`sploraProxy`, `http3Enable`). This
crate's unix sockets stay local cleartext HTTP/1.1. QUIC cannot sit on a
Unix domain socket. Integration already started in that other tree:
`SURMOUNT_SPLORA_*`, instance names, and a refuse of Electrum newline
sockets on the HTTP proxy. Remaining work, if any, is operator enable of
that proxy on the edge host, not nginx or HTTP/2 inside this crate. This
session did not edit surmount-server. Do not add nginx here.

### Agent-doable leftover

The 2026-08-31 Menhera lock refresh, nostr 0.44.8, prometheus 0.14.0
without protobuf, NIP-98 header size cap before Base64, held `idna`
1.0.3 / `idna_adapter` 1.1.0 pins, `rocksdb` 0.24.0, `cargo audit` exit
0, and `cargo deny check --config cargo-deny.toml` exit 0 are in the
tree. They are not leftover.

Clap 2 still pulls unmaintained `ansi_term` and `atty`. stderrlog 0.5
still pulls `atty`. bincode 1.3.3 is still the on-disk schema. nostr
0.44.8 still lists `instant` for wasm32 in the lockfile. Those rustsec
**warnings** are documented in [doc/supply-chain.md](doc/supply-chain.md).
They stay until a clap 4 CLI rewrite, a schema codec change, or a nostr
line that drops `instant`. That rewrite is not this wave. Do not start
clap 4 unless the operator wants those unmaintained rows gone.

The CSV two-file queue, unix-socket defaults, queue XOR bind, queue
mutex, popular-scripts timer and isolated output dir, live hyper WS
101 fixture, MWCK subscribe fill from `Query` on first track, REST
503/504 on daemon-proxy paths, named `rocksdbMoldLink` check (unproven
on the builder), README CLI versus module, no nginx, and the
surmount-server socket contract are already in this tree.

`nixosFiveInstances` now requires the default queue unit to carry
`--socket-file` `/run/splora/queue.sock` and to omit TCP `--bind`. That
matches `nix/module.nix`. Operator still owns `nix flake check`.

Named flake check `rocksdbMoldLink` is in the tree. The operator still
has to prove it with `just check-remote` after those flake files are
tracked. If that link fails, set `useSystemRocksdb = false` and keep
bundled 0.24.0.

### Sibling product paths still unfixed

Block notify walks heights after the previous tip. It does not emit a
full reorg replay of orphaned blocks. A tip that moves backward is a
no-op. That would need indexer reorg replay, which this slice did not
do.

Removed mempool transactions that are gone from both mempool and chain
are still notified as `{txid}` only. Full objects are used when the tx
is still on chain or in fixture history. This slice does not snapshot
the whole mempool on every loop to keep evicted bodies.

hyper 0.14.32 is in the lock. `Server::http1_header_read_timeout` exists
on that line. This tree did not wire that timeout.

Each RocksDB open gets its own LRU block cache. There is not one shared
cache across txstore, history, and cache. Sharing would need `schema.rs`.

Allowlist tests call `reload()` after a file write. There is no live
inotify integration test of `Allowlist::watch`.

`pkgs.nixosTest` was not added. The cheap `nixosFiveInstances` eval does
not boot five indexers and does not prove approve-then-HTTP-without-restart
on a VM.

## Highest value next

The operator still owns `just check-local` then `just check-remote` after
the flake files are staged. Agents do not run those recipes as crate-wide
proof on this laptop. The next proof on the builder is named check
`rocksdbMoldLink` (crane `NEEDED librocksdb` plus mold in `.comment`).
A laptop Wild-linked `librocksdb.so.10` ELF is not that proof.

A clap 4 rewrite is next on this crate only if the operator wants the
unmaintained clap 2 / `ansi_term` / `atty` rustsec warnings gone.
bincode schema and nostr-`instant` stay until a codec change or a
different nostr line. Do not treat those warnings as a lock-only job.

If public HTTP/2 and HTTP/3 are wanted, enable the existing
surmount-server `sploraProxy` (and HTTPS `http3Enable`) on that other
tree. Do not add nginx or HTTP/2 in this crate. This session did not
edit surmount-server.

Parked sibling gaps on this tree (not the next proof) are full MWCK
reorg replay, evicted mempool txs that are gone from chain staying
`{txid}` only, wiring hyper 0.14.32 HTTP/1 header-read timeout, shared
LRU, a live inotify watch test, and a `nixosTest` VM.
