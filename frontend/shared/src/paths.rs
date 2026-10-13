//! Indexer paths after splora-http strips the public prefix.
//!
//! `handle_request` in `src/rest.rs` matches these paths after the public prefix is gone:
//! `GET /blocks`, `GET /blocks/:start_height`, `GET /blocks/tip/hash`,
//! `GET /blocks/tip/height`, `GET /mempool/recent`, `GET /mempool`,
//! `GET /fee-estimates`, `GET /block/:hash`, `GET /block/:hash/txids`,
//! `GET /block/:hash/txs`, `GET /block/:hash/txs/:start_index`,
//! `GET /block-height/:height` (plain hash body), `GET /tx/:txid`,
//! `GET /address/:script`, `GET /address/:script/txs`,
//! `GET /address/:script/txs/chain`, `GET /address/:script/txs/chain/:last_seen_txid`,
//! `GET /address/:script/txs/mempool`, `GET /address/:script/txs/summary`,
//! `GET /address/:script/txs/summary/:last_seen_txid`, `GET /address/:script/utxo`,
//! `GET /address-prefix/:prefix`, `GET /scripthash/:hash`, `GET /scripthash/:hash/txs`,
//! `GET /scripthash/:hash/txs/chain`, `GET /scripthash/:hash/txs/chain/:last_seen_txid`,
//! `GET /scripthash/:hash/txs/mempool`, `GET /scripthash/:hash/utxo`,
//! `POST /tx` (raw hex body), `POST /txs/test` (JSON array of hex strings),
//! `POST /txs/package` (JSON array of hex strings), `GET /block/:hash/status`,
//! `GET /tx/:txid/hex`, `GET /tx/:txid/merkle-proof`, `GET /tx/:txid/outspends`,
//! `GET /tx/:txid/outspend/:index`, `GET /mempool/txids`,
//! `GET /mempool/txids/page/:last_seen_txid`,
//! `GET /scripthash/:hash/txs/summary`,
//! `GET /scripthash/:hash/txs/summary/:last_seen_txid`,
//! `POST /addresses/txs`, `POST /scripthashes/txs`,
//! `POST /addresses/txs/summary`, `POST /addresses/txs/summary/:last_seen_txid`,
//! `POST /scripthashes/txs/summary`, `POST /scripthashes/txs/summary/:last_seen_txid`,
//! `GET /txs/outspends`, `GET /tx/:txid/status`, `GET /block/:hash/txid/:index`,
//! `GET /block/:hash/header`, `GET /block/:hash/raw`, `GET /tx/:txid/raw`, and
//! `GET /tx/:txid/merkleblock-proof`.
//!
//! Liquid `GET /asset/:asset`, `GET /asset/:asset/txs`,
//! `GET /asset/:asset/txs/chain`, `GET /asset/:asset/txs/chain/:last_seen_txid`,
//! `GET /asset/:asset/txs/mempool`, `GET /asset/:asset/supply`,
//! `GET /asset/:asset/supply/decimal`, `GET /assets/registry`,
//! `GET /assets/registry/:asset`, and `GET /assets/registry/search` are path
//! strings only. They are not routes.

/// One indexer route: HTTP method plus path pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Route {
    pub method: &'static str,
    pub pattern: &'static str,
}

