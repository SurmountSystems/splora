<!-- SPDX-License-Identifier: Unlicense -->

# Open Mempool electrs pull requests versus this tree

This file is the in-repo decision log for **open** parent pull requests on [mempool/electrs](https://github.com/mempool/electrs) (accessed: 2026-09-20). Git parent stays that repository. This tree did not fork Blockstream. Do not merge `Blockstream/electrs`, `Blockstream/esplora`, or `romanz/electrs`. Lineage is [FORK.md](../FORK.md) section 1.

Live diffs were fetched on 2026-09-20 from `https://github.com/mempool/electrs/pull/NNN.diff`. The page-sizes branch is on that same parent as `mononaut/page-sizes` (not a public `mononaut/electrs` fork). Compare URL: `https://github.com/mempool/electrs/compare/mempool...mononaut/page-sizes.diff`. Commit `2dc61f50d41619ce99e4837451e7937836b2d0ad`.

Hunter's razor for these rows: keep HTTP/1 header-read timeout 10 seconds, WebSocket 101, public `/signet` 307, NIP-98, rocksdb 0.24, bitcoin 0.32, clap 4, hyper 1, crates.io only (no git crates). Confirmation keys stay `C{txid}{blockhash}`.

Stances are **take**, **already here**, or **skip**. Titles are not evidence. Constants below are quoted from the live patch or from this tree.

## Summary

| Item | Head SHA | Stance |
|------|----------|--------|
| [#84](https://github.com/mempool/electrs/pull/84) `sendrawtransaction` maxfeerate 0 | `62823e6b9b3d3029cc26032cf86501a078f07982` | **already here** |
| [#145](https://github.com/mempool/electrs/pull/145) history-row cap (title says 1000x) | `af82c5e2d75857b41d6e9fb5f037bd387e03ddc4` | **already here** (cap only; shared rayon pool already here) |
| [#154](https://github.com/mempool/electrs/pull/154) 5s shutdown `process::exit(0)` | `2962b4e3f865fc294238333401268758efd6d11b` | **already here** (flush then watchdog, not upstream arm-first) |
| [#157](https://github.com/mempool/electrs/pull/157) PROXY into `ConnectionStream` | `8c6f8552ceb248747d72c2e8c07af2561983373b` | **skip** |
| [#135](https://github.com/mempool/electrs/pull/135) `src/electrum/API.md` | `ba0c56b54703ec2e15ab3b81f2b690f89d5c870a` | **skip** (lying method list) |
| [#133](https://github.com/mempool/electrs/pull/133) `start` logging (draft) | `12538c2ec15927e2ab7a510ca4cebac859ca5456` | **skip** |
| [#119](https://github.com/mempool/electrs/pull/119) genesis `getrawtransaction` fallback | `523268e9a384c2f90455fdc383f8006cd7442f44` | **skip** |
| [#97](https://github.com/mempool/electrs/pull/97) `/utxo/recent` (draft) | `f730c43abc7088435456e9f049987450a151bc15` | **skip** |
| [#83](https://github.com/mempool/electrs/pull/83) REST TTL flags | `ec12a5ef669d5e81158cb4f8922709c234b70f8b` | **skip** |
| [#55](https://github.com/mempool/electrs/pull/55) log client IP | `0d12575047099ffd8f7518527ca09caa2d78c843` | **already here** (Electrum PROXY via crates.io `ppp`) / **skip** (REST XFF + git crate) |
| [#47](https://github.com/mempool/electrs/pull/47) Liquid pegin sigops (draft) | `cc75f9eed307ea7ee82f05dbeeefbbf710309387` | **skip** (this tree special-cases pegin; undercount remains) |
| `mononaut/page-sizes` (no PR) | `2dc61f50d41619ce99e4837451e7937836b2d0ad` | **skip** hard `MAX_HISTORY_TXS = 100`. **already here** for clamping unbounded `max_txs` to existing REST knobs. |

Take-now product work from these patches is already in this tree. Do not cherry-pick the skip rows. Remaining hole: Liquid REST `sigops` on pegin transactions still counts legacy only. Hung Electrum join still delays stop because this tree arms the watchdog after join and flush.

Diff SHA-256 of the fetched `.diff` files (content, not git object ids):

| File | SHA-256 |
|------|---------|
| `pull/84.diff` | `d816e710ce7b0d44fd60659f9082a295970934fe6ac8616219296acefb5181c9` |
| `pull/145.diff` | `adb1e4bd8ca6d89a06ab2d4336062005e4096e3ac148662cee19e979f06edcae` |
| `pull/154.diff` | `a5b2e7af27ea3b9f68e9d71822b42399d84d3d51b9e6998808831b145864fc1f` |
| `pull/157.diff` | `99dcbfc7c998e8575a45ddcb8c0338e70575275923cf1245b3ce42038811ffa6` |
| `pull/135.diff` | `ca3b3e351ffe21ab64d61a6e2f84478c5ce42d1bb824bac8f97c6477f51e86a1` |
| `pull/133.diff` | `c8594890e9f992d1c01767d7a18a8a03889d9e225cc62a92ac225e8aec7809c5` |
| `pull/119.diff` | `877d70f4120f660dcd043fa8193db0b6f11f8972c21d0c503cca341ff530e755` |
| `pull/97.diff` | `ed3997a636cc1b03ecd14f4f4a43e6d59c4064eef86fd65cf86909ab211911d5` |
| `pull/83.diff` | `1e7301ba3df97a2765ad43a3b9b62aeaeb74bd6b1506782f0758147d272c32f1` |
| `pull/55.diff` | `1dd8c1302e5a71c92504ccddf9db0b067b3788da6a6cbf0f95d49863daa4ea39` |
| `pull/47.diff` | `36ddbbfb1d6396324f29dbd49e7464220f20f6279e3fb0476e4621f83df1dcc6` |
| `compare/mempool...mononaut/page-sizes.diff` | `c32ac592c729533286aad9697c261a2752c7a0eb613ba19bd4d2e567397d657e` |

---

## #84 Remove maxfeerate safeguard from sendrawtransaction

Stance: **already here**.

Patch: one file, `src/daemon.rs` blob `707ab6e4629f`. Live hunk:

```
-        let txid = self.request("sendrawtransaction", json!([txhex]))?;
+        let txid = self.request("sendrawtransaction", json!([txhex, 0]))?;
```

Core documents `maxfeerate` as a numeric BTC/kvB argument whose default is 0.10. Numeric `0` disables the cap. See [sendrawtransaction (26.0.0)](https://bitcoincore.org/en/doc/26.0.0/rpc/rawtransactions/sendrawtransaction/) (accessed: 2026-09-20).

This tree: `sendrawtransaction_params` returns `json!([txhex, 0])`. `broadcast_raw` calls `request_proxied("sendrawtransaction", sendrawtransaction_params(txhex))`. Package `submit_package` still builds its own `maxfeerate` / `maxburnamount` array. Do not change package submit.

Named test: `sendrawtransaction_params_are_hex_and_numeric_zero_maxfeerate` in `src/daemon.rs`. It asserts JSON numeric `0`, not the string `"0"`.

---

## #145 Cap history to 1000x utxo limit

Stance: **already here** for the history-row cap. Shared rayon `THREAD_POOL` is **already here**. Do not retake the pool half.

Title is a lie until the patch is quoted. Live `src/new_index/schema.rs` blob `b40b31dfd39b`:

```
+        // If we need to iterate over 500 history entries to
+        // get one utxo then your address is too active and should be
+        // throttled.
+        // TODO: Think of better way to throttle.
+        let tx_history_limit = limit * 500;
...
+            if processed_items > tx_history_limit {
+                bail!(ErrorKind::TooManyTxs(tx_history_limit))
+            }
```

That is `limit * 500`, not 1000. Files also change `src/new_index/mod.rs` (global `THREAD_POOL`, `num_threads(0)`, name `electrs-worker-{}`) and `src/new_index/fetch.rs` (`Ok(super::THREAD_POOL.install(|| {`).

This tree: `apply_utxo_delta` uses `let tx_history_limit = limit * 500` then `TooManyTxs`. `src/new_index/mod.rs` already has `pub(crate) static THREAD_POOL`. `fetch.rs` and `lookup_txos` already call `super::THREAD_POOL.install`. Confirmation keys stay `TxConfKey { code: b'C', txid, blockhash }`. Cache prefixes in this tree are `A` (stats) and `U` (utxo), not a new `R`.

Named test: `utxo_delta_history_rows_over_limit_times_500_is_too_many_txs` in `src/new_index/schema.rs`. Constant `HISTORY_ROWS_PER_UTXO = 500`.

---

## #154 electrum: force exit if graceful shutdown exceeds 5 seconds

Stance: **already here**, with a required order change versus the live patch.

Patch: `src/bin/electrs.rs` only, blob `1bb97daa9e59`. Upstream replaces `shutdown-thread-checker` (counter 40 times 500 ms, no exit) with:

```
+            electrs::util::spawn_thread("shutdown-watchdog", || {
+                let timeout = Duration::from_secs(5);
+                let interval = Duration::from_millis(500);
...
+                error!("graceful shutdown timed out after 5 seconds, forcing exit");
+                process::exit(0);
             });
 
             rest_server.stop();
```

Upstream arms the watchdog **then** calls `rest_server.stop()`. Electrum join is still Drop of `electrum_server` after `break`. `process::exit(0)` skips every destructor, including RocksDB close and Electrum `RPC::drop` (`handle.join()`).

This tree: `SHUTDOWN_WATCHDOG_TIMEOUT` is 5 seconds, poll 500 ms, thread name `shutdown-watchdog`. `spawn_shutdown_watchdog` still calls `process::exit(0)` and the comments name the destructor skip. Stop order is rest-stop, Electrum join, `flush_index_store` (`txstore_db`, `history_db`, `cache_db`), **then** arm the watchdog. That is flush then watchdog, because a 5-second hard exit must not skip a flush this indexer relies on.

Named test: `shutdown_watchdog_armed_after_rest_stop_join_and_flush` in `src/bin/electrs.rs`.

Remaining hole: because join runs before the watchdog is armed, a hung Electrum `JoinHandle` still delays systemd stop and never reaches flush or `process::exit(0)`. Upstream would kill that hang at 5 seconds and skip flush. This tree chose flush over killing a join that has not finished.

---

## #157 refactor proxy header logic into ConnectionStream

Stance: **skip**.

Patch: `src/electrum/server.rs` only, blob `3d4848d69e1f`, +468 / -173. Live constants: `PROXY_HEADER_TIMEOUT: Duration = Duration::from_secs(5)`, `MAX_PROXY_HEADER_SIZE: usize = 4096`, `resolve_proxy` (non-blocking probe), `resolve_proxy_blocking`, leftover replay on `ConnectionStream` wrapping `ConnectionStreamInner::{Tcp, Unix}`.

This tree already has Electrum PROXY. `ConnectionStream` is `Tcp` or `Unix`. `read_proxy_headers` uses crates.io `ppp` 2.3.0 (`Cargo.toml`: `ppp = "2.3.0"`). `--electrum-haproxy-depth` default is `"0"`. Depth 0 ignores parsed addresses. Production Electrum is unix under `/run/splora/*.electrum.sock` and `POST /electrum`. `splora-http` does not speak PROXY to that socket. FORK.md forbids nginx.

Skip evidence: taking this rewrite would fight unix Electrum, leftover handling already present, idle timeout, and per-client limits in the same file. Named product tests for unix Electrum and depth 0 stay the contract. Do not port `resolve_proxy_blocking`.

---

## #135 docs: Add API docs for electrum API

Stance: **skip**. The new file is a lying method list for this product.

Patch: new `src/electrum/API.md`, blob `2e1dceb714ef`, +391. Example `server.version` response is `["mempool-electrs v3.3.0", "1.4"]`. Methods listed: `server.version`, `server.banner`, `server.donation_address`, `server.peers.subscribe`, `server.ping`, `server.features`, `server.add_peer`, `blockchain.block.header`, `blockchain.block.headers`, `blockchain.headers.subscribe`, `blockchain.estimatefee`, `blockchain.relayfee`, `blockchain.scripthash.get_balance`, `blockchain.scripthash.get_history`, `blockchain.scripthash.listunspent`, `blockchain.scripthash.subscribe`, `blockchain.scripthash.unsubscribe`, `blockchain.transaction.broadcast`, `blockchain.transaction.get` (verbose "will return an error"), `blockchain.transaction.get_merkle`, `blockchain.transaction.id_from_pos`, `mempool.get_fee_histogram`.

There is no `blockchain.transaction.broadcast_package`. There is no `POST /electrum`.

This tree: `src/electrum/server.rs` implements `blockchain_transaction_broadcast_package` and dispatches `"blockchain.transaction.broadcast_package"`. FORK.md section 3 and 4 name that method and `POST /electrum`. This tree has no `src/electrum/API.md`.

Skip evidence: shipping that file would document Mempool 3.3.0 and omit Surmount methods.

---

## #133 Make logging changes easier during start script (draft)

Stance: **skip**.

Patch files: `start` blob `9c4319898b5f` (`ELECTRS_LOG_VERBOSITY` default `-vv`, regex `^-v{1,4}$`), `Cargo.toml` blob `958b692a4b8b` (`panic = 'unwind'`), `src/electrum/server.rs` blob `2dfec0d5c2b3` (`catch_unwind(AssertUnwindSafe(|| { conn.run(); }))` then `std::panic::panic_any(err)`).

This tree: leftover Mempool `start` is not the NixOS unit. Production does not run `start`. `[profile.release] panic = 'abort'` in `Cargo.toml`. `catch_unwind` is useless with abort, and the patch re-panics anyway.

Skip evidence: NixOS has no `start`. Panic policy stays abort.

---

## #119 Fallback to getblock when getrawtransaction fails on Genesis TX

Stance: **skip**.

Patch: `src/daemon.rs` blob `2057fc247062`. On Bitcoin-only, if `getrawtransaction` error contains `"genesis block coinbase is not considered an ordinary transaction"`, and `verbose` is false, and `block.txdata.len() == 1`, and `prev_blockhash == BlockHash::default()`, serialize tx 0 from `getblock`.

This tree: `gettransaction_raw` is a straight `self.request("getrawtransaction", json!([txid.to_string(), verbose, blockhash]))`. Light mode is off in production. `nix/module.nix` comments `Do not pass --lightmode.` `add_transaction` writes `TxRow` unless `iconfig.light_mode`. Full index serves genesis from the txstore.

Skip evidence: light-mode-only hole. Light mode stays off. Dirty versus current proxied JSON-RPC in `daemon.rs`.

---

## #97 Add recent utxos rest API endpoint (draft)

Stance: **skip**.

Patch files: `src/config.rs` (`Arg::with_name("utxos_history_limit")`, `--utxos-history-limit`, default `"20000"`), `src/new_index/query.rs` (`recent_utxo`), `src/new_index/schema.rs` (`RecentUtxoCacheRow` with `code: b'R'`, key `[b"R", scripthash].concat()`), `src/rest.rs` (`GET /address/:a/utxo/recent` and scripthash). Author body (accessed: 2026-09-20): "Todo: test reorg/stale block behavior" unchecked. Reverse history scan. clap 2 `Arg::with_name` / `value_t_or_exit!`.

This tree: no `recent_utxo`. Cache prefixes stay `A` and `U`. clap 4 `Arg::new`. MWCK already tracks addresses over `/api/v1/ws` (`track-addresses`). Adding `R` keys is a new cache contract with untested reorgs. Not a confirmation-key revert, still skip.

Skip evidence: draft, unchecked reorg todo, new `R` prefix, clap 2, explorer-chart endpoint MWCK does not need.

---

## #83 Allow ttl short and ttl long for REST to be configurable

Stance: **skip**.

Patch files: `src/config.rs` (`--mempool-rest-ttl-short` default `"10"`, `--mempool-rest-ttl-long` default `"157784630"`, clap 2 `Arg::with_name` / `value_t_or_exit!`), `src/rest.rs` (replace `TTL_SHORT` / `TTL_LONG` at every `json_response`). Regtest always uses short: `if config.network_type.is_regtest() { config.mempool_rest_ttl_short }`.

This tree: `const TTL_LONG: u32 = 157_784_630;` and `const TTL_SHORT: u32 = 10;` in `src/rest.rs`. clap 4. hyper 1. NIP-98 and body valves live in the same file. `splora-http` is the public cache edge. Indexer Cache-Control is secondary.

Skip evidence: hardcoded TTL behind `splora-http` is enough. Do not rewrite `rest.rs` against clap 2 and a large TTL thread-through.

---

## #55 Feat: Log client IP during REST and Electrum requests

Stance: **already here** for Electrum PROXY. **skip** the REST X-Forwarded-For half and the git crate.

Patch files (9): `Cargo.toml` / `Cargo.lock` (`proxy-protocol` git `https://github.com/junderw/proxy-protocol` rev `5f5431ecdae75c7e8aba0f7aebcfc2e0102b70dc`; `rocksdb = "0.21.0"`; `panic = 'unwind'`), `src/bin/electrs.rs`, `src/config.rs` (`electrum_proxy_depth`, `rest_proxy_depth`, clap 2), `src/electrum/server.rs`, `src/new_index/fetch.rs`, `src/rest.rs` (`HeaderMap<HeaderValue>`, `get_client_ip` on `"X-Forwarded-For"`), `src/util/mod.rs`, `start`.

This tree: Electrum PROXY is crates.io `ppp` 2.3.0 and `--electrum-haproxy-depth` default 0. `cargo-deny.toml` has `unknown-git = "deny"` and `allow-git = []`. rocksdb is `0.24.0`. panic is abort. hyper 1, not old hyper `HeaderMap`. Queue HTTP already keys rate limits on the first `X-Forwarded-For` hop. `splora-http` inserts `X-Forwarded-Proto: https` when missing (`src/http_front/mod.rs`). REST indexer does not log XFF as #55 does.

Skip evidence: git `proxy-protocol` is denied. Do not take old hyper headers, clap 2 flags, `start` depth=1, or panic unwind. Electrum PROXY is already the crates.io path.

---

## #47 WIP: Liquid sigops (draft)

Stance: **skip** cherry-pick.

Patch: `src/util/transaction.rs` blob `8a4fa18f31d0`. For `input.is_pegin`, if `input.witness.pegin_witness.len() < 4` then `continue`; else take `scriptPubKey` from `pegin_witness[3]` and count witness/P2SH sigops against that script.

This tree: `get_sigop_cost` returns early with legacy `* 4` only:

```
        if tx.is_coinbase() || tx.input.iter().any(|input| input.is_pegin) {
            return Ok(n_sigop_cost);
        }
```

That is not the PR's pegin-witness walk. Witness and P2SH sigops are not counted for those transactions. That is a known undercount, not #47.

Skip evidence: dirty 2023 draft versus current Elements `Witness` / `script_sig` types. Cherry-pick will not apply.

Residual: if Liquid REST `sigops` on pegin must later match Elements, port a current-types witness walk. Do not take this patch as-is.

---

## Branch `mononaut/page-sizes` (no pull request)

Stance: **skip** the hard cap of 100. **already here** for the unbounded `max_txs` hole, using existing REST knobs instead of 100.

Live compare is identical to commit `2dc61f50d41619ce99e4837451e7937836b2d0ad` (2026-08-13, message `adjust page sizes`). Files: `src/config.rs` (help text claims summary default is also the maximum), `src/rest.rs`. Quoted constants:

```
+const MAX_HISTORY_TXS: usize = 100;
+
+fn capped_max_txs(query_params: &HashMap<String, String>, default: usize, limit: usize) -> usize {
+    query_params
+        .get("max_txs")
+        .and_then(|value| value.parse::<usize>().ok())
+        .unwrap_or(default)
+        .min(limit)
+}
```

Address, scripthash, mempool-mixed, and summary routes pass `MAX_HISTORY_TXS` as `limit`. That would clamp address summary from default 5000 down to 100. Mempool txid/tx pages cap at `rest_max_mempool_txid_page_size` / `rest_max_mempool_page_size` (10000 / 1000). Included test `test_capped_max_txs` expects `capped_max_txs(..., 25, 100)` of `"1000000"` to equal 100.

This tree clap defaults: `rest_default_chain_txs_per_page` 25, `rest_default_max_mempool_txs` 50, `rest_default_max_address_summary_txs` 5000, `rest_max_mempool_page_size` 1000, `rest_max_mempool_txid_page_size` 10000. `clamp_query_max_txs` is `min(cap, parsed max_txs or cap)`. Comment in `src/rest.rs`: this is not a hard `MAX_HISTORY_TXS = 100`.

Named test: `huge_max_txs_is_clamped_to_rest_knobs_summary_default_stays_5000`. It asserts a huge `max_txs` clamps to 50 / 25 / 1000 / 10000 / 5000 and `assert_ne!(..., 100)` for summary.

---

## Already here (not a parent PR)

These Surmount contracts stay. They are not reasons to merge the skip rows.

- Electrum PROXY and `--electrum-haproxy-depth` via crates.io `ppp` 2.3.0. Depth 0 is production.
- Shared rayon `THREAD_POOL` (the pool half of #145).
- Pegin special-case in `get_sigop_cost` (legacy cost only). That is not #47's witness walk. Document the undercount; do not pretend pegin witness sigops are counted.
- HTTP/1 header-read timeout 10 seconds (`HTTP1_HEADER_READ_TIMEOUT`).
- hyper 1, clap 4, bitcoin 0.32.8, rocksdb 0.24.0, panic abort.

---

## Remaining holes

1. Liquid REST `sigops` on a transaction with any `is_pegin` input undercounts (legacy `* 4` only). Port a current-types walk later only if that REST field must match Elements. Do not take #47 as-is.
2. Shutdown watchdog is armed after Electrum join and RocksDB flush. A hung Electrum join still delays process exit. Upstream #154 would `process::exit(0)` after 5 seconds and skip flush. This tree documents that destructor skip and refuses to skip flush.
3. Do not merge the skip rows. Do not add cache prefix `R`. Do not shrink summary from 5000 to 100.
