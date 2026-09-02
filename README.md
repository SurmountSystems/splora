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
| HTTP | TCP `127.0.0.1` plus the per-network port, or `--http-socket-file` if set | Unix socket `/run/splora/<instance>.http.sock` (`httpSocketFile` already defaults to that path). Do not change that module default to TCP. TCP only if you set instance `httpAddr` and `httpSocketFile = null`. |
| Electrum | No TCP bind. Unix socket if `--rpc-socket-file`. Else `POST /electrum` only | Unix socket `/run/splora/<instance>.electrum.sock` (`electrumSocketFile`) plus `POST /electrum` on the HTTP socket. The edge must not point at the Electrum socket. |
| Queue | `splora-queue --bind` or `--socket-file`, not both | Own unit. Prefer `/run/splora/queue.sock` via `--socket-file`. TCP is `queueListen` with `queueSocketFile = null`. Not on indexer `ExecStart`. |
| `--db-block-cache-mb` | **24** per DB | **24** per instance (same as CLI). Set more on the instance if you want a larger cache. REST does not require 4096. |
| `--db-parallelism` | **2** | **32** |
| `--lightmode` | off unless passed | off |
| Allowlist path | omitted means empty snapshot (nobody) | `--allow-npubs-file` one shared path |

The module default for `--db-block-cache-mb` follows the CLI (24). A larger cache is an operator choice, not a REST requirement.

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

Bitcoind cookies stay in a cookie file (`--cookie-file` or the network subdirectory under `--daemon-dir`). Do not put cookie contents in Nix. Do not put `USER:PASSWORD` on argv.

Mempool REST on this binary talks to bitcoind JSON-RPC. It does not need Let's Encrypt vhosts, HTTP/3, hypervisor UDP 443, grok-oss, or queue-only enable. The import queue is not REST. Queue HTTP is a different unit (`splora-queue`).

A remote full node does not need a local bitcoind datadir. Pass a cookie **path**, `--daemon-rpc-addr`, and `--jsonrpc-import`. Omit `--daemon-dir`. The NixOS instance sets `cookieFile`, `daemonRpcAddr`, `jsonrpcImport = true`, and `daemonDir = null`. systemd then omits that missing datadir from `ReadOnlyPaths`. `--public-health` (instance `publicHealth`) opens tip health only; an empty allowlist still 401s address, tx, and mempool REST.

## HTTP, Electrum, and MWCK

REST and WebSocket listen on `--http-addr` (CLI default `127.0.0.1`, ports 3000/3001/3003/3004 by network) or `--http-socket-file`. MWCK connects to `/api/v1/ws` on this process. Authenticated routes include `POST /electrum` and `POST /txs/package`.

`POST /electrum` is one JSON-RPC 2.0 body per request (object or batch array) with the same method names as socket Electrum. It is not a wrapped TCP framer. Socket Electrum is newline JSON-RPC on `--rpc-socket-file`. Omitting `--rpc-socket-file` does not bind Electrum TCP.

HTTP/2 and HTTP/3 are not served on the splora unix socket. They terminate on first-party Axum in another tree: TCP ALPN `h2` plus HTTP/1.1, and UDP QUIC ALPN `h3`. See [FORK.md](FORK.md) section 5. Splora sockets are local cleartext HTTP/1.1. QUIC cannot sit on a Unix domain socket. Do not put QUIC on a unix socket.

## NixOS

The flake should export:

```nix
nixosModules.splora = import ./nix/module.nix;
```

Example (unix sockets; local datadir default `/var/lib/bitcoind`):

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

Example (one instance, remote bitcoind JSON-RPC, no local datadir):

```nix
{
  services.splora.enable = true;
  services.splora.instances.mainnet = {
    network = "mainnet";
    jsonrpcImport = true;
    daemonRpcAddr = "10.0.0.1:8332";
    cookieFile = "/run/bitcoind/.cookie";
    daemonDir = null;
  };
}
```

That unit passes `--jsonrpc-import`, `--daemon-rpc-addr`, and `--cookie-file`. It does not pass `--daemon-dir`. Cookie bytes never appear in the module. Optional instance `memoryMax` sets systemd `MemoryMax`; leave it unset unless you want a cap. One instance is enough for REST; five networks are not required.

Activation creates an empty allowlist and an empty queue if they are missing. Inspect `/var/lib/splora/allow-npubs` and `/var/lib/splora/queue/import-queue` on the host. The queue unit can write only the queue directory, not the allowlist parent.

`RuntimeDirectory` is `splora` (`/run/splora`). Default sockets:

| Socket | Path | NixOS option |
| --- | --- | --- |
| Indexer HTTP (REST, `POST /electrum`, `/api/v1/ws`) | `/run/splora/<instance>.http.sock` | `httpSocketFile` (already this path; do not change the default to TCP) |
| Electrum newline JSON-RPC | `/run/splora/<instance>.electrum.sock` | `electrumSocketFile` (not an HTTP edge target) |
| Queue HTTP | `/run/splora/queue.sock` | `queueSocketFile` |

The edge binds Splora HTTP only at `/run/splora/<instance>.http.sock`. Electrum newline stays on `/run/splora/<instance>.electrum.sock`. The edge must not point at the Electrum socket.

TCP remains optional. Set instance `httpAddr` and `httpSocketFile = null` for indexer HTTP on a host port. That is an operator override, not the production default. Set `services.splora.queueListen` and `queueSocketFile = null` for queue TCP. The binary takes `--socket-file` or `--bind`, not both. There is no assertion that the queue must bind localhost. The queue is unauthenticated; prefer the unix socket.

Optional `services.splora.popularScripts.enable` starts a systemd timer that runs the existing `popular-scripts` binary (same flake app) against one instance DB.

### Surmount edge (no nginx)

There is no nginx in this tree. Do not add nginx here. Public TLS, HTTP/2, and HTTP/3 terminate on first-party Axum in surmount-server: TCP ALPN `h2` plus HTTP/1.1, and UDP QUIC ALPN `h3`. The named knobs there are `surmount.sploraProxy` and `surmount.managementUi.http3Enable`. On surmount-1, `surmount.sploraProxy` is already enabled. `http3Enable` stays true. This repo documents the socket contract. It does not implement the edge.

splora on the unix socket is local cleartext HTTP/1.1. QUIC cannot sit on a Unix domain socket. Do not put QUIC on a unix socket. Fork law for that split is [FORK.md](FORK.md) section 5.

The edge must proxy HTTP only to `/run/splora/<instance>.http.sock`. Electrum newline stays on `/run/splora/<instance>.electrum.sock`. The edge must not point at the Electrum socket. That other tree already maps Hosts to the HTTP sockets (`SURMOUNT_SPLORA_*`, instance names `mainnet`, `testnet3`, `testnet4`, `mutinynet`, `liquid`) and refuses Electrum newline sockets on the HTTP proxy.

Point that edge at the HTTP socket paths above. Forward `Host` and `X-Forwarded-Proto` (usually `https`). splora still verifies NIP-98. The `u` tag is the public absolute URL. A terminator that omits `X-Forwarded-Proto` will 401 because this binary otherwise reconstructs `http://`. Do not verify NIP-98 twice at the proxy unless that sibling tree later opts in. WebSocket `/api/v1/ws` needs the same edge websocket proxy to the indexer HTTP socket.

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