const ROUTES: [Route; 53] = [
    Route {
        method: "GET",
        pattern: "/blocks",
    },
    Route {
        method: "GET",
        pattern: "/mempool/recent",
    },
    Route {
        method: "GET",
        pattern: "/block/:hash",
    },
    Route {
        method: "GET",
        pattern: "/block/:hash/txs",
    },
    Route {
        method: "GET",
        pattern: "/block-height/:height",
    },
    Route {
        method: "GET",
        pattern: "/tx/:txid",
    },
    Route {
        method: "GET",
        pattern: "/address/:script",
    },
    Route {
        method: "GET",
        pattern: "/address/:script/txs",
    },
    Route {
        method: "GET",
        pattern: "/address/:script/utxo",
    },
    Route {
        method: "POST",
        pattern: "/tx",
    },
    Route {
        method: "POST",
        pattern: "/txs/test",
    },
    Route {
        method: "GET",
        pattern: "/blocks/tip/hash",
    },
    Route {
        method: "GET",
        pattern: "/blocks/tip/height",
    },
    Route {
        method: "GET",
        pattern: "/blocks/:start_height",
    },
    Route {
        method: "GET",
        pattern: "/block/:hash/txids",
    },
    Route {
        method: "GET",
        pattern: "/block/:hash/txs/:start_index",
    },
    Route {
        method: "GET",
        pattern: "/mempool",
    },
    Route {
        method: "GET",
        pattern: "/fee-estimates",
    },
    Route {
        method: "GET",
        pattern: "/block/:hash/status",
    },
    Route {
        method: "GET",
        pattern: "/tx/:txid/outspends",
    },
    Route {
        method: "GET",
        pattern: "/tx/:txid/outspend/:index",
    },
    Route {
        method: "GET",
        pattern: "/tx/:txid/merkle-proof",
    },
    Route {
        method: "GET",
        pattern: "/tx/:txid/hex",
    },
    Route {
        method: "GET",
        pattern: "/address/:script/txs/chain",
    },
    Route {
        method: "GET",
        pattern: "/address/:script/txs/chain/:last_seen_txid",
    },
    Route {
        method: "GET",
        pattern: "/address/:script/txs/mempool",
    },
    Route {
        method: "GET",
        pattern: "/address/:script/txs/summary",
    },
    Route {
        method: "GET",
        pattern: "/address/:script/txs/summary/:last_seen_txid",
    },
    Route {
        method: "GET",
        pattern: "/mempool/txids",
    },
    Route {
        method: "GET",
        pattern: "/mempool/txids/page/:last_seen_txid",
    },
    Route {
        method: "GET",
        pattern: "/address-prefix/:prefix",
    },
    Route {
        method: "GET",
        pattern: "/scripthash/:hash",
    },
    Route {
        method: "GET",
        pattern: "/scripthash/:hash/txs",
    },
    Route {
        method: "GET",
        pattern: "/scripthash/:hash/txs/chain",
    },
    Route {
        method: "GET",
        pattern: "/scripthash/:hash/txs/chain/:last_seen_txid",
    },
    Route {
        method: "GET",
        pattern: "/scripthash/:hash/txs/mempool",
    },
    Route {
        method: "GET",
        pattern: "/scripthash/:hash/utxo",
    },
    Route {
        method: "POST",
        pattern: "/txs/package",
    },
    Route {
        method: "GET",
        pattern: "/scripthash/:hash/txs/summary",
    },
    Route {
        method: "GET",
        pattern: "/scripthash/:hash/txs/summary/:last_seen_txid",
    },
    Route {
        method: "POST",
        pattern: "/addresses/txs",
    },
    Route {
        method: "POST",
        pattern: "/scripthashes/txs",
    },
    Route {
        method: "POST",
        pattern: "/addresses/txs/summary",
    },
    Route {
        method: "POST",
        pattern: "/addresses/txs/summary/:last_seen_txid",
    },
    Route {
        method: "POST",
        pattern: "/scripthashes/txs/summary",
    },
    Route {
        method: "POST",
        pattern: "/scripthashes/txs/summary/:last_seen_txid",
    },
    Route {
        method: "GET",
        pattern: "/txs/outspends",
    },
    Route {
        method: "GET",
        pattern: "/tx/:txid/status",
    },
    Route {
        method: "GET",
        pattern: "/block/:hash/txid/:index",
    },
    Route {
        method: "GET",
        pattern: "/block/:hash/header",
    },
    Route {
        method: "GET",
        pattern: "/block/:hash/raw",
    },
    Route {
        method: "GET",
        pattern: "/tx/:txid/raw",
    },
    Route {
        method: "GET",
        pattern: "/tx/:txid/merkleblock-proof",
    },
];

/// The indexer routes, in the order the crate promises.
pub fn path_list() -> Vec<Route> {
    ROUTES.to_vec()
}

/// Same routes as [`path_list`], borrowed.
pub fn indexer_routes() -> &'static [Route] {
    &ROUTES
}

pub fn blocks_path() -> &'static str {
    "/blocks"
}

/// `GET /blocks/:start_height`. The match arm binds `start_height`.
pub fn blocks_start_height_path(start_height: u64) -> String {
    format!("/blocks/{start_height}")
}

/// `GET /blocks/tip/hash`. The body is plain text, not JSON.
pub fn blocks_tip_hash_path() -> &'static str {
    "/blocks/tip/hash"
}

