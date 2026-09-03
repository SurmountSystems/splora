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
passed all five contracts (`ok. 5 passed`). That is not a crane build of
`packages.splora-http` and is not `NEEDED librocksdb`. Sharing one
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
points at that file. Do not list `FORK.md` as Open.

Named test `packaging_pins_rust_198_edition_2024_and_system_rocksdb`
matches `useSystemRocksdb = true`. That pin landed with this wave. It is
not leftover. It is not proof that the builder observed `NEEDED
librocksdb`.

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
compiles the in-tarball leveldb subtree via `cmake/leveldb.cmake`.
Wallet is off, so no BDB.

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

Nix flakes only see git-tracked files. `flake.nix` and `nix/module.nix`
are already tracked, so dirty edits eval. These paths are still
untracked (`git status --short` shows `??`; `git ls-files --others
--exclude-standard` names the files): `nix/bitcoind.nix`,
`nix/elementsd.nix`,
`nix/patches/bitcoin-31.1-with-system-libsecp256k1.patch`,
`src/bin/splora-http.rs`, and `src/http_front/mod.rs` (directory
`src/http_front/`). Staging only the three Nix daemon paths does not put
the HTTP front into the flake source copy. This re-run observed
`nix eval --impure --raw .#packages.x86_64-linux.bitcoind.drvPath` fail
with `error: path '/nix/store/...-source/nix/bitcoind.nix' does not
exist` (flake copy omits untracked files). Cheap check `nixosTenUnits`
still produced
`/nix/store/mwvbq6sig760wqccwy52q1ma5g4g2wcs-splora-nixos-ten-units.drv`
on `nix eval --impure --raw .#checks.x86_64-linux.nixosTenUnits.drvPath`
(dummy `pkgs.hello`, no Core compile). Named assert
`mutinynetStockCoreArgv` inside `nixosTenUnits` is still green on that
eval. Packages `bitcoind` / `elementsd` and `packages.splora-http` stay
invisible to a flake copy until the operator stages all of those
untracked paths (HTTP front plus the two Nix daemon files and the secp
patch). Agents do not stage them.

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
503/504 on daemon-proxy paths, README CLI versus module, no nginx, the
`splora-http` TLS front, [FORK.md](FORK.md) section 5, and the appliance
units are already in this tree.

`nixosFiveInstances` still requires the default queue unit to carry
`--socket-file` `/run/splora/queue.sock` and to omit TCP `--bind`. That
matches `nix/module.nix`. `nixosRemoteJsonrpcImport` requires one
instance with `jsonrpcImport = true`, `daemonRpcAddr = "10.0.0.1:8332"`,
`cookieFile = "/run/bitcoind/.cookie"`, `daemonDir = null`, and
`startLocalDaemon = false`. ExecStart must contain `--jsonrpc-import`,
`--daemon-rpc-addr`, and `--cookie-file`. ReadOnlyPaths must not list
`/var/lib/bitcoind`. Cookie user/password pairs must not appear in argv
or the module source. Operator still owns `nix flake check`. The cheap
remote-JSON-RPC eval was observed green without a crane rebuild. That is
not crate-wide `just check-remote`.

Named flake check `rocksdbMoldLink` now inspects `NEEDED librocksdb`
because `useSystemRocksdb` is true. It does not require mold in
`.comment`. Mold stays off. clang or default ld. Never gcc `-fuse-ld=`
plus a mold store path. That ELF `NEEDED librocksdb` line is still
unproven until a builder runs that check. A laptop Wild-linked
`librocksdb.so.10` ELF is not a crane proof.

`nix/elementsd.nix` pins the unpacked GitHub archive hash
`07p0zknrz74jyvxm04pa20y35kdarp9y0f5k99xz72psx9achkxv` for
`ElementsProject/elements` tag `elements-23.3.3` (`nix-prefetch-url
--unpack`, 2026-09-02). That is not a compile proof.

