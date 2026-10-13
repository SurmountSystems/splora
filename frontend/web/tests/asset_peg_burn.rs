//! Peg-in, peg-out, and burn rows on the asset page.
//!
//! `GET /asset/:id` and `GET /asset/:id/txs` are in `src/rest.rs`.
//! The Angular rows are `shows asset peg in/out and burn transactions`
//! in `liquid.spec.ts` and `liquidtestnet.spec.ts`.
//! Those rows are ordinary input and output tables. They are not asset boxes.

use splora_web::asset_markup;

const NATIVE_ASSET: &str = "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
const NATIVE_TEST_ASSET: &str = "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49";
const ISSUED_ASSET: &str = "ac3e0ff248c5051ffd61e00155b7122e5ebc04fd397a0ecbdd4f4e4a56232926";
const TXID: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const INPUT_TXID: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const INPUT_ADDRESS: &str = "1BxoGcMg14oaH3CwHD2hF4gU9VcfgX5yoR";
const OUTPUT_ADDRESS: &str = "1FeexV6bAHb8ybZjqQMjJrcCrHGW9sb6uF";

fn asset_json(id: &str, name: &str, ticker: &str) -> String {
    format!(r#"{{"asset_id":"{id}","name":"{name}","ticker":"{ticker}"}}"#)
}

fn one_tx() -> String {
    format!(
        r#"{{"txid":"{TXID}","version":2,"locktime":0,"vin":[{{"txid":"{INPUT_TXID}","vout":0,"prevout":{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"{INPUT_ADDRESS}","value":5000}},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"76a914cc88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"{OUTPUT_ADDRESS}","value":4000}}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn txs_json() -> String {
    format!("[{}]", one_tx())
}

fn txs_repeat(count: usize) -> String {
    format!("[{}]", vec![one_tx(); count].join(","))
}

fn asset_json_counts(id: &str, name: &str, ticker: &str, chain: u64, mempool: u64) -> String {
    format!(
        r#"{{"asset_id":"{id}","name":"{name}","ticker":"{ticker}","chain_stats":{{"tx_count":{chain}}},"mempool_stats":{{"tx_count":{mempool}}}}}"#
    )
}

fn table_has_plain_row(markup: &str, class: &str) -> bool {
    let start = format!("<table class=\"{class}\">");
    let Some(at) = markup.find(&start) else {
        return false;
    };
    let rest = &markup[at + start.len()..];
    let end = rest.find("</table>").unwrap_or(rest.len());
    let table = &rest[..end];
    table.contains("<tr") && !table.contains("assetBox")
}

fn assert_plain_transaction_rows(markup: &str) {
    assert!(
        table_has_plain_row(markup, "table-tx-vin"),
        "asset transaction list has no plain table-tx-vin row: {markup}"
    );
    assert!(
        table_has_plain_row(markup, "table-tx-vout"),
        "asset transaction list has no plain table-tx-vout row: {markup}"
    );
    assert!(!markup.contains("assetBox"), "{markup}");
    assert!(markup.contains(INPUT_ADDRESS), "{markup}");
    assert!(markup.contains(OUTPUT_ADDRESS), "{markup}");
    assert!(markup.contains(&format!("href=\"/tx/{TXID}\"")), "{markup}");
    assert!(!markup.contains("0.00100000"), "{markup}");
    assert!(!markup.contains("Error: Invalid blinding"), "{markup}");
    assert!(!markup.contains("Fiat price last updated"), "{markup}");
}

#[test]
fn liquid_native_asset_shows_peg_in_out_and_burn_rows() {
    let markup = asset_markup(
        &asset_json(NATIVE_ASSET, "Liquid Bitcoin", "L-BTC"),
        &txs_json(),
        true,
    );
    assert!(
        markup.contains("<h2>1 of 0&nbsp;Peg In/Out and Burn Transactions</h2>"),
        "{markup}"
    );
    assert_eq!(markup.matches("<h2>").count(), 1, "{markup}");
    assert_plain_transaction_rows(&markup);
    assert!(markup.contains("Liquid Bitcoin"), "{markup}");
    assert!(markup.contains("L-BTC"), "{markup}");
    let test_native = asset_markup(
        &asset_json(NATIVE_TEST_ASSET, "Liquid Bitcoin", "tLBTC"),
        &txs_json(),
        true,
    );
    assert!(
        test_native.contains("<h2>1 of 0&nbsp;Issuance and Burn Transactions</h2>"),
        "{test_native}"
    );
    assert_eq!(test_native.matches("<h2>").count(), 1, "{test_native}");
    assert!(
        !test_native.contains("Peg In/Out and Burn Transactions"),
        "{test_native}"
    );
    assert_plain_transaction_rows(&test_native);
    let hidden = asset_markup(
        &asset_json(NATIVE_ASSET, "Liquid Bitcoin", "L-BTC"),
        &txs_json(),
        false,
    );
    assert!(
        !hidden.contains("Peg In/Out and Burn Transactions"),
        "{hidden}"
    );
    assert!(!hidden.contains("table-tx-vin"), "{hidden}");
}

#[test]
fn liquid_testnet_asset_shows_rows_that_are_not_asset_boxes() {
    let markup = asset_markup(
        &asset_json(ISSUED_ASSET, "Test Asset", "TEST"),
        &txs_json(),
        true,
    );
    assert!(
        markup.contains("<h2>1 of 0&nbsp;Issuance and Burn Transactions</h2>"),
        "{markup}"
    );
    assert_eq!(markup.matches("<h2>").count(), 1, "{markup}");
    assert!(
        !markup.contains("Peg In/Out and Burn Transactions"),
        "{markup}"
    );
    assert_plain_transaction_rows(&markup);
}

/// Angular `asset.component.html` prefixes `transactions.length of txCount`
/// inside the same `h2`. `txCount` is `chain_stats.tx_count` plus
/// `mempool_stats.tx_count`. The number pipe is en-US, so 1234 is `1,234`.
/// The gap before the title is `&nbsp;`. An empty list omits the prefix.
#[test]
fn asset_heading_prefixes_loaded_of_total_in_the_same_h2() {
    let txs = txs_repeat(1_000);
    let native = asset_markup(
        &asset_json_counts(NATIVE_ASSET, "Liquid Bitcoin", "L-BTC", 1_000, 234),
        &txs,
        true,
    );
    assert!(
        native.contains("<h2>1,000 of 1,234&nbsp;Peg In/Out and Burn Transactions</h2>"),
        "{native}"
    );
    assert_eq!(native.matches("<h2>").count(), 1, "{native}");
    assert!(
        !native.contains("Issuance and Burn Transactions"),
        "{native}"
    );
    let issued = asset_markup(
        &asset_json_counts(ISSUED_ASSET, "Test Asset", "TEST", 1_000, 234),
        &txs,
        true,
    );
    assert!(
        issued.contains("<h2>1,000 of 1,234&nbsp;Issuance and Burn Transactions</h2>"),
        "{issued}"
    );
    assert_eq!(issued.matches("<h2>").count(), 1, "{issued}");
    assert!(
        !issued.contains("Peg In/Out and Burn Transactions"),
        "{issued}"
    );
    let empty = asset_markup(
        &asset_json_counts(NATIVE_ASSET, "Liquid Bitcoin", "L-BTC", 1_000, 234),
        "[]",
        true,
    );
    assert!(
        empty.contains("<h2>Peg In/Out and Burn Transactions</h2>"),
        "{empty}"
    );
    assert!(!empty.contains(" of "), "{empty}");
    assert_eq!(empty.matches("<h2>").count(), 1, "{empty}");
}

#[test]
fn asset_page_paints_asset_markup() {
    let src = include_str!("../src/views.rs");
    let marker = "fn AssetPage(";
    let start = src
        .find(marker)
        .unwrap_or_else(|| panic!("AssetPage is missing"));
    let rest = &src[start..];
    let end = rest.find("\nfn ").unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("asset_markup("),
        "AssetPage does not paint asset_markup"
    );
}
