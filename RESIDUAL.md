# Residual — splora production indexer

This file is the Open leftover list for the approved production indexer work.
Chat status is not this list. Dual honesty: finished slices are not Open;
sibling paths that are still unfixed stay here.

The last review pass (`/tmp/grok-1000/grok-review-splora-prod.md`) has **0
open issues**. Queue HTTP writes only `/var/lib/splora/queue`. The allowlist
stays `/var/lib/splora/allow-npubs`. Those review items are not leftover.

## Already in the tree (do not re-open)

The crate is named `splora`. Crane builds `splora` and `splora-liquid`.
`.cargo/config.toml` rewrites crates.io to the Menhera 7-day sparse index
for laptop `cargo`. Crane `src` omits that file and keeps `Cargo.lock`.
Crane `src` is `lib.cleanSource` plus `filterCargoSources`, plus
`flake.nix`, `nix/module.nix`, and `rust-toolchain`, so `src/config.rs`
tests can `include_str!` those files. It still omits `.cargo/config.toml`.
After vendor, `buildDepsOnly`, `buildPackage`, and nextest pass
`--offline --locked`, so Nix does not query `index.crates.menhera.org`.
That flake fix is in the tree. 2026-09-01 `just check-remote` after
the include_str src union passed (`all checks passed!`).
`cargo-deny.toml` allows the crates.io index URL that lockfiles still
record after that rewrite. Fetch still uses Menhera. Unknown git sources
are denied. Duplicate crate versions this tree cannot unify are skipped
in `[bans.skip]` with a reason each (bitcoin 0.32 versus nostr 0.45,
tungstenite rand 0.8 versus nostr rand 0.10, hyper 0.14 versus tungstenite
http 1, bindgen/cc shlex, hyper socket2 0.5 versus tokio 0.6, syn 2 versus
3, thiserror 1 versus 2). Those skips are intentional. They are not Open
unify-work. The validated table is in [doc/supply-chain.md](doc/supply-chain.md).
Do not bump bitcoin to 0.33-beta. Do not bump rocksdb off 0.24. Direct `base64`
is 0.22, `itertools` is 0.13, `socket2` is 0.5, `notify` is 8.2.0 (notify
7 reintroduced unmaintained `instant`). The mempool recent queue is a
capped `VecDeque`, not `bounded-vec-deque` (GPL identifier).
`just check-local` runs `cargo deny --offline --locked check`. Novel Surmount
files use the Unlicense. Inherited electrs stays MIT.

`rust-toolchain` is `1.98.0`. `Cargo.toml` is `edition = "2024"` and
`rust-version = "1.98"`. The flake uses `pkgs.rust-bin.stable."1.98.0".default`.
`hyper` is **0.14.32** (`Cargo.toml` floor 0.14.20, not hyper 1). Indexer REST and queue unix HTTP set `Server::http1_header_read_timeout` to 10 seconds. The clap 4 default for `--db-block-cache-mb` is 24. There is no `--allow-npubs` list flag. `--allow-npubs-file` stays.
`.cargo/config.toml` still rewrites crates.io to Menhera 7-day. The 2026-08-31
lock refresh pinned `nostr` **0.45.3**, `prometheus` 0.14.0 with default
features off, clap **4.6.6**, stderrlog **0.6.0**, and `serde-wincode`
**0.1.2** / `wincode` **0.6.1**. `wincode` **0.6.1** is a direct
`Cargo.toml` dependency because `src/util/bincode_util.rs` names
`wincode::config`. `idna` 1.0.3 and `idna_adapter` 1.1.0 stayed.
`rocksdb` stayed 0.24.0. NIP-98 uses `nostr::key::PublicKey` and
`nostr::event::Event` (0.45 no longer re-exports those at the crate
root), caps encoded header size before Base64, and signs test events
with `EventBuilder::finalize`. `cargo audit -n` reports 0
vulnerabilities and 0 warnings. Those rustsec rows are not leftover.