secp256k1 and leveldb: Bitcoin Core 31.1 CMake still vendors the
leveldb subtree. `WITH_SYSTEM_SECP256K1` and `WITH_SYSTEM_LEVELDB` are
unused cache variables (no `option()` in v31.1). This package no longer
passes those names. `nix/bitcoind.nix` applies
`nix/patches/bitcoin-31.1-with-system-libsecp256k1.patch` and passes
`(lib.cmakeBool "WITH_SYSTEM_LIBSECP256K1" true)`, with `secp256k1`
still in `buildInputs`. ELF `NEEDED libsecp256k1` still requires a Core
compile. There is no honest CMake switch for `pkgs.leveldb` on Core
31.1. Do not invent one. Elements 23.3.3 is autotools and also vendors
secp256k1 and leveldb. System linking of secp256k1 is still unproven.
That outcome is recorded, not a silent claim of system-only linking.

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

This review pass dropped `-signetblocktime=30` from `bitcoind-mutinynet`
ExecStart. The unit still passes the unwrapped published challenge,
`-addnode=45.79.52.207:38333`, and `-dnsseed=0`. Wallet stays off.
Named flake assert `mutinynetStockCoreArgv` forbids `-signetblocktime`
on mutinynet and on the other daemons. That assert is green on
`nix eval --impure --raw .#checks.x86_64-linux.nixosTenUnits.drvPath`.
Mutinynet's 30-second interval is a miner/network property. Stock Core
31.1 has no `-signetblocktime`. This package does not produce a
30-second-block outcome via argv.

This review pass also wired Gentoo `WITH_SYSTEM_LIBSECP256K1` into
`nix/bitcoind.nix`: the derivation applies
`nix/patches/bitcoin-31.1-with-system-libsecp256k1.patch`, passes
`(lib.cmakeBool "WITH_SYSTEM_LIBSECP256K1" true)`, keeps `secp256k1` in
`buildInputs`, and dropped unused `WITH_SYSTEM_SECP256K1` /
`WITH_SYSTEM_LEVELDB` plus `leveldb` from `buildInputs`. Unused Core
31.1 names remain unused in upstream CMake. There is no honest CMake
switch for system leveldb. Do not copy C into git. ELF
`NEEDED libsecp256k1` still requires a Core compile. That link is
unproven.

This wave landed a product type fix in `src/http_front/mod.rs` so h3
0.0.8 can typecheck, then upgrade-order and `wait_for_socket` so the
five named contracts run. This re-run of
`cargo test --offline --locked --lib http_front -- --test-threads=1`
passed all five contracts (`ok. 5 passed; 0 failed`). `nixosTenUnits`
eval is still green. Compiling `packages.splora-http` on the builder is
still unproven. It is not how those five tests run. This re-run did not
readelf a built ELF; `NEEDED librocksdb` is still unproven.

The operator staging every still-untracked flake source is the unblock:
`src/bin/splora-http.rs`, `src/http_front/` (`src/http_front/mod.rs`),
`nix/bitcoind.nix`, `nix/elementsd.nix`, and
`nix/patches/bitcoin-31.1-with-system-libsecp256k1.patch`. Staging only
the three Nix files still omits the HTTP front from the flake source
copy. Agents do not stage.

Unproven builder proofs after that full stage: `just check-remote`
observing `NEEDED librocksdb` (clang or default ld, no gcc+mold),
compiling `packages.splora-http` (aws-lc-rs + cmake) once those HTTP
front sources are tracked, and a Core compile that shows ELF `NEEDED
libsecp256k1` after this wiring. String pins are not that link proof.
This file does not claim `NEEDED librocksdb` without readelf. Do not
invent a copy workaround.

Do not re-open clap 4, wallet-on, MWCK, nginx, or RocksDB LRU-share.
Sharing one RocksDB LRU across indexer processes is not Open. Do not
put `pkgs.nixosTest` on `just check-remote`. e2e/QEMU stays off that
gate.
