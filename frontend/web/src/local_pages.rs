//! Thin wrappers over `splora_frontend_shared` for the Leptos pages.
//! Fixture rates are constants. These functions do not use the network.

use splora_frontend_shared::{
    FiatCode, btc_fiat_jpy, btc_fiat_usd, btc_to_sats, cap_btc_supply, format_fiat,
    route_availability, sats_fiat_usd, switch_usd_jpy_usd, truncate_btc_decimals,
};

/// USD per BTC fixture. Shown only through [`format_fiat`].
pub const USD_PER_BTC: &str = "123456";

/// JPY per BTC fixture. Shown only through [`format_fiat`].
pub const JPY_PER_BTC: &str = "11057757";

pub fn one_btc_sats_text() -> String {
    btc_to_sats("1").expect("one btc converts").to_string()
}

pub fn truncated_btc_text(input: &str) -> String {
    truncate_btc_decimals(input)
}

pub fn capped_btc_text(input: &str) -> String {
    cap_btc_supply(input).btc
}

pub fn usd_fixture_text() -> String {
    format_fiat(FiatCode::Usd, USD_PER_BTC)
}

pub fn jpy_fixture_text() -> String {
    format_fiat(FiatCode::Jpy, JPY_PER_BTC)
}

pub fn switched_fiat_symbols() -> [char; 3] {
    let shown = switch_usd_jpy_usd("1500");
    [
        shown[0].chars().next().expect("usd symbol"),
        shown[1].chars().next().expect("jpy symbol"),
        shown[2].chars().next().expect("usd symbol"),
    ]
}

pub fn statistics_page_text() -> &'static str {
    route_availability("/api/v1/statistics")
}

pub fn mining_page_text() -> &'static str {
    route_availability("/v1/mining")
}

pub fn lightning_page_text() -> &'static str {
    route_availability("/lightning")
}

pub fn graphs_page_text() -> &'static str {
    route_availability("/api/v1/statistics")
}

pub fn hashrate_page_text() -> &'static str {
    route_availability("/v1/mining")
}

pub fn price_chart_page_text() -> &'static str {
    route_availability("/api/v1/historical-price")
}

/// Route list the docs view shows.
pub fn docs_view_text() -> String {
    [
        "Indexer routes. This screen is local text. It has no private-key field.",
        "Prefixes",
        "/",
        "/testnet",
        "/testnet4",
        "/mutinynet",
        "/liquid",
        "/signet is HTTP 307 to /mutinynet, not a sixth chain.",
        "Routes",
        "GET /api/v1/ws",
        "GET /blocks/tip/hash",
        "GET /blocks/tip/height",
        "GET /blocks",
        "GET /blocks/:start_height",
        "GET /block-template",
        "GET /block-height/:height",
        "GET /block/:hash",
        "GET /block/:hash/status",
        "GET /block/:hash/txids",
        "GET /block/:hash/header",
        "GET /block/:hash/raw",
        "GET /block/:hash/txid/:index",
        "GET /block/:hash/txs",
        "GET /block/:hash/txs/:start_index",
        "GET /internal/block/:hash/txs",
        "GET /tx/:txid",
        "GET /tx/:txid/hex",
        "GET /tx/:txid/raw",
        "GET /tx/:txid/status",
        "GET /tx/:txid/merkle-proof",
        "GET /tx/:txid/merkleblock-proof",
        "GET /tx/:txid/outspend/:index",
        "GET /tx/:txid/outspends",
        "GET /txs/outspends",
        "POST /tx",
        "GET /broadcast",
        "POST /txs/test",
        "POST /txs/package",
        "POST /electrum",
        "POST /internal/txs",
        "POST /internal/txs/outspends/by-txid",
        "POST /internal/txs/outspends/by-outpoint",
        "GET /address/:address",
        "GET /address/:address/txs",
        "GET /address/:address/txs/chain",
        "GET /address/:address/txs/summary",
        "GET /address/:address/txs/mempool",
        "GET /address/:address/utxo",
        "GET /scripthash/:scripthash",
        "GET /scripthash/:scripthash/txs",
        "GET /scripthash/:scripthash/txs/chain",
        "GET /scripthash/:scripthash/txs/summary",
        "GET /scripthash/:scripthash/txs/mempool",
        "GET /scripthash/:scripthash/utxo",
        "GET /address-prefix/:prefix",
        "POST /addresses/txs",
        "POST /addresses/txs/summary",
        "POST /scripthashes/txs",
        "POST /scripthashes/txs/summary",
        "GET /mempool",
        "GET /mempool/txids",
        "GET /mempool/txids/page",
        "GET /mempool/txids/page/:last_seen_txid",
        "GET /mempool/recent",
        "GET /internal/mempool/txs",
        "GET /internal/mempool/txs/:last_seen_txid",
        "GET /internal/mempool/txs/all",
        "POST /internal/mempool/txs",
        "GET /fee-estimates",
        "Liquid feature routes",
        "GET /assets/registry",
        "GET /assets/registry/search",
        "GET /assets/registry/:asset",
        "GET /asset/:asset",
        "GET /asset/:asset/txs",
        "GET /asset/:asset/txs/chain",
        "GET /asset/:asset/txs/chain/:last_seen_txid",
        "GET /asset/:asset/txs/mempool",
        "GET /asset/:asset/supply",
        "GET /asset/:asset/supply/:decimal",
        "GET /tx/:txid/merkleblock-proof is omitted when the liquid feature is on.",
    ]
    .join("\n")
}

/// Four fixture amounts the calculator shows. Formatting stays in shared.
pub fn calculator_fiat_text() -> String {
    [
        btc_fiat_usd("0.33"),
        btc_fiat_usd("0.00000001"),
        btc_fiat_jpy("1"),
        sats_fiat_usd(10_000),
    ]
    .join("\n")
}
