# Residual for the splora production indexer

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
the include_str src union passed (`all checks passed!`). 2026-09-09
`just check-remote` (`nix flake check`) also exited 0 in about 12
minutes 50 seconds on rustc **1.98.1**. `checks.x86_64-linux.nextest`
passed as part of that gate. 2026-09-19 `just check-remote` after the
`src/rest.rs` graceful fix exited 0 in 149 seconds (`all checks
passed!`). nextest was 94/94. Named HTTP contracts passed (10s
header-read timeout, WS 101, signet 307, daemon 503/504). The first
`just check-remote` that day exited 1 on compile E0277 at
`src/rest.rs`. That compile miss is closed by the rest.rs fix. It is
not Open. 2026-09-20 `just check-remote` (`nix flake check`) exited 0
in 2 minutes 43 seconds (`all checks passed!`). It built
`checks.x86_64-linux.splora`, `splora-liquid`, `splora-http`,
`nextest`, and `rocksdbMoldLink` on surmount-1. Nix omitted
aarch64-linux. That omit is not a fail. Flake check is not
`cargo audit`. The 2026-09-22 Menhera lock has rustls **0.23.45**, which closes [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285) (accessed: 2026-09-22). That advisory is not Open. There is no `test-remote` recipe. Nix omitted aarch64-linux.
That omit is not a fail. `bitcoind` and
`elementsd` evaluated. `packages.splora-http` built.
`rocksdbMoldLink` observed `NEEDED librocksdb`. Previously untracked
flake sources (`nix/bitcoind.nix`, `nix/elementsd.nix`, the secp patch,
`src/bin/splora-http.rs`, `src/http_front/`) are tracked. They did not
block that run.
`cargo-deny.toml` allows the crates.io index URL that lockfiles still
record after that rewrite. Fetch still uses Menhera. Unknown git sources
are denied. Duplicate crate versions this tree cannot unify are skipped
in `[bans.skip]` with a reason each (bitcoin 0.32 versus nostr 0.45,
tungstenite rand 0.8 versus nostr rand 0.10, bindgen/cc shlex, direct
socket2 0.5 versus tokio 0.6, syn 2 versus
3, thiserror 1 versus 2, cpufeatures 0.2 versus 0.3). Those skips are intentional. They are not Open
unify-work. The validated table is in [doc/supply-chain.md](doc/supply-chain.md).
Do not bump bitcoin to 0.33-beta. Do not bump rocksdb off 0.24. Direct `base64`
is 0.22, `itertools` is 0.13, `socket2` is 0.5, `notify` is 8.2.0 (notify
7 reintroduced unmaintained `instant`). The mempool recent queue is a
capped `VecDeque`, not `bounded-vec-deque` (GPL identifier).
`just check-local` runs `cargo deny --offline --locked check`. Novel Surmount
files use the Unlicense. Inherited electrs stays MIT.

