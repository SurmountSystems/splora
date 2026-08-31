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

- Unknown registries are denied. Allowed registries are the Menhera 7-day
  sparse index and the crates.io index URL that lockfiles still record after
  the rewrite.
- Unknown git sources are denied. `allow-git` is empty. The old
  `[patch.crates-io.electrum-client]` git rev was only for optional
  `electrum-discovery`. Default and liquid production packages do not enable
  that feature, so the patch is gone. If a git crate returns, pin `rev` in
  `Cargo.toml` and `rev` plus `narHash` in the flake, then list that URL under
  `allow-git` with a reason.
- AGPL and SSPL are not on the license allow list. Those licenses would infect
  the binary. GPL-2.0 and GPL-3.0 are omitted for the same reason. Inherited
  electrs stays MIT. Novel Surmount files use the Unlicense (`UNLICENSE`).

`just check-local` runs `cargo deny check --config cargo-deny.toml` and
`cargo audit` on the laptop. They are not flake checks. cargo-deny 0.19 looks
for `deny.toml` by default, so the recipe passes `--config cargo-deny.toml`.

## Pins after the 2026-08-31 lock refresh

Blanket `cargo update` used the Menhera 7-day index. `idna` stayed **1.0.3**
and `idna_adapter` stayed **1.1.0**. `rocksdb` stayed **0.24.0** /
`librocksdb-sys` **0.17.3+10.4.2**. hyper stayed on **0.14.32** (not hyper 1).
The CLI `--db-block-cache-mb` default is still 24.

Direct crate bumps that closed rustsec rows:

- `nostr` **0.44.8** (Menhera `pubtime` 2026-08-05). Patches
  [RUSTSEC-2026-0216](https://rustsec.org/advisories/RUSTSEC-2026-0216)
  through [RUSTSEC-2026-0230](https://rustsec.org/advisories/RUSTSEC-2026-0230)
  (accessed: 2026-08-31). 0.45.0 on the index is yanked; 0.45.1 is a larger
  API move (bech32 0.12, secp256k1 0.30).
- `prometheus` **0.14.0** with **default features off**. Default `protobuf`
  pulled protobuf 2.28.0
  ([RUSTSEC-2024-0437](https://rustsec.org/advisories/RUSTSEC-2024-0437),
  accessed: 2026-08-31). This crate only uses `TextEncoder`. 0.14.0 is on
  Menhera (published 2025-03-27). Do not re-enable the `protobuf` feature.

`bitcoin` floated to **0.32.102**. That pulls `hex_lit` under SPDX **MITNFA**
(MIT plus no-false-attribs). That identifier is on the cargo-deny allow list.

## Remaining rustsec rows (not patched in this wave)

`cargo audit` exits 0 with these **warnings**. `cargo deny` ignores the
unmaintained rows that would fail the Linux graph. Do not treat a warning as
a patched crate.

| ID | Crate | Why it remains |
|----|-------|----------------|
| [RUSTSEC-2021-0139](https://rustsec.org/advisories/RUSTSEC-2021-0139) | `ansi_term` 0.12.1 via clap 2.34.0 | Unmaintained. clap 4 is a CLI rewrite. (accessed: 2026-08-31) |
| [RUSTSEC-2024-0375](https://rustsec.org/advisories/RUSTSEC-2024-0375) | `atty` 0.2.14 via clap 2 and stderrlog 0.5.4 | Unmaintained. Replacement is `std::io::IsTerminal` after those callers move. (accessed: 2026-08-31) |
| [RUSTSEC-2021-0145](https://rustsec.org/advisories/RUSTSEC-2021-0145) | `atty` 0.2.14 unsound on Windows | This crate's deny graph is Linux. cargo-audit still warns from the lockfile. (accessed: 2026-08-31) |
| [RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141) | `bincode` 1.3.3 | Unmaintained. On-disk indexer schema. Do not swap codecs in a lock-only wave. (accessed: 2026-08-31) |
| [RUSTSEC-2024-0384](https://rustsec.org/advisories/RUSTSEC-2024-0384) | `instant` 0.1.13 via nostr 0.44.8 | Unmaintained wasm32 Instant polyfill. Linux cargo-deny graph does not include it; cargo-audit scans the lockfile. (accessed: 2026-08-31) |

error-chain 0.12.4 was not in the 2026-08-31 audit output. Do not rewrite the
error stack unless a later audit fails closed on it.

## How to add a crate

1. Pick a version that is at least 7 days old on crates.io so the Menhera
   7-day index can serve it.
2. Add it to `Cargo.toml`.
3. Run `cargo update -p <crate>` (or `cargo update` when the resolver must
   retie several packages). That refreshes `Cargo.lock`.
4. Run `cargo deny check --config cargo-deny.toml` and `cargo audit`
   locally (`just check-local`).
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
  `cargo deny check --config cargo-deny.toml`, then `cargo audit`.
- Remote (`just check-remote`): `nix flake check`. That is nextest and the
  crane package builds. It is not deny or audit.
