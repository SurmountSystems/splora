//! Indexer paths after splora-http strips the public prefix.
//!
//! `handle_request` in `src/rest.rs` matches these paths after the public prefix is gone:
//! `GET /blocks`, `GET /blocks/:start_height`, `GET /blocks/tip/hash`,
//! `GET /blocks/tip/height`, `GET /mempool/recent`, `GET /mempool`,
//! `GET /fee-estimates`, `GET /block/:hash`, `GET /block/:hash/txids`,
//! `GET /block/:hash/txs`, `GET /block/:hash/txs/:start_index`,
//! `GET /block-height/:height` (plain hash body), `GET /tx/:txid`,
//! `GET /address/:script`, `GET /address/:script/txs`,
//! `GET /address/:script/utxo`, `POST /tx` (raw hex body), and
//! `POST /txs/test` (JSON array of hex strings).

/// One indexer route: HTTP method plus path pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Route {
    pub method: &'static str,
    pub pattern: &'static str,
}

const ROUTES: [Route; 18] = [
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