`rust-toolchain` is `1.98.1`. `Cargo.toml` is `edition = "2024"` and
`rust-version = "1.98"` (the 1.98 series MSRV, not a leftover 1.98.0
patch pin). The flake uses `pkgs.rust-bin.stable."1.98.1".default`.
The rust-overlay lock is `26a71e661c47bd21a05d06fec749f3f7c75e9d12`
after the 2026-09-19 `nix flake update`. rustc stays **1.98.1**.
`hyper` is **1.11.1** (h2 **0.4.19**). Indexer REST and queue unix HTTP set `http1::Builder::header_read_timeout` to 10 seconds with `TokioTimer`. The clap 4 default for `--db-block-cache-mb` is 24. There is no `--allow-npubs` list flag. `--allow-npubs-file` stays.
`.cargo/config.toml` still rewrites crates.io to Menhera 7-day. The 2026-08-31
lock refresh pinned `prometheus` 0.14.0 with default
features off, clap **4.6.6**, stderrlog **0.6.0**, and `serde-wincode`
**0.1.2** / `wincode` **0.6.1**. `wincode` **0.6.1** is a direct
`Cargo.toml` dependency because `src/util/bincode_util.rs` names
`wincode::config`. `idna` 1.0.3 and `idna_adapter` 1.1.0 stayed.
The 2026-09-09 Menhera lock refresh pinned `nostr` **0.45.4**. Direct
`rustls-pemfile` left the graph
([RUSTSEC-2025-0134](https://rustsec.org/advisories/RUSTSEC-2025-0134),
accessed: 2026-09-09). The 2026-09-19 Menhera `cargo update` pinned
`nostr` **0.45.5** and `rustls` **0.23.44**. The 2026-09-21 Menhera
`cargo update` (UTC) then locked `cc` **1.4.6**, `lru-slab` **0.1.3**,
and `tinyvec` **1.13.3**, and dropped `tinyvec_macros`. That day's
lock had `rustls` **0.23.44**. `rocksdb` stayed 0.24.0
(`librocksdb-sys` **0.17.3+10.4.2**). `bitcoin` stayed **0.32.102**.
That 2026-09-21 lock refresh is not Open leftover. 2026-09-21
`cargo fetch --locked` exited 0. 2026-09-20 `just check-local` ran
the full laptop recipe: `cargo fmt --all --check` exited 0 after
file-level rustfmt on `src/daemon.rs` and `src/new_index/schema.rs`;
`cargo clippy --all -- -D warnings` exited 0 in 4 minutes 18 seconds
with no warnings; `cargo deny --offline --locked check --config
cargo-deny.toml` exited 0 (advisories ok, bans ok, licenses ok,
sources ok). `cargo audit` that day exited 1 on
[RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285)
(accessed: 2026-09-22) while rustls was **0.23.44**. That audit result
is history. On 2026-09-22 the Menhera 7-day index
(`sparse+https://index.crates.menhera.org/7d/` via
`replace-with = "menhera-cooldown"` in `.cargo/config.toml`) served
rustls **0.23.45**. The lock update used that index. crates.io was
not used to skip the 7-day wait. `cargo audit` did print `Updating
crates.io index` as its own yanked-crate check. That line is not how
the lock was resolved. Cargo.toml already allowed rustls `0.23`. No
manifest edit. The locked checksum is
`0d41d731c7d2f962d1ccc364cec258de3c0e93b38c2fb3ba97ac74513048d634`.
The same wave also locked clap **4.6.7** (from 4.6.6), clap_builder
**4.6.7** (from 4.6.6), clap_lex **1.1.1** (from 1.1.0), quinn
**0.11.12** (from 0.11.11), and quinn-proto **0.11.18** (from
0.11.17). bitcoin stayed **0.32.102**. rocksdb stayed **0.24.0**.
`librocksdb-sys` stayed **0.17.3+10.4.2**. Do not take bitcoin 0.33.
Do not take rocksdb 0.25.
[RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285)
(accessed: 2026-09-22) is patched for rustls `>= 0.23.45`. `cargo
audit` on 2026-09-22 loaded advisory-db HEAD
`17af77682cecd2afa72b217ad7c6c30585d5003f`, scanned 312 crate
dependencies, and printed no vulnerability. Exit 0. No audit rows
remain. That advisory is not Open. Do not add an `[advisories]
ignore`. The ignore list stays empty. `cargo deny --offline --locked
check --config cargo-deny.toml` exited 0 (`advisories ok, bans ok,
licenses ok, sources ok`). Its offline advisory clone is still HEAD
`ba9db2a77a6a0fe93bc63a3d9b730e08b145aff5` (2026-08-31) and does not
contain RUSTSEC-2026-0285. Deny green is not the rustls proof. The
proof is `cargo audit`. The deny database was not updated. `just
check-local` (fmt, clippy `-D warnings`, deny, audit) exited 0 on
2026-09-22. No Rust sources changed. Clippy finished in 11.58s after
recompiling rustls 0.23.45 and quinn 0.11.12. fmt, clippy, deny, and
audit on that recipe are not Open. 2026-09-20 `just check-remote`
then exited 0 (`all checks passed!`). That earlier remote gate is not
Open. It did not run `cargo audit`, and it is not a run of the
2026-09-22 lock. On 2026-09-22, `just check-remote` (`nix flake check`)
ran on this lock (rustls **0.23.45**, bitcoin **0.32.102**, rocksdb
**0.24.0**). It exited 0. It started at 2026-09-22T08:04:46-06:00 and
ended at 2026-09-22T08:09:38-06:00. Wall time was 292 seconds. Nix
printed `all checks passed!` and warned that the check omitted
aarch64-linux. The tail showed derivation `splora-nixpkgs-rocksdb-mold`
building on `ssh-ng://nixbuilder@23.182.128.234`. Nix also warned that
the git tree is dirty and that app `apps.x86_64-linux.popular-scripts`
lacks attribute `meta`. Those warnings did not fail the check. No
product source files changed. That recipe is not `cargo audit` and not
`cargo deny`. It is not Open. Do not bump bitcoin to 0.33-beta. Do not bump rocksdb
off 0.24. Those floors are standing constraints, not unfinished 1.98.1
work. NIP-98 uses `nostr::key::PublicKey` and
`nostr::event::Event` (0.45 no longer re-exports those at the crate
root), caps encoded header size before Base64, and signs test events
with `EventBuilder::finalize`. Closed rustsec rows from the 2026-08-31
wave are not leftover. The hyper 1 port and
[RUSTSEC-2026-0258](https://rustsec.org/advisories/RUSTSEC-2026-0258)
(accessed: 2026-09-09) closed on the prior lock are not Open.
2026-09-09 `just check-remote` on rustc **1.98.1**, ELF
`NEEDED librocksdb`, and tracked flake sources are not Open.
2026-09-19 `just check-remote` after the rest.rs fix is not Open.
[RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285)
(accessed: 2026-09-22) is closed by the rustls **0.23.45** lock. It is
not Open.

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
matching nostr 0.45.4 / RUSTSEC-2026-0229). Queue HTTP is the
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
tiny hyper 1 HTTP/1.1 server fixture (not a full indexer) for empty-allowlist
401, listed-npub 101, and socket close after an allowlist reload drop.
`http1_header_read_timeout_closes_incomplete_request_line` is a live
hyper 1 fixture for the HTTP/1 header-read timeout.
`Allowlist::load` is already `Arc`; the fixture compares tungstenite http
1 status as `u16` against hyper 1 401/101.

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

`nix/module.nix` ships five-instance NixOS defaults, isolated queue
`ReadWritePaths`, and an assertion that the queue directory is not the
allowlist directory. Queue listen is `--socket-file` XOR `--bind`. Default
listen is unix `/run/splora/queue.sock`. TCP needs `queueSocketFile =
null`. Instance `daemonDir` is `nullOr str`. Appliance defaults are
`/var/lib/bitcoind/<net>` and `/var/lib/elementsd/liquid` with explicit
cookie paths. When `cookieFile` and `daemonRpcAddr` are set, `daemonDir`
may be null: the unit omits `--daemon-dir` and omits a missing
`/var/lib/bitcoind` from `ReadOnlyPaths`. `startLocalDaemon = false`
skips that chain's bitcoind or elementsd unit. Remote mode asserts cookie
path plus rpc addr. `jsonrpcImport` default true passes `--jsonrpc-import`.
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
`--daemon-dir` when that instance has no datadir.

The appliance units `bitcoind-mainnet`, `bitcoind-testnet3`,
`bitcoind-testnet4`, `bitcoind-mutinynet`, and `elementsd-liquid` are in
the module. Four bitcoind units share one `bitcoindPackage`. Wallet is
off. RPC is 127.0.0.1 only. P2P stays on. Cookie group-read for `splora`
is supplementary groups plus `-rpccookieperms=group` on Core 31 and
`-startupnotify=chmod g+r` on elementsd. `flake.nix` exports
`packages.bitcoind`, `packages.elementsd`, overlay replacements for
`pkgs.bitcoind` / `pkgs.elementsd`, `nixosModules.splora`,
`apps.popular-scripts`, the `nixosFiveInstances` eval check,
`nixosTenUnits` (ten units, same bitcoind store path, `-disablewallet`,
cookie paths), `nixosRemoteJsonrpcImport` (one instance, remote JSON-RPC,
no local datadir, no local daemon), `nixosQueueListenXorSocket`
(default socket plus `queueListen` must fail the module assertion), and
named check `rocksdbMoldLink` (`NEEDED librocksdb` when
`useSystemRocksdb` is true; mold is not required in `.comment`). README
matches the production argv (CLI versus module), including the appliance
story, remote-node REST without Let's Encrypt, hypervisor UDP 443,
grok-oss, or queue-only enable. Queue is not REST. There is no nginx in
this tree.

The HTTP front is in this tree. Cargo bin `splora-http` and
`src/http_front` terminate TLS 1.3, HTTP/2 (TCP ALPN `h2`), HTTP/1.1, and
HTTP/3 (UDP QUIC ALPN `h3`) on this host. Unix indexers stay local
cleartext HTTP/1.1 on `/run/splora/<instance>.http.sock`. QUIC is UDP.
QUIC is not a Unix domain socket. Public `/signet` is HTTP 307 to
`/mutinynet` and does not connect a backend (named test
`signet_api_tx_returns_307_to_mutinynet_and_does_not_connect_backend`).
The five named `http_front` contracts are unchanged
(`signet_api_tx_returns_307_to_mutinynet_and_does_not_connect_backend`,
`liquid_api_tx_hits_liquid_uds_with_path_tx`,
`mainnet_api_v1_ws_stays_on_mainnet_http_sock`,
`electrum_sock_is_never_a_proxy_target`,
`tls_1_2_handshake_is_refused`). This wave landed a product type fix in
`src/http_front/mod.rs` so h3 0.0.8 can typecheck, then
`hyper::upgrade::on` before `into_parts` and `wait_for_socket` using
`tokio::time::sleep` so unix tests do not stall the current-thread
runtime. Those tests were not rewritten. Named command
`cargo test --offline --locked --lib http_front -- --test-threads=1`
passed all five contracts (`ok. 5 passed`). 2026-09-09
`just check-remote` also built `packages.x86_64-linux.splora-http` and
`checks.x86_64-linux.rocksdbMoldLink` (`NEEDED librocksdb`). Sharing one
RocksDB LRU across indexer processes stays not Open.
[FORK.md](FORK.md) section 5 records that this crate terminates TLS,
HTTP/2, and HTTP/3 on `splora-http`. Do not say the edge lives only on
surmount-server. Do not claim HTTP/2 and HTTP/3 do not terminate in this
crate. surmount-server `sploraProxy` can still sit in front of this
process, or instead of it. That is not leftover for this tree. Sharing
one RocksDB LRU across indexer processes is not Open.

Repo-root [FORK.md](FORK.md) names lineage, the Mempool schema lock,
Blockstream ports that were copied without merging trees, Surmount-only
modules, the appliance, and the HTTP/2 HTTP/3 split (section 5). README
points at that file. Do not list `FORK.md` as Open. Section 9 points at
[doc/mempool-latent-prs.md](doc/mempool-latent-prs.md) for the eleven
open parent Mempool electrs pull requests plus branch
`mononaut/page-sizes`. Lineage stays Mempool. This tree did not fork
Blockstream. That decision log is in the tree. Do not re-open it.

`sendrawtransaction` JSON is `[hex, 0]` via `sendrawtransaction_params`.
Named test `sendrawtransaction_params_are_hex_and_numeric_zero_maxfeerate`
covers numeric zero maxfeerate. `request_proxied` stays. Package submit
maxfeerate is unchanged. That is Mempool #84 already here.

`apply_utxo_delta` caps history rows at `limit * 500`. Named test
`utxo_delta_history_rows_over_limit_times_500_is_too_many_txs` expects
`TooManyTxs(500)`. Shared `THREAD_POOL` was not retaken. Confirmation
keys stay `C{txid}{blockhash}`. There are no `R` keys. That is Mempool
#145 already here for the cap.

Shutdown order is rest-stop, RocksDB flush of `txstore_db`,
`history_db`, and `cache_db`, bounded Electrum join (5 seconds, poll
500 ms), then the leftover-thread watchdog. Named test
`shutdown_rest_stop_flush_three_cfs_then_bounded_electrum_join`. A hung
unix Electrum join cannot delay that flush. `process::exit(0)` on a
stuck join runs only after flush. This tree did not arm the watchdog
before rest-stop, because that is upstream #154 and would skip the
flush. `process::exit(0)` still skips remaining destructors. That skip
is documented. That is Mempool #154 already here with flush-then-bounded-join.

Unbounded query `max_txs` is `clamp_query_max_txs` with `min()` against
the existing `rest_default_*` and `rest_max_*` knobs. Named test
`huge_max_txs_is_clamped_to_rest_knobs_summary_default_stays_5000`.
Address summary default stays 5000. This tree did not take
`MAX_HISTORY_TXS = 100`. That clamp is already here. Skip stances for
the other parent pull requests live in the decision log. They are not
Open leftover of this wave. Do not duplicate that table here.

Liquid REST `sigops` on a peg-in transaction no longer returns early
for the whole transaction. `get_sigop_cost` still returns legacy cost
only for coinbase. A peg-in input has no sidechain prevout, so that
input skips P2SH and counts witness sigops against the claim script at
`pegin_witness[3]` when that stack has at least four items. Sibling
inputs still count P2SH and witness. Named tests live in
`src/util/transaction.rs` (`pegin_sigop_cost_tests`). Do not cherry-pick
Mempool #47. Skip evidence for that dirty 2023 patch is in
[doc/mempool-latent-prs.md](doc/mempool-latent-prs.md). This hole is
closed.

Named test `packaging_pins_rust_198_edition_2024_and_system_rocksdb`
matches `useSystemRocksdb = true`. That pin landed with this wave. It is
not leftover. 2026-09-09 `rocksdbMoldLink` observed `NEEDED librocksdb`
on the builder. That observation is not leftover.

Crane packages set `meta.mainProgram`. The bin is `splora` for both
`splora` and `splora-liquid` (Cargo `--bin splora`; the liquid pname is
not the binary name). The HTTP front bin is `splora-http`.
`lib.getExe` on those derivations does not warn. Appliance `bitcoind`
and `elementsd` also set `mainProgram`. Named flake check
`getExeMainProgram` calls `lib.getExe` on those five packages.
`nix/elementsd.nix` uses `stdenv.hostPlatform.isLinux`. This tree does
not read `stdenv.isLinux` or `stdenv.isDarwin`. That Nix eval-warning
slice is not Open.

The indexer was not rewritten. `mempool/mempool` was not vendored.

Bitcoin Core 31.1 is fetched from the bitcoincore.org tarball. This git
tree does not vendor that C. Do not copy C into git. CMake follows
nixpkgs bitcoin 31 flags plus Gentoo `WITH_SYSTEM_LIBSECP256K1`. Unused
Core 31.1 names `WITH_SYSTEM_SECP256K1` and `WITH_SYSTEM_LEVELDB` remain
unused in upstream CMake; this package no longer passes them.
`nix/bitcoind.nix` applies
`nix/patches/bitcoin-31.1-with-system-libsecp256k1.patch`, passes
`(lib.cmakeBool "WITH_SYSTEM_LIBSECP256K1" true)`, and keeps
`secp256k1` in `buildInputs`. `leveldb` is not in `buildInputs`. There
is no honest CMake switch for system leveldb on v31.1; Core 31.1 still
compiles the in-tarball leveldb subtree via `cmake/leveldb.cmake` into
static `libleveldb.a`. Do not invent a CMake switch for `pkgs.leveldb`.
Wallet is off, so no BDB.

Bitcoin Core 31.1 `packages.x86_64-linux.bitcoind` on surmount-1
produced `/nix/store/qwbm7zcnddd9kk7hlnqrjaqz0zknqda0-bitcoind-31.1`.
`readelf -d` on
`/nix/store/qwbm7zcnddd9kk7hlnqrjaqz0zknqda0-bitcoind-31.1/bin/bitcoind`
shows:

```
 0x0000000000000001 (NEEDED)             Shared library: [libsecp256k1.so.7]
```

That is shared nixpkgs secp256k1 0.8.0, not an in-tarball secp
compile-in. Drv eval alone is no longer the last word for Core secp.
There is no `NEEDED` libleveldb.

## Shared path crate called from both apps in tests

The shared path crate `splora-frontend-shared` exists at `frontend/shared`. `cargo test --manifest-path frontend/shared/Cargo.toml` exited 0, with 15 tests. Evidence report: `/home/hunter/.agents/reports/frontend-shared.md`.

The native dashboard now requests GET `/fee-estimates`, GET `/mempool`, GET `/blocks/tip/height`, and GET `/blocks/tip/hash` when it opens. The same open also requests GET `/blocks` and GET `/mempool/recent`. A block requests GET `/block/:hash/txids` and, when the start index is a multiple of 25, GET `/block/:hash/txs/:start_index`. A block still requests the unpaged GET `/block/:hash/txs`. `cargo test --manifest-path frontend/native/Cargo.toml` exited 0, with 34 library tests. The report is `/home/hunter/.agents/reports/native-remaining-paths.md`.

The web client already called those shared paths. Its last `cargo test` exited 0. The report is `/home/hunter/.agents/reports/web-shared-gaps.md`. That command was `cargo test --manifest-path frontend/web/Cargo.toml`, with 23 explorer tests and 1 missing signer test. Trunk is not on PATH. There was no Trunk build. The report is `/home/hunter/.agents/reports/trunk-build.md`. Nobody served the Leptos page. Tests feed JSON. Those tests do not mean the web screen is done.

There is no GPUI window pixel test. `cargo test` does not run `cx.open_window`. The report is `/home/hunter/.agents/reports/gpui-window-limit.md`. No GPUI window was opened. Tests feed JSON.

The shared path crate exists and both apps call it in tests for the data routes the inventory classes as ship in v1. Tests feed JSON. Nobody served the Leptos page. No GPUI window was opened. That proof is only the crate and the JSON tests. It is not a served Leptos page and it is not an opened GPUI window. Views are being checked. The explorer is not finished. The screens are not done. Serving the Leptos page in a browser is still not done.

## Open

### Explorer screens

The explorer is not finished. The screens are not done. Views are being checked. Trunk is not on PATH. No GPUI window was opened. Mining, prices, lightning, and acceleration stay out.

Explorer surfaces for mining, prices, lightning, acceleration, RBF, stale tips, statistics, named wallets, faucet, and liquid reserves are still out, because the indexer has no API for them. Layouts are not Mempool's. Those surfaces are leftovers, not work to start now. They are not the current job. The `--enable-mining-rest` switch already in this tree is not an explorer API for those surfaces. The mutinynet signet challenge hex recorded below is not an explorer faucet. There was no Trunk build. The report is `/home/hunter/.agents/reports/trunk-build.md`. There is no GPUI window pixel test. `cargo test` does not run `cx.open_window`. The report is `/home/hunter/.agents/reports/gpui-window-limit.md`. Nobody served the Leptos page. Tests feed JSON. Serving the Leptos page in a browser is still not done.

### Shared crate gaps

The native dashboard now requests GET `/fee-estimates`, GET `/mempool`, GET `/blocks/tip/height`, and GET `/blocks/tip/hash` when it opens. A block requests GET `/block/:hash/txids` and, when the start index is a multiple of 25, GET `/block/:hash/txs/:start_index`. The web client already called those shared paths. Those routes are not an open shared-crate gap.

### Operator-owned gates

`just check-local` (fmt, clippy, deny, audit) is the standing laptop
recipe. It is not leftover of the rustc 1.98.1 remote proof. 2026-09-22
`just check-local` exited 0. fmt, clippy `-D warnings`, deny, and
`cargo audit` each exited 0. Clippy finished in 11.58s after
recompiling rustls 0.23.45 and quinn 0.11.12. No Rust sources changed.
[RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285)
(accessed: 2026-09-22) is closed by the rustls **0.23.45** lock. It is
not Open. Do not list fmt, clippy, deny, or audit as Open. 2026-09-20
`just check-remote` (`nix flake check`) exited 0 (`all checks
passed!`). That earlier remote recipe is not leftover. It is not deny
or audit, and it is not a run of the 2026-09-22 lock. On 2026-09-22,
`just check-remote` (`nix flake check`) ran on this lock (rustls
**0.23.45**, bitcoin **0.32.102**, rocksdb **0.24.0**). It exited 0.
It started at 2026-09-22T08:04:46-06:00 and ended at
2026-09-22T08:09:38-06:00. Wall time was 292 seconds. Nix printed `all
checks passed!` and warned that the check omitted aarch64-linux. The
tail showed derivation `splora-nixpkgs-rocksdb-mold` building on
`ssh-ng://nixbuilder@23.182.128.234`. Nix also warned that the git tree
is dirty and that app `apps.x86_64-linux.popular-scripts` lacks
attribute `meta`. Those warnings did not fail the check. No product
source files changed. That recipe is not `cargo audit` and not
`cargo deny`. It is not leftover. The crane
Menhera DNS miss (`Could not resolve host: index.crates.menhera.org`
on `splora-deps-3.4.0-dev` after vendor) is fixed in `flake.nix`.

Agents do not stage. The staged `ref/mempool` gitlink is unsigned until the Operator runs `git commit -S`. Agents do not commit.

### Agent-doable leftover

[RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285)
(accessed: 2026-09-22) is not leftover. The 2026-09-22 Menhera lock
(`sparse+https://index.crates.menhera.org/7d/` via
`replace-with = "menhera-cooldown"`) has rustls **0.23.45**. That
version was on the Menhera 7-day index. The lock update used that
index. crates.io was not used to skip the 7-day wait. `cargo audit`
on 2026-09-22 loaded advisory-db HEAD
`17af77682cecd2afa72b217ad7c6c30585d5003f`, scanned 312 crate
dependencies, and printed no vulnerability. Exit 0. No audit rows
remain. The advisory is patched for rustls `>= 0.23.45`. It was closed
by the 0.23.45 lock, not by an ignore. Do not `[advisories] ignore`.
The ignore list stays empty. `cargo deny --offline --locked check
--config cargo-deny.toml` exited 0 (`advisories ok, bans ok, licenses
ok, sources ok`). Its offline advisory clone is still HEAD
`ba9db2a77a6a0fe93bc63a3d9b730e08b145aff5` (2026-08-31) and does not
contain RUSTSEC-2026-0285. Deny green is not the rustls proof. The
proof is `cargo audit`. The deny database was not updated. bitcoin
stays **0.32.102**. rocksdb stays **0.24.0** with `librocksdb-sys`
**0.17.3+10.4.2**. Do not take bitcoin 0.33. Do not take rocksdb 0.25.
nostr lock is **0.45.5**.
[RUSTSEC-2026-0258](https://rustsec.org/advisories/RUSTSEC-2026-0258)
(accessed: 2026-09-09) stays closed on the prior lock by hyper
**1.11.1** and h2 **0.4.19**. That closed row is not Open leftover of
the hyper 1 port. `just check-local` on 2026-09-22 exited 0. That
recipe is not Open.

`nix/elementsd.nix` pins the unpacked GitHub archive hash
`07p0zknrz74jyvxm04pa20y35kdarp9y0f5k99xz72psx9achkxv` for
`ElementsProject/elements` tag `elements-23.3.3` (`nix-prefetch-url
--unpack`, 2026-09-02). That is not a compile proof. 2026-09-09
evaluated `packages.x86_64-linux.elementsd` to `elementsd-23.3.3.drv`.
That eval is not ELF proof of system secp256k1.

Leveldb on Core 31.1 is still the in-tarball subtree compiled into
static `libleveldb.a`. `readelf -d` on the named Core 31.1 `bitcoind`
has no `NEEDED` for libleveldb. There is no honest CMake switch for
`pkgs.leveldb` on Core 31.1. Do not invent one. Core shared secp is
already proven: `NEEDED libsecp256k1.so.7` on
`/nix/store/qwbm7zcnddd9kk7hlnqrjaqz0zknqda0-bitcoind-31.1/bin/bitcoind`.
Elements 23.3.3 is autotools and still vendors secp256k1 and leveldb.
System linking of secp256k1 on Elements remains unproven. That
Elements outcome is recorded, not a silent claim of system-only
linking.

### Sibling product paths still unfixed

MWCK reorg replay is in the tree. `notify_new_tip` is not a no-op when
the new height is lower or equal and the tip hash changed. Orphaned
heights emit `removed`, then the new branch emits `confirmed`. Do not
re-open MWCK.

`Allowlist::watch` has a live inotify unit test in `src/auth.rs`
(`watch_reloads_listed_npub_without_calling_reload`). That is not leftover.

`pkgs.nixosTest` was not added and is not leftover. The cheap
`nixosFiveInstances` and `nixosTenUnits` evals do not boot QEMU and do
not prove approve-then-HTTP-without-restart on a VM. e2e/QEMU is not on
`just check-remote`. Do not put a NixOS VM test on that gate.

Public `/signet` 307 to `/mutinynet` is implemented in
`src/http_front` (named test
`signet_api_tx_returns_307_to_mutinynet_and_does_not_connect_backend`).
README and FORK also document that redirect for the public edge. That
redirect is not leftover.

Mutinynet `signetchallenge` is the published faucet hex
`512102f7561d208dd9ae99bf497273e16f389bdbd6c4742ddb8e6b216e64fa2928ad8f51ae`.
Indexer magic stays `a5df2dcb`. `bitcoind-mutinynet` ExecStart keeps
that unwrapped challenge, `-addnode=45.79.52.207:38333`, and
`-dnsseed=0`. It does not pass `-signetblocktime`. Named flake assert
`mutinynetStockCoreArgv` inside `nixosTenUnits` forbids
`-signetblocktime` on mutinynet and on the other daemons. Mutinynet's
30-second interval is a miner/network property. Stock Core 31.1 has no
`-signetblocktime`. This package does not produce a 30-second-block
outcome via argv. Do not wrap the challenge on stock Core: wrapping
would change P2P magic. Wallet stays off.

## Highest value next

The explorer is not finished. The screens are not done. Views are being checked. Trunk is not on PATH. No GPUI window was opened. Mining, prices, lightning, and acceleration stay out. The highest-value next step is not mining. It is running Trunk on a machine that has Trunk, then running `trunk build` in `frontend/web`. The GPUI pixel check needs a display. If both user interfaces change, that work still uses two app coordinators, one web and one native. Do not assign both apps to one coordinator. Layouts are not Mempool's. Explorer surfaces for mining, prices, lightning, acceleration, RBF, stale tips, statistics, named wallets, faucet, and liquid reserves stay out until the indexer has an API for them. They are not the current job.

The Menhera wait for rustls >=0.23.45 is done. The 2026-09-22 lock
has rustls **0.23.45**. The locked checksum is
`0d41d731c7d2f962d1ccc364cec258de3c0e93b38c2fb3ba97ac74513048d634`.
The same wave locked clap **4.6.7**, clap_builder **4.6.7**, clap_lex
**1.1.1**, quinn **0.11.12**, and quinn-proto **0.11.18**. Cargo.toml
already allowed rustls `0.23`. No manifest edit. `cargo audit` on
2026-09-22 exited 0. No audit rows remain.
[RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285)
(accessed: 2026-09-22) is closed by that lock, not by an ignore. Do
not `[advisories] ignore`. The ignore list stays empty. Do not fetch
crates.io to skip the Menhera wait. bitcoin is **0.32.102**. rocksdb
is **0.24.0** (`librocksdb-sys` **0.17.3+10.4.2**). Do not bump
bitcoin to 0.33-beta. Do not bump rocksdb off 0.24. `just check-local`
on 2026-09-22 exited 0 (fmt, clippy `-D warnings`, deny, and audit).
No Rust sources changed. That recipe is not the next proof. The
2026-09-20 `just check-remote` exit 0 is an earlier gate. It is not
leftover, and it is not a run of the 2026-09-22 lock. On 2026-09-22,
`just check-remote` (`nix flake check`) ran on this lock (rustls
**0.23.45**, bitcoin **0.32.102**, rocksdb **0.24.0**). It exited 0.
It started at 2026-09-22T08:04:46-06:00 and ended at
2026-09-22T08:09:38-06:00. Wall time was 292 seconds. Nix printed `all
checks passed!` and warned that the check omitted aarch64-linux. The
tail showed derivation `splora-nixpkgs-rocksdb-mold` building on
`ssh-ng://nixbuilder@23.182.128.234`. Nix also warned that the git tree
is dirty and that app `apps.x86_64-linux.popular-scripts` lacks
attribute `meta`. Those warnings did not fail the check. No product
source files changed. That recipe is not `cargo audit` and not
`cargo deny`. It exited 0 on 2026-09-22 and is not leftover. It is not
the next proof.

2026-09-19 `just check-remote` after the rest.rs fix is already green.
Command `just check-remote`, duration 149s, exit 0, `all checks
passed!`, nextest 94/94. Named HTTP contracts passed (10s header-read
timeout, WS 101, signet 307, daemon 503/504). That remote is not the
next proof. The first run that day exited 1 on compile E0277. That
miss is closed by the rest.rs fix. It is not Open. 2026-09-09
`just check-remote` already observed `NEEDED librocksdb` and built
`packages.splora-http` on rustc **1.98.1**. Those are not leftover.

Core 31.1 already has ELF `NEEDED libsecp256k1.so.7` on
`/nix/store/qwbm7zcnddd9kk7hlnqrjaqz0zknqda0-bitcoind-31.1/bin/bitcoind`.
That is not the next proof. Leveldb stays in-tarball static
`libleveldb.a`. There is no honest CMake switch for system leveldb.
Do not invent one. Do not copy C into git. Elements 23.3.3 still
vendors secp256k1 and leveldb. System linking on Elements remains
unproven.

Do not re-open clap 4, wallet-on, MWCK, nginx, or RocksDB LRU-share.
Sharing one RocksDB LRU across indexer processes is not Open. Do not
put `pkgs.nixosTest` on `just check-remote`. e2e/QEMU stays off that
gate. Do not re-open Mempool #84, the #145 history-row cap, the #154
rest-stop then flush then bounded Electrum join order, or the unbounded
`max_txs` clamp against existing REST knobs. Do not take
`MAX_HISTORY_TXS = 100`. Skip rows stay skip. Liquid peg-in REST
`sigops` is closed. The hung Electrum join before the watchdog is
closed.