/// `GET /blocks/tip/height`. The body is plain text, not JSON.
pub fn blocks_tip_height_path() -> &'static str {
    "/blocks/tip/height"
}

pub fn mempool_recent_path() -> &'static str {
    "/mempool/recent"
}

/// `GET /mempool`. The body is `BacklogStats`, not the recent-tx list.
pub fn mempool_path() -> &'static str {
    "/mempool"
}

pub fn fee_estimates_path() -> &'static str {
    "/fee-estimates"
}

pub fn block_path(hash: &str) -> String {
    format!("/block/{hash}")
}

pub fn block_txs_path(hash: &str) -> String {
    format!("/block/{hash}/txs")
}

/// `GET /block/:hash/txs/:start_index`.
///
/// `start_index` is the optional fourth segment on the `txs` match in
/// `src/rest.rs`. When that segment is absent the indexer starts at 0, which
/// is [`block_txs_path`].
pub fn block_txs_start_index_path(hash: &str, start_index: u64) -> String {
    format!("/block/{hash}/txs/{start_index}")
}

pub fn block_txids_path(hash: &str) -> String {
    format!("/block/{hash}/txids")
}

pub fn block_height_path(height: u64) -> String {
    format!("/block-height/{height}")
}

pub fn tx_path(txid: &str) -> String {
    format!("/tx/{txid}")
}

pub fn address_path(script: &str) -> String {
    format!("/address/{script}")
}

pub fn address_txs_path(script: &str) -> String {
    format!("/address/{script}/txs")
}

pub fn address_utxo_path(script: &str) -> String {
    format!("/address/{script}/utxo")
}

pub fn broadcast_path() -> &'static str {
    "/tx"
}

pub fn test_txs_path() -> &'static str {
    "/txs/test"
}

/// Raw hex body for `POST /tx`. Surrounding whitespace is removed.
/// The indexer forwards this string to `sendrawtransaction`.
pub fn broadcast_body(hex: &str) -> String {
    hex.trim().to_string()
}

/// JSON array of hex strings for `POST /txs/test`.
pub fn test_txs_body(hexes: &[&str]) -> String {
    serde_json::to_string(hexes).expect("hex strings serialize to a JSON array")
}

/// `GET /block/:hash/status`.
pub fn block_status_path(hash: &str) -> String {
    format!("/block/{hash}/status")
}

/// `GET /tx/:txid/outspends`.
pub fn tx_outspends_path(txid: &str) -> String {
    format!("/tx/{txid}/outspends")
}

/// `GET /tx/:txid/outspend/:index`.
pub fn tx_outspend_path(txid: &str, index: u64) -> String {
    format!("/tx/{txid}/outspend/{index}")
}

/// `GET /tx/:txid/merkle-proof`.
pub fn tx_merkle_proof_path(txid: &str) -> String {
    format!("/tx/{txid}/merkle-proof")
}

/// `GET /tx/:txid/hex`. The body is plain hex, not JSON.
pub fn tx_hex_path(txid: &str) -> String {
    format!("/tx/{txid}/hex")
}

/// `GET /address/:script/txs/chain`.
pub fn address_txs_chain_path(script: &str) -> String {
    format!("/address/{script}/txs/chain")
}

/// `GET /address/:script/txs/chain/:last_seen_txid`.
pub fn address_txs_chain_page_path(script: &str, last_seen_txid: &str) -> String {
    format!("/address/{script}/txs/chain/{last_seen_txid}")
}

/// `GET /address/:script/txs/mempool`.
pub fn address_txs_mempool_path(script: &str) -> String {
    format!("/address/{script}/txs/mempool")
}

/// `GET /address/:script/txs/summary`.
pub fn address_txs_summary_path(script: &str) -> String {
    format!("/address/{script}/txs/summary")
}

/// `GET /address/:script/txs/summary/:last_seen_txid`.
pub fn address_txs_summary_page_path(script: &str, last_seen_txid: &str) -> String {
    format!("/address/{script}/txs/summary/{last_seen_txid}")
}

/// `GET /mempool/txids`.
pub fn mempool_txids_path() -> &'static str {
    "/mempool/txids"
}

/// `GET /mempool/txids/page/:last_seen_txid`.
pub fn mempool_txids_page_path(last_seen_txid: &str) -> String {
    format!("/mempool/txids/page/{last_seen_txid}")
}