Authorization is two files. Pending queue is CSV `npub,email` with no status
column (`src/queue.rs`, `tests/queue_csv.rs`). Approved allowlist is one npub
per line. NIP-98 allowlist load, reload, and verify live in `src/auth.rs`.
Named unit test `watch_reloads_listed_npub_without_calling_reload` starts
`Allowlist::watch` on a temp file, writes a listed npub, and asserts
`contains()` without the test calling `reload()`. Empty allowlist stays
fail-closed. The wait is bounded at 3 seconds. That live inotify test is
in the tree.
That verifier caps the encoded `Authorization` payload before Base64
allocates, then caps decoded JSON at 64 KiB (`MAX_NIP98_AUTH_EVENT_BYTES`,
matching nostr 0.45.3 / RUSTSEC-2026-0229). Queue HTTP is the
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
`http1_header_read_timeout_closes_incomplete_request_line` is a live
hyper 0.14 fixture for the HTTP/1 header-read timeout.
`Allowlist::load` is already `Arc`; the fixture compares tungstenite http
1 status as `u16` against hyper 0.14 401/101.

`MwckHub` holds a `Query` and fills subscribe snapshots on first
`track-addresses` / `track-scriptpubkeys`. Empty arrays until a later
notify are not the only path. Named test
`handle_socket_track_addresses_fills_known_history` is green on that
contract (fixture history; production fills from `Query`). Block notify
emits confirmed txs for every new height after the previous tip (named
test `notify_block_emits_confirmed_from_two_new_heights`). A tip that
moves to a different hash at the same or a lower height replays orphaned
heights as `removed` then the new branch as `confirmed` via in-process
`Query` (named test `notify_new_tip_reorg_emits_removed_then_confirmed`).
Dropped mempool txs keep a full object in `removed` when the hub saw that
tx on add, on subscribe fill, or when Query still has the body. `MwckHub`
holds a capped txid-to-JSON map (`MAX_MEMPOOL_TX_BODIES`), uses cache then
Query then `{txid}`, and drops the cache entry after emit. Named tests
`prefer_full_removed_uses_available_body_not_txid_stub`,
`notify_mempool_removed_emits_full_object_when_available`, and
`notify_mempool_removed_emits_full_object_from_add_when_query_empty`.
This slice does not snapshot the whole mempool on every loop.

`ErrorKind::DaemonBusy` and `ErrorKind::DaemonUnavailable` exist.
REST daemon-proxy paths map occupancy to HTTP 503 and a missing or
timed-out daemon to HTTP 504. Named test
`daemon_proxy_failures_map_to_503_and_504` covers that mapping.
Electrum JSON-RPC stays JSON on the wire, not those HTTP statuses.

`nix/module.nix` ships five-instance NixOS options, isolated queue
`ReadWritePaths`, and an assertion that the queue directory is not the
allowlist directory. Queue listen is `--socket-file` XOR `--bind`. Default
listen is unix `/run/splora/queue.sock`. TCP needs `queueSocketFile =
null`. Instance `daemonDir` is `nullOr str`. When `cookieFile` and
`daemonRpcAddr` are set, `daemonDir` may be null: the unit omits
`--daemon-dir` and omits a missing `/var/lib/bitcoind` from
`ReadOnlyPaths`. Remote mode asserts cookie path plus rpc addr.
`jsonrpcImport` default false passes `--jsonrpc-import` when true.
`publicHealth` default false passes `--public-health`. Empty allowlist
still 401s address, tx, and mempool REST. `dbBlockCacheMb` module
default is 24 (same as CLI). REST does not require 4096. Optional
instance `memoryMax` sets systemd `MemoryMax` when set; public sample
leaves it unset. Cookie bytes never appear in the module.
`systemd.timers.splora-popular-scripts` and
`systemd.services.splora-popular-scripts` are in the module when
`services.splora.popularScripts.enable` is set. The oneshot writes
`/var/lib/splora/popular-scripts/popular-scripts.txt` and
`ReadWritePaths` is that directory only. Popular-scripts omits
`--daemon-dir` when that instance has no datadir. `flake.nix` exports
`nixosModules.splora`, `apps.popular-scripts`, overlays, the
`nixosFiveInstances` eval check, `nixosRemoteJsonrpcImport` (one
instance, remote JSON-RPC, no local datadir), `nixosQueueListenXorSocket`
(default socket plus `queueListen` must fail the module assertion), and
named check `rocksdbMoldLink` (`splora-nixpkgs-rocksdb-mold`). README
matches the production argv (CLI versus module), including remote-node
REST without Let's Encrypt, HTTP/3, hypervisor UDP 443, grok-oss, or
queue-only enable. Queue is not REST. There is no nginx in this tree.
Public TLS, HTTP/2, HTTP/3, and cipher suites belong on the first-party
Axum edge in surmount-server. splora on the unix socket is local
cleartext HTTP/1.1. QUIC cannot sit on a Unix domain socket. The README
and [FORK.md](FORK.md) section 5 state that socket contract. This
session did not edit surmount-server.

