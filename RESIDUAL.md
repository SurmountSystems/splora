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

Authorization is two files. Pending queue is CSV `npub,email` with no status
column (`src/queue.rs`, `tests/queue_csv.rs`). Approved allowlist is one npub
per line. NIP-98 allowlist load, reload, and verify live in `src/auth.rs`.
Queue HTTP is the `splora-queue` binary. Import is `splora-import` with
`approve`, `reject`, and `remove`. The indexer clap app has no `queue`
subcommand. Queue disk writes serialize on a `Mutex`.

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
contract (fixture history; production fills from `Query`).

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
cleartext. The README states that socket contract. This session did not
edit surmount-server.

The indexer was not rewritten. `mempool/mempool` was not vendored.

Repo-root [FORK.md](FORK.md) names lineage, the Mempool schema lock, Blockstream ports that were copied without merging trees, and Surmount-only modules. README points at that file. Do not list `FORK.md` as Open.

## Open

### Operator-owned gates

The operator must run `just check-local` and then `just check-remote`.
Agents did not prove crate-wide cargo fmt, clippy, deny, audit, nextest, or
`nix flake check` on this laptop. Named unit tests are in the tree. They
are not a substitute for those two recipes.

The flake sets `useSystemRocksdb = true` and exports named check
`rocksdbMoldLink` (`splora-nixpkgs-rocksdb-mold`): both crane binaries
must `NEEDED` `librocksdb` and show mold in `.comment`. Pinned nixpkgs
rocksdb is 10.10.1 versus crate `librocksdb-sys` 0.17.3+10.4.2. Agents
did not run `just check-remote`. The system RocksDB plus mold link is
therefore unproven on the builder. The bundled `rocksdb` 0.24.0 fallback,
and how to set `useSystemRocksdb = false`, is in `doc/supply-chain.md`.

Nix flakes only see git-tracked files. `flake.nix`, `flake.lock`,
`nix/module.nix`, and other new packaging files stay invisible to
`just check-remote` until the operator stages them. Agents do not stage
those files.

### HTTP/2 and HTTP/3 (other tree)

HTTP/2 and HTTP/3 are not served on the splora unix socket. They belong
only on the surmount-server HTTPS edge around that JSON-RPC. Wiring a
vhost to `/run/splora/http.sock` and `/run/splora/queue.sock` is a later
session on the surmount-server tree. Do not add nginx here.

### Agent-doable leftover

None in this wave. The CSV two-file queue, unix-socket defaults, queue
XOR bind, queue mutex, popular-scripts timer and isolated output dir,
live hyper WS 101 fixture, MWCK subscribe fill from `Query` on first
track, REST 503/504 on daemon-proxy paths, named `rocksdbMoldLink`
check (unproven on the builder), README CLI versus module, no nginx,
and the surmount-server socket contract are already in this tree.

`nixosFiveInstances` now requires the default queue unit to carry
`--socket-file` `/run/splora/queue.sock` and to omit TCP `--bind`. That
matches `nix/module.nix`. Operator still owns `nix flake check`.

Named flake check `rocksdbMoldLink` is in the tree. The operator still
has to prove it with `just check-remote` after those flake files are
tracked. If that link fails, set `useSystemRocksdb = false` and keep
bundled 0.24.0.

### Sibling product paths still unfixed

Block notify covers heights after the previous tip only. It does not emit
a full reorg replay.

Removed mempool transactions that are gone from both mempool and chain
are notified as `{txid}` only.

hyper 0.14.18 has no HTTP/1 header-read timeout API. That API landed in
0.14.20. This tree did not bump hyper.

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
proof on this laptop. The handshake fixture, MWCK subscribe fill test,
REST 503/504 mapping, and `rocksdbMoldLink` check are in the tree; those
two recipes are the live proof of system RocksDB plus mold.

If public HTTP/2 and HTTP/3 are wanted, the next work is a surmount-server
vhost to the splora sockets on that other tree, not more nginx or h2 in
this crate. This session did not edit surmount-server.

After those gates, parked sibling gaps on this tree are block notify
tip-delta only, dropped txs as txid only, hyper 0.14.18 header-read
timeout, shared LRU, a live inotify watch test, and a `nixosTest` VM.
