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

Nix flakes only see git-tracked files. `nix flake lock` and
`just check-remote` in this checkout fail with "Path 'flake.nix' is not
tracked" until the operator tracks `flake.nix` (and the other flake
inputs) in git. Agents do not stage.

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
[crates.io nostr](https://crates.io/crates/nostr) **0.45.3** published
2026-08-19. This lock pins nostr **0.45.3**. nostr **0.45.4** published
2026-08-30 is still inside the Menhera 7-day cooldown.

Published bitcoin **0.33.0-beta** does not drop the bech32 split with
nostr. Its crates.io dependencies (accessed: 2026-08-31) are `bech32`
`^0.11.0`, `bitcoin_hashes` `^0.20.0`, `hex-conservative` both `^0.3.0`
and `^1.0.0`, and `secp256k1` `^0.32.0-beta.2`. nostr 0.45.3 on this
lock uses bech32 **0.12.0**, bitcoin_hashes **1.2.0**, and secp256k1
**0.30.0**. Taking the beta would still leave bech32 0.11 versus 0.12
and would not land nostr’s secp 0.30 line.

### Cluster A: bitcoin 0.32.102 versus nostr 0.45.3

| Skip | Older (who, lock) | Newer (who, lock) | Honest unify |
|------|-------------------|-------------------|--------------|
| `bech32@0.11` | **0.11.1** via bitcoin 0.32.102 and elements 0.26.2 | **0.12.0** via nostr 0.45.3 | nostr on bech32 0.11, or a **future** bitcoin that takes 0.12. Not bitcoin 0.33-beta. |
| `bitcoin_hashes@0.14` | **0.14.101** via bitcoin 0.32.102, bip39 2.2.2, secp256k1 0.29.1, and secp256k1 0.30.0 | **1.2.0** via nostr 0.45.3 | A bitcoin (and secp256k1 0.30) that depend on hashes 1.x. Published 0.33-beta wants hashes **0.20**, not 1.2. Still no stable 0.33. |
| `hex-conservative@0.2` | **0.2.2** via bitcoin 0.32.102 and hashes 0.14.101 | **1.2.0** via hashes 1.2.0 | Follows hashes. |
| `secp256k1@0.29` | **0.29.1** via bitcoin 0.32.102 and secp256k1-zkp 0.11.0 | **0.30.0** via nostr 0.45.3 | A bitcoin major using secp ≥0.30. 0.33-beta wants secp **0.32-beta**, still not 0.30. |

[bech32](https://crates.io/crates/bech32),
[bitcoin_hashes](https://crates.io/crates/bitcoin_hashes),
[hex-conservative](https://crates.io/crates/hex-conservative),
[secp256k1](https://crates.io/crates/secp256k1) (accessed: 2026-08-31).

### Cluster B: hyper 0.14.32 versus tokio-tungstenite 0.21

[crates.io hyper](https://crates.io/crates/hyper) current default is
**1.11.1** (2026-08-28). This lock stays on hyper **0.14.32** (last 0.14,
2024-12-16) so REST keeps `Server::http1_header_read_timeout`.
tokio-tungstenite is **0.21.0** and tungstenite is **0.21.0**.

| Skip | Older (who, lock) | Newer (who, lock) | Honest unify |
|------|-------------------|-------------------|--------------|
| `http@0.2` | **0.2.12** via hyper 0.14.32 and http-body 0.4.6 | **1.5.0** via tungstenite 0.21.0 | hyper **1.11.x** (after Menhera 7-day) plus tokio-tungstenite **0.30**. REST rewrite. Not a skip deletion. hyper 1.11.1 is 3 days old as of 2026-08-31. |
| `rand@0.8` | **0.8.7** via tungstenite 0.21.0, secp256k1 0.29.1, secp256k1 0.30.0, and secp256k1-zkp 0.11.0 | **0.10.2** via nostr 0.45.3 | One rand line across tungstenite, secp, and nostr, or the hyper-1 stack. |
| `getrandom@0.2` | **0.2.17** via rand_core 0.6.4 and redox_users 0.4.6 | **0.4.3** via rand 0.10.2, tempfile 3.27.0, and jobserver 0.1.35 | Follows rand. |
| `rand_core@0.6` | **0.6.4** via rand 0.8.7 and rand_chacha 0.3.1 | **0.10.1** via rand 0.10.2 | Follows rand. |
| `socket2@0.5` | **0.5.10** via hyper 0.14.32 and this crate’s direct `socket2` 0.5 | **0.6.5** via tokio 1.53.1 | hyper 1. Direct socket2 stays 0.5 until that stack moves. |
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

## Pins after the 2026-08-31 lock refresh

Blanket `cargo update` used the Menhera 7-day index. `idna` stayed **1.0.3**
and `idna_adapter` stayed **1.1.0**. `rocksdb` stayed **0.24.0** /
`librocksdb-sys` **0.17.3+10.4.2**. The CLI `--db-block-cache-mb` default is
still 24.

hyper stayed on **0.14.32** (Cargo.toml floor **0.14.20**, not hyper 1).
Indexer REST (TCP and unix) and queue unix HTTP call
`Server::http1_header_read_timeout` with a **10 second** HTTP/1 header-read
timeout (`HTTP1_HEADER_READ_TIMEOUT`). That method landed in hyper 0.14.20.
Queue TCP still uses tiny_http and does not set it. This crate does not
enable hyper `http2` and does not terminate HTTP/3 on unix sockets.

0.14.32 is past the patched versions for these hyper rustsec rows (none are
open on this lock):

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

- `nostr` **0.45.3** (caret from 0.45.1). Drops wasm32 `instant`
  ([RUSTSEC-2024-0384](https://rustsec.org/advisories/RUSTSEC-2024-0384),
  accessed: 2026-08-31). Still carries the NIP-98 / NIP-44 patches from
  [RUSTSEC-2026-0216](https://rustsec.org/advisories/RUSTSEC-2026-0216)
  through [RUSTSEC-2026-0230](https://rustsec.org/advisories/RUSTSEC-2026-0230)
  (accessed: 2026-08-31). 0.45.0 is yanked. 0.45.4 published 2026-08-30 is
  still inside the 7-day cooldown, so this lock does not pick it. Features
  are `std`, `os-rng`, and `nip98`.
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
  accessed: 2026-08-31). Do not re-add clap 2.

`bitcoin` floated to **0.32.102**. That pulls `hex_lit` under SPDX **MITNFA**
(MIT plus no-false-attribs). That identifier is on the cargo-deny allow list.

## Remaining rustsec rows

`cargo audit -n` on this lock reports **0 vulnerabilities and 0 warnings**.
`cargo deny --offline --locked check --config cargo-deny.toml` exits 0 with
an empty advisory ignore list. Duplicate crate versions that cannot be
unified are skipped with reasons in `cargo-deny.toml`. They are not rustsec.

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
is rustc 1.98 (edition 2024); ICU4X is not required for that pin.

## RocksDB and mold

The flake **does not** currently link nixpkgs `rocksdb` or mold.

`just check-remote` on 2026-09-01 (`nix flake check` to
`ssh-ng://nixbuilder@23.182.128.234`) evaluated
`checks.x86_64-linux.rocksdbMoldLink` to
`/nix/store/jnc9iq7rlbc8c8wp7v6wwyn0dnqbv36g-splora-nixpkgs-rocksdb-mold.drv`
then failed in `splora-deps-3.4.0-dev` (exit 101) before that check ran.
`cargo check --release --offline --locked --all-targets` did **not** query
[index.crates.menhera.org](https://index.crates.menhera.org/7d/)
(accessed: 2026-09-01). The builder log has no `Could not resolve host`.
gcc rejected the mold flag:

`gcc: error: unrecognized command-line option '-fuse-ld=/nix/store/.../mold-unwrapped-wrapper-2.42.0/bin/mold'`

That is a mold link failure, not builder DNS. Per the fallback plan,
`useSystemRocksdb` is **false** in `flake.nix`. Crane keeps bundled
`rocksdb` **0.24.0** (`librocksdb-sys` 0.17.3+10.4.2). Mold does not
still link, so `RUSTFLAGS` `-fuse-ld=` and the `mold` native input are
dropped, not only the `ROCKSDB_*` env vars. Leave
`rocksdb = "0.24.0"` in `Cargo.toml`. Do not bump to 0.25.0.

Named check `rocksdbMoldLink` now writes
`useSystemRocksdb is false; bundled rocksdb 0.24.0` and does not require
`NEEDED librocksdb` or mold in `.comment`. The 2026-09-01 re-run built
that stub (`/nix/store/psjfa2wkfniyk5vifhj75frysyp1pmlh-splora-bundled-rocksdb`)
and finished `splora-deps` with `--offline --locked`. It then failed
`checks.x86_64-linux.nextest` because crane `cleanCargoSource` omitted
`flake.nix`, `nix/module.nix`, and `rust-toolchain` that `src/config.rs`
tests `include_str!`. Crane `src` now unions those three paths onto
`filterCargoSources` and still omits `.cargo/config.toml`. 2026-09-01
`just check-remote` after that union exited 0 (`all checks passed!`).
That nextest miss is not leftover. It is not a Menhera DNS miss.

Pinned flake nixpkgs (`github:NixOS/nixpkgs/9fbb54b33e91ee4ca368e35a78e0613c720600b3`)
still evaluates `pkgs.rocksdb.version` to **10.10.1** (accessed: 2026-08-31).
That eval is not a link proof. A laptop `cargo` ELF is not a link proof.

Do not set `useSystemRocksdb = true` again until crane on the builder
links `NEEDED librocksdb` with a linker gcc (or rustc) actually accepts.
Do not treat builder DNS as the first fix for this fail.

## Flake checks versus laptop checks

- Laptop (`just check-local`): `cargo fmt --all --check`, then
  `cargo clippy --all -- -D warnings`, then
  `cargo deny --offline --locked check --config cargo-deny.toml`, then
  `cargo audit`.
- Remote (`just check-remote`): `nix flake check`. That is nextest and the
  crane package builds. It is not deny or audit. Crane vendors from
  `Cargo.lock` and does not query Menhera.