/// `GET /address-prefix/:prefix`.
pub fn address_prefix_path(prefix: &str) -> String {
    format!("/address-prefix/{prefix}")
}

/// `GET /scripthash/:hash`.
pub fn scripthash_path(hash: &str) -> String {
    format!("/scripthash/{hash}")
}

/// `GET /scripthash/:hash/txs`.
pub fn scripthash_txs_path(hash: &str) -> String {
    format!("/scripthash/{hash}/txs")
}

/// `GET /scripthash/:hash/txs/chain`.
pub fn scripthash_txs_chain_path(hash: &str) -> String {
    format!("/scripthash/{hash}/txs/chain")
}

/// `GET /scripthash/:hash/txs/chain/:last_seen_txid`.
pub fn scripthash_txs_chain_page_path(hash: &str, last_seen_txid: &str) -> String {
    format!("/scripthash/{hash}/txs/chain/{last_seen_txid}")
}

/// `GET /scripthash/:hash/txs/mempool`.
pub fn scripthash_txs_mempool_path(hash: &str) -> String {
    format!("/scripthash/{hash}/txs/mempool")
}

/// `GET /scripthash/:hash/utxo`.
pub fn scripthash_utxo_path(hash: &str) -> String {
    format!("/scripthash/{hash}/utxo")
}

/// `POST /txs/package`.
pub fn package_path() -> &'static str {
    "/txs/package"
}

/// JSON array of hex strings for `POST /txs/package`.
pub fn package_body(hexes: &[&str]) -> String {
    test_txs_body(hexes)
}

/// `GET /asset/:asset`. Path string only. Not an indexer route.
pub fn asset_path(asset: &str) -> String {
    format!("/asset/{asset}")
}

/// `GET /asset/:asset/txs`. Path string only. Not an indexer route.
pub fn asset_txs_path(asset: &str) -> String {
    format!("/asset/{asset}/txs")
}

/// `GET /assets/registry/:asset`. Path string only. Not an indexer route.
pub fn asset_registry_path(asset: &str) -> String {
    format!("/assets/registry/{asset}")
}

/// The indexer rejects `GET /txs/outspends` above this many transaction ids.
pub const TXS_OUTSPENDS_MAX: usize = 50;

/// `GET /scripthash/:hash/txs/summary`.
pub fn scripthash_txs_summary_path(hash: &str) -> String {
    format!("/scripthash/{hash}/txs/summary")
}

/// `GET /scripthash/:hash/txs/summary/:last_seen_txid`.
pub fn scripthash_txs_summary_page_path(hash: &str, last_seen_txid: &str) -> String {
    format!("/scripthash/{hash}/txs/summary/{last_seen_txid}")
}

/// `POST /addresses/txs`.
pub fn addresses_txs_path() -> &'static str {
    "/addresses/txs"
}

/// `POST /scripthashes/txs`.
pub fn scripthashes_txs_path() -> &'static str {
    "/scripthashes/txs"
}

/// `POST /addresses/txs/summary`.
pub fn addresses_txs_summary_path() -> &'static str {
    "/addresses/txs/summary"
}

/// `POST /addresses/txs/summary/:last_seen_txid`.
pub fn addresses_txs_summary_page_path(last_seen_txid: &str) -> String {
    format!("/addresses/txs/summary/{last_seen_txid}")
}

/// `POST /scripthashes/txs/summary`.
pub fn scripthashes_txs_summary_path() -> &'static str {
    "/scripthashes/txs/summary"
}

/// `POST /scripthashes/txs/summary/:last_seen_txid`.
pub fn scripthashes_txs_summary_page_path(last_seen_txid: &str) -> String {
    format!("/scripthashes/txs/summary/{last_seen_txid}")
}

/// JSON array of address or scripthash strings for the batch transaction posts.
pub fn addresses_txs_body(scripts: &[&str]) -> String {
    serde_json::to_string(scripts).expect("script strings serialize to a JSON array")
}

/// JSON array of scripthash strings for `POST /scripthashes/txs` and the summary posts.
pub fn scripthashes_txs_body(hashes: &[&str]) -> String {
    addresses_txs_body(hashes)
}

