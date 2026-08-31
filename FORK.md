<!-- SPDX-License-Identifier: Unlicense -->

# Fork law for this tree

This file is Surmount documentation. It is released under the Unlicense (`UNLICENSE` in this repository). It names every way this checkout diverges from upstream Mempool electrs, and from Blockstream `new-index` where we copied behavior without merging those trees. It is not a product changelog and not an invitation to open pull requests against those projects.

Do not invent unshipped work here. If a behavior is not in this tree, it is not a fork difference yet.

## 1. Lineage

The default git branch of this product is `mempool`. That branch is `mempool/electrs`. `mempool/electrs` is Blockstream/electrs `new-index`. Blockstream `new-index` is `romanz/electrs`.

This repository is not a new fork of romanz/electrs. Do not treat it as one. Do not open pull requests against romanz, Blockstream, or Mempool for Surmount-only work.

`Cargo.toml` still records `homepage` and `repository` as `https://github.com/mempool/electrs`. The crate name is `splora`. The library crate is still named `electrs`. The indexer source path is still `src/bin/electrs.rs`. The cargo bin name is `splora`.

## 2. Schema we keep

RocksDB keys stay Mempool shapes. Confirmation rows are `C{txid}{confirmed-blockhash}`. Spend rows are paired `S{outpoint}{inpoint}` (`S{funding-txid:vout}{spending-txid:vin}`).

The full index layout is in [doc/schema.md](doc/schema.md). Do not copy that file here.

Do not revert confirmation keys to Blockstream height-only `C{txid}`. A height-only confirmation key cannot name which block confirmed the transaction when the same txid appears on more than one chain. This tree keeps the confirmed block hash in the key.

## 3. Behavior ported from Blockstream `new-index` without merging trees

We copied these behaviors into this tree. We did not merge Blockstream `new-index` as a git parent. Call that **ported behavior, trees not merged**.

- `--db-block-cache-mb` sets the RocksDB LRU block cache size in MiB for each of `txstore`, `history`, and `cache`. The clap default is 24. Production NixOS numbers are a different argv. See section 5.
- `--db-parallelism` sets RocksDB compaction and flush parallelism. The clap default is 2.
- `--enable-mining-rest` gates `GET /block-template`. When the flag is on, Bitcoin uses daemon `getblocktemplate` and Elements uses `getnewblockhex`. When the flag is off, that route is forbidden.
- `POST /txs/package` proxies daemon `submitpackage` (at most 25 hex transactions, optional `maxfeerate` and `maxburnamount` query params).
- Electrum method `blockchain.transaction.broadcast_package` is the same package submit on the Electrum JSON-RPC surface.
- REST body and time valves: request body collection times out after 30 seconds (`REQUEST_BODY_TIMEOUT`) and rejects bodies larger than 1,000,000 bytes (`MAX_BODY_SIZE`) with HTTP 413. A Blockstream-style HTTP/1 header-read timeout is **not** in this tree. hyper 0.14.18 in the lockfile has no `Server::http1_header_read_timeout` (that API landed in 0.14.20). This tree did not bump hyper. Do not document that header timeout as shipped.
- REST paths that proxy the daemon map occupancy to HTTP 503 (`ErrorKind::DaemonBusy`) and a missing or timed-out daemon to HTTP 504 (`ErrorKind::DaemonUnavailable`, and `ErrorKind::Connection` on that HTTP mapping). Electrum JSON-RPC stays JSON on the wire. Those HTTP statuses are REST only.

## 4. Surmount-only (not in mempool/electrs)

These modules and policies exist in this tree and are not upstream Mempool electrs.