The indexer was not rewritten. `mempool/mempool` was not vendored.

Repo-root [FORK.md](FORK.md) names lineage, the Mempool schema lock, Blockstream ports that were copied without merging trees, Surmount-only modules, and the HTTP/2 HTTP/3 split (section 5). README points at that file. Do not list `FORK.md` as Open.

## Open

### Operator-owned gates

The operator still owns `just check-local` (fmt, clippy, deny, audit).
The crane Menhera DNS miss (`Could not resolve host:
index.crates.menhera.org` on `splora-deps-3.4.0-dev` after vendor) is
fixed in `flake.nix`. 2026-09-01 `just check-remote` run 3 after the
include_str src union exited 0: `all checks passed!` Nix omitted
aarch64-linux. This deny-hygiene wave ran `cargo check --lib` (exit 0),
`cargo check --lib --features liquid` (exit 0), named `--lib` tests
`new_index::mempool::tests` (exit 0), `cargo fmt --all --check` (exit
0), and `cargo deny --offline --locked check --config cargo-deny.toml`
(exit 0, no unmatched-source, license-not-encountered, or SPDX
parse-error). Named unit tests are not a substitute for `just
check-local`.

The flake sets `useSystemRocksdb = false`. Crane uses bundled `rocksdb`
0.24.0 (`librocksdb-sys` 0.17.3+10.4.2). 2026-09-01 `just check-remote`
did not query Menhera. Run 1 `splora-deps` failed because gcc rejected
`-fuse-ld=` plus the mold store path. Named check `rocksdbMoldLink`
is `splora-bundled-rocksdb` and does not run `readelf` (`NEEDED
librocksdb` / mold `.comment` unproven). That is a mold fail, not
builder DNS. Mold does not still link, so `RUSTFLAGS` mold and the
`mold` native input are dropped. How the fallback works is in
`doc/supply-chain.md`.
The `include_str!` nextest compile miss is fixed in `flake.nix`.
`packaging_pins_rust_198_edition_2024_and_system_rocksdb` now asserts
`useSystemRocksdb = false` and the bundled 0.24.0 stub string.

Nix flakes only see git-tracked files. `flake.nix`, `flake.lock`,
`nix/module.nix`, and other new packaging files stay invisible to
`just check-remote` until the operator stages them. Agents do not stage
those files.

### HTTP/2 and HTTP/3 (other tree)

HTTP/2 and HTTP/3 are not served on the splora unix socket. They
terminate on first-party Axum in surmount-server: TCP ALPN `h2` plus
HTTP/1.1, and UDP QUIC ALPN `h3` (`sploraProxy`, `http3Enable`). This
crate's unix sockets stay local cleartext HTTP/1.1. QUIC cannot sit on a
Unix domain socket. On surmount-1, `surmount.sploraProxy` is already
enabled. `http3Enable` stays true. Module `httpSocketFile` already
defaults to `/run/splora/${name}.http.sock`; do not change that default
to TCP. The edge must proxy HTTP only to that path, not to
`/run/splora/${name}.electrum.sock`. This session did not edit
surmount-server. Do not add nginx here. README and [FORK.md](FORK.md)
section 5 state that contract.

### Agent-doable leftover

The 2026-08-31 Menhera lock refresh, nostr 0.45.3, prometheus 0.14.0
without protobuf, clap 4.6.6, stderrlog 0.6.0, serde-wincode 0.1.2 with
direct wincode 0.6.1 for the on-disk schema, NIP-98 header size cap
before Base64, held `idna` 1.0.3 / `idna_adapter` 1.1.0 pins, `rocksdb`
0.24.0, `cargo audit -n` with 0 remaining RUSTSEC, and
`cargo deny --offline --locked check --config cargo-deny.toml` exit 0
are in the tree. They are not leftover. Direct crate hygiene in this
wave (base64 0.22, itertools 0.13, socket2 0.5, notify 8.2.0, mempool
capped VecDeque) is also in the tree. This wave ran the named `--lib`
tests (clap HTTP-wire help without an `--allow-npubs` list flag, HTTP/1
header-read timeout, historical 56-byte `bincode_settings`, oversized
NIP-98 caps, mempool recent-queue cap eviction). `just check-remote`
run 3 on 2026-09-01 ran crate-wide nextest and passed. The operator
still owns `just check-local`.

