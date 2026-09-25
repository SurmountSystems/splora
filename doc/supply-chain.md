# Supply chain

This tree pins crates with `Cargo.lock` and fetches them through a committed
Cargo source rewrite, not through a laptop-only user config.

## How dependencies are pinned

1. Versions live in `Cargo.toml`.
2. Exact crates and checksums live in `Cargo.lock`. Updating that file is a
   project file edit. It is not git.
3. Workspace [`.cargo/config.toml`](../.cargo/config.toml) rewrites `crates-io`
   to the Menhera cooldown sparse index:
   [sparse+https://index.crates.menhera.org/7d/](https://index.crates.menhera.org/7d/)
   (accessed: 2026-08-28). Laptop `cargo` in this workspace uses that rewrite.
   Do not delete that file.
4. Crane `src` is `lib.cleanSource` plus `craneLib.filterCargoSources`,
   plus `flake.nix`, `nix/module.nix`, and `rust-toolchain` so
   `src/config.rs` tests can `include_str!` those files. It still omits
   `.cargo/config.toml`. `Cargo.lock` stays. After vendor,
   `buildDepsOnly`, `buildPackage`, and nextest pass `--offline --locked`.

Nix crane builds vendor crate sources from `Cargo.lock` and must not query
[index.crates.menhera.org](https://index.crates.menhera.org/7d/)
(accessed: 2026-08-28). The Menhera rewrite is for laptop `cargo` only. A
prior `splora-deps` failure on nixbuilder was `Could not resolve host:
index.crates.menhera.org` after vendor succeeded, because that workspace
config was inside crane `src`.

Nix flakes only see git-tracked files. Agents do not stage. 2026-09-09
`just check-remote` evaluated this flake. Untracked paths did not block
that run.

The committed window is **7 days**. A crate version must have been on crates.io
for at least that long before Menhera serves it. This host’s user cargo config
may still mention a 10-day rewrite. The repository file wins inside this
workspace.

## cargo-deny bans

[`cargo-deny.toml`](../cargo-deny.toml) is the license and source policy.

- Unknown registries are denied. The allow list is the crates.io index URL
  that lockfiles still record after the rewrite
  (`registry+https://github.com/rust-lang/crates.io-index`). Fetch still uses
  the Menhera 7-day sparse index via [`.cargo/config.toml`](../.cargo/config.toml).
  Do not list the Menhera sparse URL as an extra `allow-registry`; cargo-deny
  would report it as unused.
- Unknown git sources are denied. `allow-git` is empty. The old
  `[patch.crates-io.electrum-client]` git rev was only for optional
  `electrum-discovery`. Default and liquid production packages do not enable
  that feature, so the patch is gone. If a git crate returns, pin `rev` in
  `Cargo.toml` and `rev` plus `narHash` in the flake, then list that URL under
  `allow-git` with a reason.
- AGPL and SSPL are not on the license allow list. Those licenses would infect
  the binary. GPL-2.0 and GPL-3.0 are omitted for the same reason. Inherited
  electrs stays MIT. Novel Surmount files use the Unlicense (`UNLICENSE`).

`just check-local` runs `cargo deny --offline --locked check --config
cargo-deny.toml` and then `cargo audit` on the laptop. They are not flake checks.
cargo-deny 0.19 looks for `deny.toml` by default, so the recipe passes
`--config cargo-deny.toml`.

`--offline` keeps licenses, bans, and sources checks, and it uses the cached
RustSec advisory database plus the local registry index. It does not contact
crates.io (or time out on yanked HTTP). Yanked crates stay `yanked = "deny"`
when the local index already knows they are yanked. `cargo audit` still runs
next and is the live advisory and yanked pass. Do not fetch crates.io directly
to skip the Menhera wait.

Duplicate crate versions that this tree cannot unify are listed in
[`cargo-deny.toml`](../cargo-deny.toml) `[bans.skip]`. Skip means “do not
warn about this **older** copy.” Newer copies stay. The inventory below
matches `Cargo.lock` and crates.io metadata (accessed: 2026-08-31).
`multiple-versions` stays warn so a new unskipped pair still shows. Do
not bump `rocksdb` off 0.24. Do not raise CLI `--db-block-cache-mb` off
24. Do not take bitcoin `0.33.0-beta`. There is no stable bitcoin 0.33
on crates.io.

## cargo-deny `[bans.skip]` (validated 2026-08-31)

Lock versions in this section come from `Cargo.lock`. crates.io crate
pages and dependency APIs were checked on 2026-08-31.

[crates.io bitcoin](https://crates.io/crates/bitcoin) reports
`max_stable_version` **0.32.102**, `newest_version` **0.32.11**
(2026-07-22), and `max_version` **0.33.0-beta** (2026-02-23).
`0.33.0-beta.0` is yanked. This lock pins bitcoin **0.32.102**.
[crates.io nostr](https://crates.io/crates/nostr) **0.45.4** is on this
lock after the 2026-09-09 Menhera refresh. Do not bump bitcoin to
0.33-beta.

Published bitcoin **0.33.0-beta** does not drop the bech32 split with
nostr. Its crates.io dependencies (accessed: 2026-08-31) are `bech32`
`^0.11.0`, `bitcoin_hashes` `^0.20.0`, `hex-conservative` both `^0.3.0`
and `^1.0.0`, and `secp256k1` `^0.32.0-beta.2`. nostr 0.45.4 on this
lock uses bech32 **0.12.0**, bitcoin_hashes **1.2.0**, and secp256k1
**0.30.0**. Taking the beta would still leave bech32 0.11 versus 0.12
and would not land nostr’s secp 0.30 line.

### Cluster A: bitcoin 0.32.102 versus nostr 0.45.4

| Skip | Older (who, lock) | Newer (who, lock) | Honest unify |
|------|-------------------|-------------------|--------------|
| `bech32@0.11` | **0.11.1** via bitcoin 0.32.102 and elements 0.26.2 | **0.12.0** via nostr 0.45.4 | nostr on bech32 0.11, or a **future** bitcoin that takes 0.12. Not bitcoin 0.33-beta. |
| `bitcoin_hashes@0.14` | **0.14.101** via bitcoin 0.32.102, bip39 2.2.2, secp256k1 0.29.1, and secp256k1 0.30.0 | **1.2.0** via nostr 0.45.4 | A bitcoin (and secp256k1 0.30) that depend on hashes 1.x. Published 0.33-beta wants hashes **0.20**, not 1.2. Still no stable 0.33. |
| `hex-conservative@0.2` | **0.2.3** via bitcoin 0.32.102 and hashes 0.14.101 | **1.2.0** via hashes 1.2.0 | Follows hashes. |
| `secp256k1@0.29` | **0.29.1** via bitcoin 0.32.102 and secp256k1-zkp 0.11.0 | **0.30.0** via nostr 0.45.4 | A bitcoin major using secp ≥0.30. 0.33-beta wants secp **0.32-beta**, still not 0.30. |

[bech32](https://crates.io/crates/bech32),
[bitcoin_hashes](https://crates.io/crates/bitcoin_hashes),
[hex-conservative](https://crates.io/crates/hex-conservative),
[secp256k1](https://crates.io/crates/secp256k1) (accessed: 2026-08-31).

### Cluster B: hyper 1.11.1 versus tokio-tungstenite 0.21

[crates.io hyper](https://crates.io/crates/hyper) lock is **1.11.1**
(accessed: 2026-09-09). h2 is **0.4.19**, which is past the
[RUSTSEC-2026-0258](https://rustsec.org/advisories/RUSTSEC-2026-0258)
patch line **>= 0.4.16** (accessed: 2026-09-09). Indexer REST and queue
unix HTTP use `http1::Builder::header_read_timeout` with `TokioTimer`
for **10 seconds**. `splora-http` still speaks HTTP/2 after TLS ALPN
`h2`. tokio-tungstenite stays **0.21.0**. `http@0.2` left the graph.

| Skip | Older (who, lock) | Newer (who, lock) | Honest unify |
|------|-------------------|-------------------|--------------|
| `rand@0.8` | **0.8.7** via tungstenite 0.21.0, secp256k1 0.29.1, secp256k1 0.30.0, and secp256k1-zkp 0.11.0 | **0.10.2** via nostr 0.45.4 | One rand line across tungstenite, secp, and nostr. |
| `getrandom@0.2` | **0.2.17** via rand_core 0.6.4 and redox_users 0.4.6 | **0.4.3** via rand 0.10.2, tempfile 3.27.0, and jobserver 0.1.35 | Follows rand. |
| `rand_core@0.6` | **0.6.4** via rand 0.8.7 and rand_chacha 0.3.1 | **0.10.1** via rand 0.10.2 | Follows rand. |
| `socket2@0.5` | **0.5.10** via this crate’s direct `socket2` 0.5 | **0.6.5** via tokio 1.53.1 | Direct socket2 stays 0.5 until that stack moves. |
| `thiserror@1` and `thiserror-impl@1` | **1.0.69** via tungstenite 0.21.0, ppp 2.3.0, and redox_users 0.4.6 | **2.0.20** via wincode 0.6.1, serde-wincode 0.1.2, and prometheus 0.14.0 | those crates on thiserror 2. |

[http](https://crates.io/crates/http),
[rand](https://crates.io/crates/rand),
[getrandom](https://crates.io/crates/getrandom),
[rand_core](https://crates.io/crates/rand_core),
[socket2](https://crates.io/crates/socket2),
[thiserror](https://crates.io/crates/thiserror) (accessed: 2026-08-31).

### Cluster C: librocksdb-sys bindgen / cc

This lock has bindgen **0.72.1**, cc **1.4.4**, rocksdb **0.24.0**,
librocksdb-sys **0.17.3+10.4.2**. Unify here is a bindgen or proc-macro
bump. It is **not** a rocksdb 0.25 bump.

| Skip | Older (who, lock) | Newer (who, lock) | Honest unify |
|------|-------------------|-------------------|--------------|
| `shlex@1` | **1.3.0** via bindgen 0.72.1 | **2.0.1** via cc 1.4.4 | bindgen `shlex = "2"`. Build-only. |
| `syn@2` | **2.0.119** via bindgen 0.72.1 and thiserror-impl 1.0.69 (also pin-project-internal, wasm-bindgen-macro-support, windows-implement, windows-interface, zerocopy-derive) | **3.0.4** via serde_derive 1.0.229, thiserror-impl 2.0.20, tokio-macros 2.7.2, and futures-macro 0.3.34 | those proc macros on syn 3. [syn](https://crates.io/crates/syn) **3.0.4** published 2026-08-24. |

[shlex](https://crates.io/crates/shlex),
[syn](https://crates.io/crates/syn),
[serde_derive](https://crates.io/crates/serde_derive) (accessed:
2026-08-31).

**serde_derive syn requirement (confirmed):** this lock’s
`serde_derive` **1.0.229** depends on `syn 3.0.4`. The published
[serde_derive 1.0.229 Cargo.toml](https://docs.rs/crate/serde_derive/1.0.229/source/Cargo.toml)
sets `[dependencies.syn] version = "3"` (crates.io dependency API
`req` `^3`, accessed: 2026-08-31). serde_derive 1.0.229 itself
published 2026-07-18.

These skips are intentional. They are not leftover unify-work.

### Cluster D: cpufeatures 0.2 versus 0.3 (2026-09-09 lock refresh)

nostr **0.45.4** pulls `rand` 0.10 → `chacha20` 0.10.2 → `cpufeatures` **0.3.1**.
`sha2` (direct) and `sha1` via tungstenite stay on `cpufeatures` **0.2.17**.
Skip the older 0.2 copy. Unify is those hashes on 0.3, not a rocksdb or bitcoin bump.

| Skip | Older (who, lock) | Newer (who, lock) | Honest unify |
|------|-------------------|-------------------|--------------|
| `cpufeatures@0.2` | **0.2.17** via sha1 0.10.7 and sha2 0.10.9 | **0.3.1** via chacha20 0.10.2 (nostr rand 0.10 and quinn-proto) | sha1/sha2 on cpufeatures 0.3. |

[cpufeatures](https://crates.io/crates/cpufeatures) (accessed: 2026-09-09).

## Pins after the 2026-08-31 lock refresh

Blanket `cargo update` used the Menhera 7-day index. `idna` stayed **1.0.3**
and `idna_adapter` stayed **1.1.0**. `rocksdb` stayed **0.24.0** /
`librocksdb-sys` **0.17.3+10.4.2**. The CLI `--db-block-cache-mb` default is
still 24.

hyper is **1.11.1**. Indexer REST (TCP and unix) and queue unix HTTP call
`http1::Builder::header_read_timeout` with `TokioTimer` and a **10 second**
HTTP/1 header-read timeout (`HTTP1_HEADER_READ_TIMEOUT`). Queue TCP still
uses tiny_http and does not set it. Indexer listeners are HTTP/1.1 only.
`splora-http` enables hyper `http2` on TLS ALPN `h2` and HTTP/3 on UDP
QUIC. It does not terminate HTTP/2 or HTTP/3 on unix sockets.

hyper 1.11.1 is past the patched versions for these older hyper rustsec
rows (none are open on this lock):

- [RUSTSEC-2022-0022](https://rustsec.org/advisories/RUSTSEC-2022-0022)
  (unsound `mem::uninitialized` in the HTTP/1 parser; patched `>=0.14.12`,
  accessed: 2026-08-31)
- [RUSTSEC-2021-0079](https://rustsec.org/advisories/RUSTSEC-2021-0079)
  (Transfer-Encoding chunk size overflow; patched `>=0.14.10`, accessed:
  2026-08-31)
- [RUSTSEC-2021-0078](https://rustsec.org/advisories/RUSTSEC-2021-0078)
  (lenient Content-Length; patched `>=0.14.10`, accessed: 2026-08-31)
- [RUSTSEC-2021-0020](https://rustsec.org/advisories/RUSTSEC-2021-0020)
  (multiple Transfer-Encoding; patched `>=0.14.3`, accessed: 2026-08-31)

Direct crate bumps that closed rustsec rows:

- `nostr` **0.45.4** (caret from 0.45.1; lock was 0.45.3 until the
  2026-09-09 Menhera refresh). Drops wasm32 `instant`
  ([RUSTSEC-2024-0384](https://rustsec.org/advisories/RUSTSEC-2024-0384),
  accessed: 2026-08-31). Still carries the NIP-98 / NIP-44 patches from
  [RUSTSEC-2026-0216](https://rustsec.org/advisories/RUSTSEC-2026-0216)
  through [RUSTSEC-2026-0230](https://rustsec.org/advisories/RUSTSEC-2026-0230)
  (accessed: 2026-08-31). 0.45.0 is yanked. Features are `std`, `os-rng`,
  and `nip98`.
- `prometheus` **0.14.0** with **default features off**. Default `protobuf`
  pulled protobuf 2.28.0
  ([RUSTSEC-2024-0437](https://rustsec.org/advisories/RUSTSEC-2024-0437),
  accessed: 2026-08-31). This crate only uses `TextEncoder`. 0.14.0 is on
  Menhera (published 2025-03-27). Do not re-enable the `protobuf` feature.
- `serde-wincode` **0.1.2** with `wincode` **0.6.1** replaced bincode 1.3.3
  ([RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141),
  accessed: 2026-08-31). `src/util/bincode_util.rs` still emits the historical
  little/big-endian fixint layout. Do not rewrite `new_index` keys.
- clap **4.6.6** and stderrlog **0.6.0** dropped unmaintained `ansi_term` and
  `atty` ([RUSTSEC-2021-0139](https://rustsec.org/advisories/RUSTSEC-2021-0139),
  [RUSTSEC-2024-0375](https://rustsec.org/advisories/RUSTSEC-2024-0375),
  [RUSTSEC-2021-0145](https://rustsec.org/advisories/RUSTSEC-2021-0145),
  accessed: 2026-08-31). Do not re-add clap 2. The 2026-09-22 Menhera
  lock moved clap to **4.6.7**, clap_builder to **4.6.7**, and clap_lex
  to **1.1.1**. Those patches did not re-add clap 2.

`bitcoin` floated to **0.32.102**. That pulls `hex_lit` under SPDX **MITNFA**
(MIT plus no-false-attribs). That identifier is on the cargo-deny allow list.

## Rustsec status

The 2026-09-21 Menhera `cargo update` (UTC) locked `cc` **1.4.6**,
`lru-slab` **0.1.3**, and `tinyvec` **1.13.3**, and dropped
`tinyvec_macros`. `cargo fetch --locked` then exited 0. That day's
lock had rustls **0.23.44**. bitcoin stayed **0.32.102**. rocksdb
stayed **0.24.0** and `librocksdb-sys` stayed **0.17.3+10.4.2**. The
same day's live `cargo audit` (advisory-db HEAD
`d5c17953a895cf19e8d3ce66eaa42b6fcfe1fb16`, 2026-09-19, 1251
advisories) exited **1**. `cargo audit --json` listed one
vulnerability and an empty `warnings` map. `cargo audit -D warnings`
exited 1 on that same row and did not print yanked, unmaintained,
unsound, or notice warnings. That 2026-09-21 audit result is history.
It is not the current lock.

On 2026-09-22 a Menhera `cargo update` locked rustls **0.23.45**.
`.cargo/config.toml` sets `replace-with = "menhera-cooldown"` and the
Menhera index is `sparse+https://index.crates.menhera.org/7d/`. The
lock update used that index. crates.io was not used to skip the 7-day
wait. `cargo audit` did print `Updating crates.io index` as its own
yanked-crate check. That line is not how the lock was resolved.
Cargo.toml already allowed rustls `0.23`. No manifest edit. The locked
checksum is
`0d41d731c7d2f962d1ccc364cec258de3c0e93b38c2fb3ba97ac74513048d634`.
The same wave also moved clap **4.6.6** to **4.6.7**, clap_builder
**4.6.6** to **4.6.7**, clap_lex **1.1.0** to **1.1.1**, quinn
**0.11.11** to **0.11.12**, and quinn-proto **0.11.17** to **0.11.18**.
Those are patch updates. bitcoin stayed **0.32.102**. Do not take
bitcoin 0.33. rocksdb stayed **0.24.0**. `librocksdb-sys` stayed
**0.17.3+10.4.2**. Do not take rocksdb 0.25.

[RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285)
(accessed: 2026-09-22) covers TLS 1.3 handshake messages accepted
across encryption levels. It is patched for rustls `>= 0.23.45`. It
is not an open leftover. `cargo audit` on 2026-09-22 loaded
advisory-db HEAD `17af77682cecd2afa72b217ad7c6c30585d5003f`, scanned
312 crate dependencies, and printed no vulnerability. Exit 0. No audit
rows remain. The ignore list stays empty. Do not add an ignore for
that advisory. Do not fetch crates.io to skip the Menhera wait.

`cargo deny --offline --locked check --config cargo-deny.toml` exited
**0** (`advisories ok, bans ok, licenses ok, sources ok`). Its offline
advisory clone (`~/.cargo/advisory-dbs/advisory-db-3157b0e258782691`,
HEAD `ba9db2a77a6a0fe93bc63a3d9b730e08b145aff5`, 2026-08-31) does not
contain RUSTSEC-2026-0285. Deny green is not the rustls proof. The
proof is `cargo audit`. The deny database was not updated.

`just check-local` (fmt, clippy `-D warnings`, deny, audit) exited 0
on 2026-09-22. No Rust sources changed. Clippy finished in 11.58s
after recompiling rustls 0.23.45 and quinn 0.11.12.

On the prior lock after the hyper 1 port,
[RUSTSEC-2026-0258](https://rustsec.org/advisories/RUSTSEC-2026-0258)
(accessed: 2026-09-09) was closed by h2 **0.4.19** (patched line
`>= 0.4.16`). That row is not leftover of rustc 1.98.1 or of the hyper
1 port.

Yanked crates: cargo-audit JSON had an empty `warnings` map (no yanked,
unmaintained, unsound, or notice). cargo-deny `yanked = "deny"`
reported no yanked rows.

`rustls-pemfile` **2.2.0** ([RUSTSEC-2025-0134](https://rustsec.org/advisories/RUSTSEC-2025-0134),
accessed: 2026-09-09) left the graph. `splora-http` PEM load uses
`rustls::pki_types::pem::PemObject`.

Duplicate crate versions that cannot be unified are skipped with reasons
in `cargo-deny.toml`. They are not rustsec.

js-sys / wasm-bindgen remain in the lockfile as wasm target deps of
`iana-time-zone` (chrono via stderrlog timestamps). They are not
[RUSTSEC-2024-0384](https://rustsec.org/advisories/RUSTSEC-2024-0384)
(accessed: 2026-08-31). `instant` is gone.

error-chain 0.12.4 was not in the 2026-08-31 audit output. Do not rewrite the
error stack unless a later audit fails closed on it.

## How to add a crate

1. Pick a version that is at least 7 days old on crates.io so the Menhera
   7-day index can serve it.
2. Add it to `Cargo.toml`.
3. Run `cargo update -p <crate>` (or `cargo update` when the resolver must
   retie several packages). That refreshes `Cargo.lock`.
4. Run `cargo deny --offline --locked check --config cargo-deny.toml` and
   `cargo audit` locally (`just check-local`).
5. If deny reports a license or git source, fix the crate choice or document
   an explicit exception. Do not fetch crates.io directly to skip the Menhera
   wait.

Do not vendor crates into this tree as a long-term fix.

`url` 2.5 pulls `idna` 1.x. That crate’s default Unicode backend is
`idna_adapter` 1.2 plus ICU4X 2.3. This tree pins `idna = "=1.0.3"` and
`idna_adapter = "=1.1.0"` (the unicode-rs backend) so a blanket
`cargo update` does not float `idna_adapter` to 1.2. Re-apply
`cargo update -p idna_adapter --precise 1.1.0` if it drifts. The toolchain
is rustc 1.98.1 (edition 2024); ICU4X is not required for that pin.

## RocksDB and mold

The flake sets `useSystemRocksdb = true`. Named check `rocksdbMoldLink`
runs `readelf -d` and requires `NEEDED librocksdb` on `splora` and
`splora-liquid`. Mold stays off. clang or default ld. Never gcc
`-fuse-ld=` plus a mold store path.

2026-09-01 `just check-remote` failed when gcc was given a mold
`-fuse-ld=` path:

`gcc: error: unrecognized command-line option '-fuse-ld=/nix/store/.../mold-unwrapped-wrapper-2.42.0/bin/mold'`

That is history. Do not restore that gcc plus mold flag. Crane `src`
unions `flake.nix`, `nix/module.nix`, and `rust-toolchain` onto
`filterCargoSources` and still omits `.cargo/config.toml`. After vendor,
builds pass `--offline --locked`. That nextest include_str miss is not
leftover. It is not a Menhera DNS miss.

2026-09-09 `just check-remote` built `checks.x86_64-linux.rocksdbMoldLink`
(`splora-nixpkgs-rocksdb-mold.drv`) and exited 0. That is the builder
observation of `NEEDED librocksdb`. Leave `rocksdb = "0.24.0"` in
`Cargo.toml` (`librocksdb-sys` 0.17.3+10.4.2). Do not bump to 0.25.0.

2026-09-19 `nix flake update` moved flake.lock nixpkgs to
`20b1ddd1aa5ace70c9468305030aa4f9ef79671b`. This pass did not re-eval
`pkgs.rocksdb.version` on that pin. The prior pin
(`github:NixOS/nixpkgs/9fbb54b33e91ee4ca368e35a78e0613c720600b3`)
evaluated `pkgs.rocksdb.version` to **10.10.1** (accessed: 2026-08-31).
Do not invent a new rocksdb version. That eval is not a substitute for
the named check. A laptop `cargo` ELF is not a substitute either.

## Flake checks versus laptop checks

- Laptop (`just check-local`): `cargo fmt --all --check`, then
  `cargo clippy --all -- -D warnings`, then
  `cargo deny --offline --locked check --config cargo-deny.toml`, then
  `cargo audit`.
- Remote (`just check-remote`): `nix flake check`. That is nextest and the
  crane package builds. It is not deny or audit. Crane vendors from
  `Cargo.lock` and does not query Menhera. 2026-09-09 this recipe exited 0
  on rustc 1.98.1. 2026-09-19 `just check-remote` after the rest.rs
  graceful fix exited 0 in 149 seconds (`all checks passed!`). nextest
  was 94/94. Named HTTP contracts passed (10s header-read timeout, WS
  101, signet 307, daemon 503/504). The first run that day exited 1 on
  compile E0277. That compile miss is closed by the rest.rs fix. It is
  not leftover. Nix omitted aarch64-linux. That omit is not a fail.
  The 2026-09-21 laptop `cargo audit` exited 1 on
  [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285)
  (accessed: 2026-09-22) while rustls was **0.23.44**. That result is
  history. On 2026-09-22 the Menhera lock moved rustls to **0.23.45**
  and `cargo audit` exited 0. No audit rows remain. `just check-local`
  on 2026-09-22 exited 0. On 2026-09-22, `just check-remote` (`nix flake
  check`) ran on this lock (rustls **0.23.45**, bitcoin **0.32.102**,
  rocksdb **0.24.0**). It exited 0. It started at
  2026-09-22T08:04:46-06:00 and ended at 2026-09-22T08:09:38-06:00. Wall
  time was 292 seconds. Nix printed `all checks passed!` and warned
  that the check omitted aarch64-linux. The tail showed derivation
  `splora-nixpkgs-rocksdb-mold` building on
  `ssh-ng://nixbuilder@23.182.128.234`. Nix also warned that the git
  tree is dirty and that app `apps.x86_64-linux.popular-scripts` lacks
  attribute `meta`. Those warnings did not fail the check. No product
  source files changed. That recipe is not `cargo audit` and not
  `cargo deny`. It is not leftover. Deny `--offline` still uses
  advisory-db HEAD
  `ba9db2a77a6a0fe93bc63a3d9b730e08b145aff5` (2026-08-31), which does
  not contain RUSTSEC-2026-0285. Deny green is not the rustls proof.
  The proof is `cargo audit`. The deny database was not updated.
  [RUSTSEC-2026-0258](https://rustsec.org/advisories/RUSTSEC-2026-0258)
  (accessed: 2026-09-09) is closed on the prior lock. That closed row
  is not leftover. `just check-local` as a whole was not this
  session's 1.98.1 proof.
