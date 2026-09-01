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
   (accessed: 2026-08-28).
4. Crane builds copy that same `.cargo/config.toml` into the sandbox so the
   laptop and the remote builder use the same registry.

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
[`cargo-deny.toml`](../cargo-deny.toml) `[bans.skip]` with a one-line reason
each: bitcoin 0.32 versus nostr 0.45 (bech32, bitcoin_hashes,
hex-conservative, secp256k1); tungstenite 0.21 rand 0.8 versus nostr rand
0.10 (and getrandom / rand_core); hyper 0.14 `http` 0.2 versus tungstenite
`http` 1; bindgen/cc `shlex` 1 versus 2; hyper `socket2` 0.5 versus tokio
`socket2` 0.6; syn 2 versus syn 3; thiserror 1 versus 2. `multiple-versions`
stays warn so a new unskipped pair still shows. Do not bump `rocksdb` off
0.24. Do not raise CLI `--db-block-cache-mb` off 24.

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

The flake links one nixpkgs `rocksdb` for both `splora` and `splora-liquid`,
with mold as the linker (`useSystemRocksdb = true` in `flake.nix`):

- `ROCKSDB_LIB_DIR` / `ROCKSDB_INCLUDE_DIR` from `pkgs.rocksdb`
- `clang` and `cmake` as native inputs (bindgen and the crate build script)
- `liburing` in `buildInputs` because nixpkgs `librocksdb.so.10` `NEEDED`s it
- `RUSTFLAGS` `-C link-arg=-fuse-ld=` plus the store path of `pkgs.mold`

Pinned flake nixpkgs (`github:NixOS/nixpkgs/9fbb54b33e91ee4ca368e35a78e0613c720600b3`)
evaluates `pkgs.rocksdb.version` to **10.10.1** (accessed: 2026-08-31). The
shared object SONAME is `librocksdb.so.10`. That library’s own `RUNPATH`
already includes liburing.

The crate still vendors `librocksdb-sys` **0.17.3+10.4.2**. That gap is
closed by **bindgen against nixpkgs headers**, not by overlay-pinning
RocksDB 10.4.2 and not by bumping to `rocksdb` 0.25.0 (that crate vendors
**11.8.1**, farther from 10.10.1). When `ROCKSDB_LIB_DIR` and
`ROCKSDB_INCLUDE_DIR` are set, bindgen reads `pkgs.rocksdb` 10.10.1
headers and the linker uses `librocksdb.so`. The bundled 10.4.2 tree is
not the linked ABI.

Proven path versus fallback:

1. **Proven (crane / `just check-remote`):** named flake check
   **`checks.<system>.rocksdbMoldLink`** (`splora-nixpkgs-rocksdb-mold`)
   builds both crane packages and requires:
   - `readelf -d` `NEEDED` contains `librocksdb` on `splora` and `splora-liquid`
   - `readelf -p .comment` mentions `mold` on both binaries
2. **Not a proof:** laptop `cargo build` without those env vars. A local
   `target/debug/splora` on this machine (2026-08-31) `NEEDED`s libc and
   libstdc++ only, and `.comment` names the Wild linker, not mold. That
   ELF is bundled RocksDB. `nix eval` of `pkgs.rocksdb.version` is also
   not the link proof.
3. **Fallback:** if `rocksdbMoldLink` fails (liburing symbols, header
   skew versus `rust-rocksdb` 0.24, or mold), set `useSystemRocksdb =
   false` in `flake.nix`. The same check then writes that it is using
   bundled 0.24.0 and does not require `NEEDED librocksdb`. Leave
   `rocksdb = "0.24.0"` in `Cargo.toml`. Keep mold if it still links;
   drop only the RocksDB env vars.

Do not leave `useSystemRocksdb = true` if the check is rewritten to skip
`NEEDED`. Do not compile RocksDB once per binary when the system library
links. The operator recipe that runs the proof is `just check-remote`
(`nix flake check`). Agents do not run that recipe.

## Flake checks versus laptop checks

- Laptop (`just check-local`): `cargo fmt --all --check`, then
  `cargo clippy --all -- -D warnings`, then
  `cargo deny --offline --locked check --config cargo-deny.toml`, then
  `cargo audit`.
- Remote (`just check-remote`): `nix flake check`. That is nextest and the
  crane package builds. It is not deny or audit.
