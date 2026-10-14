//! Assets registry table header. Angular `i18n="Asset name header"` paints
//! `<th class="td-name">Name</th>` on the loaded assets list. This registry
//! list paints `<th>Name</th>` once when `GET /assets/registry` returns rows.
//!
//! The header is that element. It is first, before Ticker, Issuer domain, and
//! Asset ID. It is not the parenthetical ticker on an issued asset Name line
//! (`Test Asset (TEST)`), and it is not `<td>Name</td>`. An empty list, a
//! notice, invalid JSON, and mainnet do not paint it. The loading thead is
//! not painted.

use splora_web::registry_list_markup;

const ASSET: &str = "1212121212121212121212121212121212121212121212121212121212121212";
const OTHER: &str = "3434343434343434343434343434343434343434343434343434343434343434";
const ISSUANCE_TX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HEADER: &str = "<th>Name</th>";

fn issued_row(asset_id: &str, name: &str, ticker: &str) -> String {
    format!(
        "{{\"asset_id\":\"{asset_id}\",\"name\":\"{name}\",\"ticker\":\"{ticker}\",\"precision\":2,\"issuance_txin\":{{\"txid\":\"{ISSUANCE_TX}\",\"vin\":0}},\"issuance_prevout\":{{\"txid\":\"{ISSUANCE_TX}\",\"vout\":0}},\"reissuance_token\":\"{ISSUANCE_TX}\",\"status\":{{\"confirmed\":true}},\"chain_stats\":{{\"tx_count\":1,\"issuance_count\":1,\"issued_amount\":1,\"burned_amount\":0,\"has_blinded_issuances\":false,\"burned_reissuance_tokens\":0}},\"mempool_stats\":{{\"tx_count\":0,\"issuance_count\":0,\"issued_amount\":0,\"burned_amount\":0,\"has_blinded_issuances\":false,\"burned_reissuance_tokens\":0}}}}"
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
    for needle in ["<td>Name</td>", "Maximum burn amount", "Test Asset (TEST)"] {
        assert_absent(markup, needle);
    }
}

/// One issued row paints the column header once, before Ticker, and still
/// paints the name and the ticker value.
#[test]
fn issued_row_paints_the_name_header_once() {
    let markup = registry_list_markup(&one_row(), true);
    assert_one(&markup, HEADER);
    assert_eq!(markup.matches("Name").count(), 1, "{markup}");
    assert_one(&markup, "<th>Ticker</th>");
    assert_one(&markup, "<th>Issuer domain</th>");
    assert_one(&markup, "<th>Asset ID</th>");
    assert_one(&markup, "Test Asset");
    assert_one(&markup, "TEST");
    assert!(
        markup.find(HEADER).unwrap() < markup.find("<th>Ticker</th>").unwrap(),
        "Name stays before Ticker: {markup}"
    );
    assert!(
        markup.find("<th>Ticker</th>").unwrap() < markup.find("<th>Issuer domain</th>").unwrap(),
        "Ticker stays before Issuer domain: {markup}"
    );
    assert!(
        markup.find("<th>Issuer domain</th>").unwrap() < markup.find("<th>Asset ID</th>").unwrap(),
        "Issuer domain stays before Asset ID: {markup}"
    );
    assert!(
        markup.find("<th>Asset ID</th>").unwrap() < markup.find("Test Asset").unwrap(),
        "the header belongs before the row: {markup}"
    );
    assert!(
        markup.contains(&format!("/asset/{ASSET}")),
        "the asset link was dropped: {markup}"
    );
    assert_other_headers_stay_off(&markup);
    assert_absent(&registry_list_markup(&one_row(), false), "Name");
    assert_absent(&registry_list_markup(&one_row(), false), "<th>");
}

/// Two rows still paint the header once, not once per asset.
#[test]
fn two_rows_paint_the_name_header_once() {
    let markup = registry_list_markup(&two_rows(), true);
    assert_one(&markup, HEADER);
    assert_eq!(markup.matches("Name").count(), 1, "{markup}");
    assert_one(&markup, "<th>Ticker</th>");
    assert_one(&markup, "<th>Issuer domain</th>");
    assert_one(&markup, "<th>Asset ID</th>");
    assert_one(&markup, "Test Asset");
    assert_one(&markup, "Other Asset");
    assert_one(&markup, "TEST");
    assert_one(&markup, "OTHR");
    assert_other_headers_stay_off(&markup);
}

/// No rows means no column header. The empty list sentence stays.
#[test]
fn empty_list_omits_the_name_header() {
    let empty = registry_list_markup("[]", true);
    assert_eq!(empty, "<p class=\"empty\">There are no assets.</p>");
    assert_absent(&empty, "Name");
    assert_absent(&empty, "<th>");

    let blank = registry_list_markup("", true);
    assert_eq!(blank, "");
    assert_absent(&blank, "Name");

    let invalid = registry_list_markup("{not-json", true);
    assert_eq!(invalid, "");
    assert_absent(&invalid, "Name");

    let notice = registry_list_markup("not-served", true);
    assert!(notice.contains("not-served"), "{notice}");
    assert_absent(&notice, "Name");
    assert_absent(&notice, "<th>");
}
