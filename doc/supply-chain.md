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

`just check-local` runs `cargo deny` and `cargo audit` on the laptop. They are
not flake checks.

## How to add a crate

1. Pick a version that is at least 7 days old on crates.io so the Menhera
   7-day index can serve it.
2. Add it to `Cargo.toml`.
3. Run `cargo update -p <crate>` (or `cargo update` when the resolver must
   retie several packages). That refreshes `Cargo.lock`.
4. Run `cargo deny` and `cargo audit` locally (`just check-local`).
5. If deny reports a license or git source, fix the crate choice or document
   an explicit exception. Do not fetch crates.io directly to skip the Menhera
   wait.

Do not vendor crates into this tree as a long-term fix.

`url` 2.5 pulls `idna` 1.x. That crate’s default Unicode backend is
`idna_adapter` 1.2 plus ICU4X 2.3, which needs rustc 1.88. This tree pins
`idna = "=1.0.3"` and `idna_adapter = "=1.1.0"` (the unicode-rs backend)
so rust-toolchain 1.87 still builds. A blanket `cargo update` can float
`idna_adapter` back to 1.2. Re-apply
`cargo update -p idna_adapter --precise 1.1.0` unless you also bump the
toolchain.

## RocksDB and mold

The flake links one nixpkgs `rocksdb` for both `splora` and `splora-liquid`,
with mold as the linker (`useSystemRocksdb = true` in `flake.nix`):

- `ROCKSDB_LIB_DIR` / `ROCKSDB_INCLUDE_DIR` from `pkgs.rocksdb`
- `clang` and `cmake` as native inputs (bindgen and the crate build script)
- `RUSTFLAGS` `-C link-arg=-fuse-ld=` plus the store path of `pkgs.mold`

Pinned flake nixpkgs (`github:NixOS/nixpkgs/9fbb54b33e91ee4ca368e35a78e0613c720600b3`)
evaluates `pkgs.rocksdb.version` to **10.10.1** (accessed: 2026-08-29). The
crate still vendors `librocksdb-sys` **0.17.3+10.4.2**. System link uses
nixpkgs headers via bindgen, not the bundled 10.4.2 tree. That version gap
is why the named check must actually inspect the linked ELF.

Named flake path: **`checks.<system>.rocksdbMoldLink`**
(`splora-nixpkgs-rocksdb-mold`). It builds both crane packages and requires:

- `readelf -d` `NEEDED` contains `librocksdb` on `splora` and `splora-liquid`
- `readelf -p .comment` mentions `mold` on both binaries

The operator recipe that runs this is `just check-remote` (`nix flake check`).
Agents do not run that recipe. A laptop `nix eval` of `pkgs.rocksdb.version`
is not the link proof.

If that check fails (liburing symbols, header skew versus `rust-rocksdb` 0.24,
or mold), keep the crate’s bundled RocksDB:

1. In `flake.nix`, set `useSystemRocksdb = false`. The same check then writes
   that it is using bundled 0.24.0 and does not require `NEEDED librocksdb`.
2. Leave `rocksdb = "0.24.0"` in `Cargo.toml`.
3. Keep mold if it still links; drop only the RocksDB env vars.

Do not compile RocksDB once per binary when the system library links. Bundled
RocksDB is the fallback so Auth, MWCK, the flake packages, and the queue are
not blocked on this experiment.

## Flake checks versus laptop checks

- Laptop (`just check-local`): `cargo fmt --all --check`, then
  `cargo clippy --all -- -D warnings`, then `cargo deny`, then `cargo audit`.
- Remote (`just check-remote`): `nix flake check`. That is nextest and the
  crane package builds. It is not deny or audit.