- **NIP-98 and an allowlist file.** `src/auth.rs` loads a read-only npub list, reloads it, and verifies NIP-98 (`Authorization` header, kind 27235). Omitting `--allow-npubs-file` is an empty snapshot. Nobody is authorized, including localhost. The NixOS path is one shared `--allow-npubs-file`.
- **Two-file CSV queue.** Pending rows are `npub,email` with no status column (`src/queue.rs`, `tests/queue_csv.rs`). `splora-queue` is unauthenticated HTTP onto that file. `splora-import` is the only writer of the allowlist (`approve`, `reject`, `remove`). The indexer clap app has no `queue` subcommand. Queue disk writes serialize on a `Mutex`.
- **MWCK in-process `/api/v1/ws`.** `src/mwck.rs` speaks the Mempool Wallet Connector Kit JSON dialect on this binary. This tree is not `mempool/mempool` and does not vendor that explorer. Subscribe snapshots fill from `Query` on first `track-addresses` / `track-scriptpubkeys`.
- **`POST /electrum` JSON-RPC 2.0.** One JSON-RPC 2.0 body per request (object or batch array) with the same method names as socket Electrum. It is not a wrapped TCP framer.
- **No default Electrum TCP.** Omitting `--rpc-socket-file` does not bind Electrum TCP (`ElectrumListenPlan::HttpOnly`). Production Electrum is the unix socket and/or `POST /electrum`.
- **Unix sockets.** Indexer HTTP, Electrum JSON-RPC, and queue HTTP can listen on unix paths. NixOS defaults live under `/run/splora/`.
- **Flake and NixOS module.** `flake.nix` and `nix/module.nix` export `nixosModules.splora`, crane packages `splora` and `splora-liquid`, overlays, five-instance eval checks, queue listen XOR socket, optional popular-scripts timer, and named check `rocksdbMoldLink`. The flake sets `useSystemRocksdb = true`. That system RocksDB plus mold link is **not proven** on the builder until the operator runs `just check-remote` after the flake files are git-tracked. Do not claim it is proven. Fallback and the version gap are in [doc/supply-chain.md](doc/supply-chain.md).
- **Menhera and cargo-deny.** `.cargo/config.toml` rewrites crates.io to the Menhera 7-day sparse index. `cargo-deny.toml` denies unknown registries and git sources. How to add a crate is in [doc/supply-chain.md](doc/supply-chain.md). Do not duplicate that file here.
- **License split.** Inherited electrs stays MIT (`LICENSE`). Novel Surmount files use the Unlicense (`UNLICENSE` and `SPDX-License-Identifier: Unlicense` on those files). This document is Unlicense.

HTTP/2 and HTTP/3 are **not** served on the splora unix socket. They belong on the Surmount HTTPS edge in another tree (surmount-server). There is no nginx in this repository. splora on the unix socket is local cleartext.

## 5. CLI versus NixOS numbers

CLI defaults and production module argv are allowed to differ. The binary does not secretly use NixOS numbers. Production gets those numbers because the unit passes them.

The table lives in [README.md](README.md) under **CLI default vs NixOS module**. The numbers that matter for cache size are `--db-block-cache-mb` **24** on the CLI versus **4096** on the NixOS production module, and `--db-parallelism` **2** versus **32**. Do not maintain a second full copy of that table here.

## 6. What we did not change

- `src/new_index` key shapes stay Mempool (section 2). Do not “simplify” them.
- Liquid stays a cfg/feature split (`--features liquid`, cargo bin `splora-liquid` from the same indexer source with Elements types). It is not a second schema language.
- Two indexer binaries remain: `splora` (Bitcoin networks) and `splora-liquid` (Elements). Crane builds both. Do not collapse them into one process that shares one RocksDB across chains.
- Light mode stays off in production. The NixOS module does not pass `--lightmode`. Do not pass it for this deployment unless you intend the reduced index.
- There is no nginx in this tree. Do not add an nginx vhost here for TLS, HTTP/2, or HTTP/3.

The indexer was not rewritten. `mempool/mempool` was not vendored.

## 7. How to add a divergence

When you ship a new Surmount behavior, add a bullet to this file in the same wave as the code. Put Blockstream ports in section 3 and say the trees were not merged. Put Surmount-only modules in section 4. If the change is a schema lock, put it in section 2 and keep [doc/schema.md](doc/schema.md) true. If the change is a CLI versus module number, update the README table and leave this file as a pointer.

Do not list planned-but-unshipped work as a divergence. Do not claim HTTP/2 inside splora. Do not claim system RocksDB is proven because `useSystemRocksdb` is true in the flake.
