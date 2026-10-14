//! Assets registry table header. Angular `i18n="Asset ticker header"` paints
//! `<th>Ticker</th>` once on the assets list when `GET /assets/registry`
//! returns rows.
//!
//! The header is that element. It is not the parenthetical ticker on an
//! issued asset Name line (`Test Asset (TEST)`). An empty list, a notice,
//! invalid JSON, and mainnet do not paint it. Issuer domain is a separate
//! column header on the same table. This file does not forbid that header.

use splora_web::registry_list_markup;

const ASSET: &str = "1212121212121212121212121212121212121212121212121212121212121212";
const OTHER: &str = "3434343434343434343434343434343434343434343434343434343434343434";
const ISSUANCE_TX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HEADER: &str = "<th>Ticker</th>";

fn issued_row(asset_id: &str, name: &str, ticker: &str) -> String {
    format!(
        r#"{{"asset_id":"{asset_id}","name":"{name}","ticker":"{ticker}","precision":2,"issuance_txin":{{"txid":"{ISSUANCE_TX}","vin":0}},"issuance_prevout":{{"txid":"{ISSUANCE_TX}","vout":0}},"reissuance_token":"{ISSUANCE_TX}","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}"#
    )
}

fn one_row() -> String {
    format!("[{}]", issued_row(ASSET, "Test Asset", "TEST"))
}

fn two_rows() -> String {
    format!(
        "[{},{}]",
        issued_row(ASSET, "Test Asset", "TEST"),
        issued_row(OTHER, "Other Asset", "OTHR")
    )
}

fn assert_one(markup: &str, expected: &str) {
    assert_eq!(
        markup.matches(expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

fn assert_absent(markup: &str, needle: &str) {
    assert!(
        !markup.contains(needle),
        "{needle} leaked into the markup: {markup}"
    );
}

fn assert_other_headers_stay_off(markup: &str) {
    for needle in [
        "Maximum burn amount",
        "Maximum fee rate",
        "Submit Package",
        "transaction hex",
        "script asm",
        "<td>Name</td>",
        "Test Asset (TEST)",
    ] {
        assert_absent(markup, needle);
    }
}

/// One issued row paints the column header once, and still paints the name
/// and the ticker value. The Name-line parenthetical is not that header.
#[test]
fn issued_row_paints_the_ticker_header_once() {
    let markup = registry_list_markup(&one_row(), true);
    assert_one(&markup, HEADER);
    assert_eq!(markup.matches("Ticker").count(), 1, "{markup}");
    assert_one(&markup, "Test Asset");
    assert_one(&markup, "TEST");
    assert!(
        markup.find(HEADER).unwrap() < markup.find("Test Asset").unwrap(),
        "the header belongs before the row: {markup}"
    );
    assert!(
        markup.contains(&format!("/asset/{ASSET}")),
        "the asset link was dropped: {markup}"
    );
    assert_other_headers_stay_off(&markup);
    assert_absent(&registry_list_markup(&one_row(), false), "Ticker");
}

/// Two rows still paint the header once, not once per asset.
#[test]
fn two_rows_paint_the_ticker_header_once() {
    let markup = registry_list_markup(&two_rows(), true);
    assert_one(&markup, HEADER);
    assert_eq!(markup.matches("Ticker").count(), 1, "{markup}");
    assert_one(&markup, "Test Asset");
    assert_one(&markup, "Other Asset");
    assert_one(&markup, "TEST");
    assert_one(&markup, "OTHR");
    assert_other_headers_stay_off(&markup);
}

/// No rows means no column header. The empty list sentence stays.
#[test]
fn empty_list_omits_the_ticker_header() {
    let empty = registry_list_markup("[]", true);
    assert_eq!(empty, "<p class=\"empty\">There are no assets.</p>");
    assert_absent(&empty, "Ticker");
    assert_absent(&empty, "<th>");

    let blank = registry_list_markup("", true);
    assert_eq!(blank, "");
    assert_absent(&blank, "Ticker");

    let invalid = registry_list_markup("{not-json", true);
    assert_eq!(invalid, "");
    assert_absent(&invalid, "Ticker");

    let notice = registry_list_markup("not-served", true);
    assert!(notice.contains("not-served"), "{notice}");
    assert_absent(&notice, "Ticker");
    assert_absent(&notice, "<th>");
}