The CSV two-file queue, unix-socket defaults, queue XOR bind, queue
mutex, popular-scripts timer and isolated output dir, live hyper WS
101 fixture, MWCK subscribe fill from `Query` on first track, REST
503/504 on daemon-proxy paths, named `rocksdbMoldLink` check (fallback:
`useSystemRocksdb = false` after mold gcc reject on 2026-09-01), README CLI versus module, no nginx, and the
surmount-server socket contract are already in this tree.

`nixosFiveInstances` now requires the default queue unit to carry
`--socket-file` `/run/splora/queue.sock` and to omit TCP `--bind`. That
matches `nix/module.nix`. `nixosRemoteJsonrpcImport` requires one
instance with `jsonrpcImport = true`, `daemonRpcAddr = "10.0.0.1:8332"`,
`cookieFile = "/run/bitcoind/.cookie"`, and `daemonDir = null`. ExecStart
must contain `--jsonrpc-import`, `--daemon-rpc-addr`, and `--cookie-file`.
ReadOnlyPaths must not list `/var/lib/bitcoind`. Cookie `USER:PASSWORD`
bytes must not appear in argv or the module source. Operator still owns
`nix flake check`. The cheap remote-JSON-RPC eval was observed green
without a crane rebuild. That is not crate-wide `just check-remote`.

Named flake check `rocksdbMoldLink` now records bundled 0.24.0 because
`useSystemRocksdb` is false. Do not flip it back to true until crane
on the builder links `NEEDED librocksdb` with a linker gcc accepts.

### Sibling product paths still unfixed

MWCK reorg replay is in the tree. `notify_new_tip` is not a no-op when
the new height is lower or equal and the tip hash changed. Orphaned
heights emit `removed`, then the new branch emits `confirmed`.

Each RocksDB open gets its own LRU block cache. There is not one shared
cache across txstore, history, and cache. Sharing would need `schema.rs`.

`Allowlist::watch` has a live inotify unit test in `src/auth.rs`
(`watch_reloads_listed_npub_without_calling_reload`). That is not leftover.

`pkgs.nixosTest` was not added. The cheap `nixosFiveInstances` eval does
not boot five indexers and does not prove approve-then-HTTP-without-restart
on a VM.

## Highest value next

`just check-remote` on 2026-09-01 run 1 failed in `splora-deps` on mold,
not Menhera DNS. After `useSystemRocksdb = false` and dropping mold
`RUSTFLAGS`, run 2 built `splora-bundled-rocksdb` (text
`useSystemRocksdb is false; bundled rocksdb 0.24.0`) and copied
`splora-deps` from the builder. Run 2 then failed nextest (exit 101)
because crane `cleanCargoSource` omitted `flake.nix`, `nix/module.nix`,
and `rust-toolchain`. That filter hole is closed in `flake.nix`: crane
`src` unions those three paths onto `filterCargoSources` and still omits
`.cargo/config.toml`. `cargoExtraArgs` stays `--offline --locked`.
Run 3 (`just check-remote`) exited 0 with `all checks passed!` and
`checks.x86_64-linux.rocksdbMoldLink` still `splora-bundled-rocksdb`.
`useSystemRocksdb` stays false. A laptop Wild-linked
`librocksdb.so.10` ELF is not a crane proof.
Evicted mempool txs the hub saw on add keep a full object in `removed`.
That slice is in the tree, not leftover.

The clap 4 rewrite, the serde-wincode on-disk codec, and nostr 0.45.3
NIP-98 types are in the tree. Do not treat those rustsec rows as a
lock-only job.

If public HTTP/2 and HTTP/3 are wanted, enable the existing
surmount-server `sploraProxy` (and HTTPS `http3Enable`) on that other
tree. Do not add nginx or HTTP/2 in this crate. This session did not
edit surmount-server.

Parked sibling gaps on this tree (not the next proof) are a shared LRU
block cache and a `nixosTest` VM. Full MWCK reorg replay is in the tree.
Evicted mempool txs the hub saw on add keep a full object in `removed`.
The live inotify allowlist watch test is in the tree.