/// `GET /txs/outspends?txids=`.
///
/// At most [`TXS_OUTSPENDS_MAX`] ids. An empty list, a comma inside an id, or
/// a longer list is an error. The indexer returns 400 above that limit.
pub fn txs_outspends_path(txids: &[&str]) -> Result<String, &'static str> {
    if txids.is_empty() {
        return Err("txids query was empty");
    }
    if txids.len() > TXS_OUTSPENDS_MAX {
        return Err("txids query is limited to 50");
    }
    if txids
        .iter()
        .any(|txid| txid.is_empty() || txid.contains(',') || txid.contains('&'))
    {
        return Err("txid was empty or contained a separator");
    }
    Ok(format!("/txs/outspends?txids={}", txids.join(",")))
}

/// `GET /tx/:txid/status`.
pub fn tx_status_path(txid: &str) -> String {
    format!("/tx/{txid}/status")
}

/// `GET /block/:hash/txid/:index`. The body is one txid as plain text.
pub fn block_txid_path(hash: &str, index: u64) -> String {
    format!("/block/{hash}/txid/{index}")
}

/// `GET /block/:hash/header`. The body is hex text.
pub fn block_header_path(hash: &str) -> String {
    format!("/block/{hash}/header")
}

/// `GET /block/:hash/raw`. The body is raw bytes.
pub fn block_raw_path(hash: &str) -> String {
    format!("/block/{hash}/raw")
}

/// `GET /tx/:txid/raw`. The body is raw bytes.
pub fn tx_raw_path(txid: &str) -> String {
    format!("/tx/{txid}/raw")
}

/// `GET /tx/:txid/merkleblock-proof`. Hex text. Omitted when the liquid feature is on.
pub fn tx_merkleblock_proof_path(txid: &str) -> String {
    format!("/tx/{txid}/merkleblock-proof")
}

/// `GET /assets/registry`. Path string only. Not an indexer route in this table.
pub fn assets_registry_path() -> &'static str {
    "/assets/registry"
}

/// `GET /assets/registry` with the query that arm already reads.
///
/// Order is `start_index`, `limit`, `sort_field`, `sort_dir`. `None` omits
/// the key. `Some("")` keeps `key=`. A missing `sort_field` is ticker and a
/// missing `sort_dir` is asc. An empty sort key is rejected. An empty
/// `start_index` or `limit` parses the same as a missing key.
pub fn assets_registry_page_path(
    start_index: Option<&str>,
    limit: Option<&str>,
    sort_field: Option<&str>,
    sort_dir: Option<&str>,
) -> String {
    let mut query = String::new();
    append_query(&mut query, "start_index", start_index);
    append_query(&mut query, "limit", limit);
    append_query(&mut query, "sort_field", sort_field);
    append_query(&mut query, "sort_dir", sort_dir);
    if query.is_empty() {
        assets_registry_path().to_string()
    } else {
        format!("{}?{query}", assets_registry_path())
    }
}

/// `GET /assets/registry/search?q=`. Path string only.
pub fn assets_registry_search_path(query: &str) -> String {
    format!("/assets/registry/search?q={}", encode_query(query))
}

/// `GET /asset/:asset/txs/chain`. Path string only.
pub fn asset_txs_chain_path(asset: &str) -> String {
    format!("/asset/{asset}/txs/chain")
}

/// `GET /asset/:asset/txs/chain/:last_seen_txid`. Path string only.
pub fn asset_txs_chain_page_path(asset: &str, last_seen_txid: &str) -> String {
    format!("/asset/{asset}/txs/chain/{last_seen_txid}")
}

/// `GET /asset/:asset/txs/mempool`. Path string only.
pub fn asset_txs_mempool_path(asset: &str) -> String {
    format!("/asset/{asset}/txs/mempool")
}

/// `GET /asset/:asset/supply`. Path string only. The body is plain integer text.
pub fn asset_supply_path(asset: &str) -> String {
    format!("/asset/{asset}/supply")
}

/// `GET /asset/:asset/supply/decimal`. Path string only.
pub fn asset_supply_decimal_path(asset: &str) -> String {
    format!("/asset/{asset}/supply/decimal")
}

fn append_query(query: &mut String, key: &str, value: Option<&str>) {
    let Some(value) = value else {
        return;
    };
    if !query.is_empty() {
        query.push('&');
    }
    query.push_str(key);
    query.push('=');
    query.push_str(&encode_query(value));
}

fn encode_query(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
