# splora

Splora is a Bitcoin and Elements chain index with a REST API, a WebSocket, and Electrum over HTTP and/or a Unix socket. It comes from [romanz/electrs](https://github.com/romanz/electrs) and [Blockstream/electrs](https://github.com/Blockstream/electrs). This tree is not the mempool/mempool explorer. MWCK talks to `/api/v1/ws` on this binary. You do not need mempool/mempool. Fork differences from Mempool electrs live in [FORK.md](FORK.md).

## Multi-process network model

Run one process per network. Do not share one RocksDB across chains.

| Instance | `--network` | Notes |
| --- | --- | --- |
| mainnet | `mainnet` | Bitcoin. `--daemon-dir` is the bitcoind datadir root. |
| testnet3 | `testnet` | Bitcoin testnet3. The process appends `testnet3` under `--daemon-dir`. |
| testnet4 | `testnet4` | Bitcoin testnet4. |
| mutinynet | `signet` | Uses the bitcoind **signet** datadir. Pass `--magic a5df2dcb`. Default Bitcoin signet magic is `0x0A03CF40`. |
| liquid | `liquid` | Build with `--features liquid` (`splora-liquid`). Set `--asset-db-path`. |

The NixOS module is `services.splora` in [`nix/module.nix`](nix/module.nix). CLI defaults and production module argv are allowed to differ. The binary does not secretly use NixOS numbers. Production gets those numbers because the unit passes them.

Light mode stays off unless you pass `--lightmode`. Do not pass it for this deployment.

## CLI default vs NixOS module

| Setting | CLI default (no flags) | NixOS production module |
| --- | --- | --- |
| HTTP | TCP `127.0.0.1` plus the per-network port, or `--http-socket-file` if set | Unix socket `/run/splora/<instance>.http.sock`. TCP only if you set instance `httpAddr` and `httpSocketFile = null`. |
| Electrum | No TCP bind. Unix socket if `--rpc-socket-file`. Else `POST /electrum` only | Unix socket `/run/splora/<instance>.electrum.sock` plus `POST /electrum` on the HTTP socket |
| Queue | `splora-queue --bind` or `--socket-file`, not both | Own unit. Prefer `/run/splora/queue.sock` via `--socket-file`. TCP is `queueListen` with `queueSocketFile = null`. Not on indexer `ExecStart`. |
| `--db-block-cache-mb` | **24** per DB | **4096** per instance |
| `--db-parallelism` | **2** | **32** |
| `--lightmode` | off unless passed | off |
| Allowlist path | omitted means empty snapshot (nobody) | `--allow-npubs-file` one shared path |

Five instances times three DBs times 4096 MiB is about 60 GB of RocksDB cache on a large box. That is an operator choice for the module, not a change to the CLI default.

## Allowlist, queue, and import CLI

Authorization is two files. There is no status column.

1. **Pending queue** (`services.splora.queueFile`, default `/var/lib/splora/queue/import-queue`): one line `npub,email`. A comma inside the email is rejected (exactly two CSV fields). Same npub POST updates that line. This file is not JSON lines.
2. **Approved allowlist** (`services.splora.allowNpubsFile`, default `/var/lib/splora/allow-npubs`): one npub per line. Every indexer watches this file read-only. An empty file means nobody is authorized.

Always check the filesystem. Writes to the allowlist are the `splora-import` binary only:

```bash
splora-import approve --queue /var/lib/splora/queue/import-queue --allowlist /var/lib/splora/allow-npubs <npub>
splora-import reject --queue /var/lib/splora/queue/import-queue <npub>
splora-import remove --allowlist /var/lib/splora/allow-npubs <npub>
```

Approve deletes the queue line, upserts the allowlist, then fsyncs. Reject deletes the queue line and does not touch the allowlist.

The `splora-queue` binary accepts unauthenticated POST (`{npub,email}` JSON on the wire, CSV on disk). It does not write the allowlist. Do not put the queue on indexer HTTP. Do not pass an npub list on the indexer `ExecStart`. The indexer binary (`splora`) has no `queue` subcommand.

The queue file must live in its own directory so the queue unit `ReadWritePaths` is not the allowlist parent.

Authenticated indexer HTTP uses a **NIP-98** header. Queue POST stays unauthenticated.

Bitcoind cookies stay in a cookie file (`--cookie-file` or the network subdirectory under `--daemon-dir`). Do not put cookie contents in Nix.

## HTTP, Electrum, and MWCK

REST and WebSocket listen on `--http-addr` (CLI default `127.0.0.1`, ports 3000/3001/3003/3004 by network) or `--http-socket-file`. MWCK connects to `/api/v1/ws` on this process. Authenticated routes include `POST /electrum` and `POST /txs/package`.

`POST /electrum` is one JSON-RPC 2.0 body per request (object or batch array) with the same method names as socket Electrum. It is not a wrapped TCP framer. Socket Electrum is newline JSON-RPC on `--rpc-socket-file`. Omitting `--rpc-socket-file` does not bind Electrum TCP.

HTTP/2 and HTTP/3 are not served on the splora unix socket. They terminate on the Surmount Axum HTTPS edge in another tree. See [FORK.md](FORK.md) section 5. Splora sockets are local cleartext HTTP/1.1. QUIC cannot sit on a Unix domain socket.

## NixOS

The flake should export:

```nix
nixosModules.splora = import ./nix/module.nix;
```

Example (unix sockets, module cache numbers):

```nix
{
  services.splora.enable = true;
  services.splora.instances.mainnet.network = "mainnet";
  services.splora.instances.testnet3.network = "testnet3";
  services.splora.instances.testnet4.network = "testnet4";
  services.splora.instances.mutinynet.network = "mutinynet";
  services.splora.instances.liquid.network = "liquid";
}
```

Activation creates an empty allowlist and an empty queue if they are missing. Inspect `/var/lib/splora/allow-npubs` and `/var/lib/splora/queue/import-queue` on the host. The queue unit can write only the queue directory, not the allowlist parent.

`RuntimeDirectory` is `splora` (`/run/splora`). Default sockets:

| Socket | Path |
| --- | --- |
| Indexer HTTP (REST, `POST /electrum`, `/api/v1/ws`) | `/run/splora/<instance>.http.sock` |
| Electrum JSON-RPC | `/run/splora/<instance>.electrum.sock` |
| Queue HTTP | `/run/splora/queue.sock` |

TCP remains optional. Set instance `httpAddr` and `httpSocketFile = null` for indexer HTTP on a host port. Set `services.splora.queueListen` and `queueSocketFile = null` for queue TCP. The binary takes `--socket-file` or `--bind`, not both. There is no assertion that the queue must bind localhost. The queue is unauthenticated; prefer the unix socket.

Optional `services.splora.popularScripts.enable` starts a systemd timer that runs the existing `popular-scripts` binary (same flake app) against one instance DB.

### Surmount edge (no nginx)

There is no nginx in this tree. Do not enable an nginx vhost for splora. Public TLS, HTTP/2, HTTP/3, and cipher suites live on the first-party Axum edge in surmount-server (`surmount.sploraProxy`, `surmount.managementUi.http3Enable`). splora on the unix socket is local cleartext HTTP/1.1. QUIC cannot sit on a Unix domain socket. Fork law for that split is [FORK.md](FORK.md) section 5.

That other tree already maps Hosts to `/run/splora/<instance>.http.sock` (`SURMOUNT_SPLORA_*`, instance names `mainnet`, `testnet3`, `testnet4`, `mutinynet`, `liquid`) and refuses Electrum newline sockets on the HTTP proxy. This repo documents the socket contract. It does not implement the edge.

Point that edge at the socket paths above. Forward `Host` and `X-Forwarded-Proto` (usually `https`). splora still verifies NIP-98. The `u` tag is the public absolute URL. A terminator that omits `X-Forwarded-Proto` will 401 because this binary otherwise reconstructs `http://`. Do not verify NIP-98 twice at the proxy unless that sibling tree later opts in. WebSocket `/api/v1/ws` needs the same edge websocket proxy to the indexer HTTP socket.

## Build (without Nix)

Install Rust, a synced bitcoind (`txindex` is not required), `clang`, and `cmake`.

```bash
cargo run --release --bin splora -- -vvvv --daemon-dir ~/.bitcoin
# liquid:
cargo run --features liquid --release --bin splora -- -vvvv --network liquid --daemon-dir ~/.liquid
```

The indexer process source path is still `src/bin/electrs.rs`. The cargo bin name is `splora`. Nix installs `splora` / `splora-liquid`. Import is `splora-import`. Queue HTTP is `splora-queue`.

Index layout is in [doc/schema.md](doc/schema.md). Historical electrs usage notes are in [doc/usage.md](doc/usage.md).

## License

MIT
