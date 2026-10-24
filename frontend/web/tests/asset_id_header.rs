//! Assets registry table header. Angular `i18n="Asset ID header"` paints
//! `<th>Asset ID</th>` once on the assets list when `GET /assets/registry`
//! returns rows.
//!
//! The header is that element. It follows Ticker and Issuer domain. It is
//! not the parenthetical ticker on an issued asset Name line
//! (`Test Asset (TEST)`). An empty list, a notice, invalid JSON, and mainnet
//! do not paint it. This file does not forbid the Ticker header or the
//! Issuer domain header, and it does not require a Name header.

use splora_web::registry_list_markup;

const ASSET: &str = "1212121212121212121212121212121212121212121212121212121212121212";
const OTHER: &str = "3434343434343434343434343434343434343434343434343434343434343434";
const ISSUANCE_TX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HEADER: &str = "<th>Asset ID</th>";

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

/// Whole line `Asset ID` beside the header cell when registry rows parse.
///
/// GPUI requires one line whose entire text is Asset ID. The th stays.
/// Mainnet, an empty list, and an unparsed body omit the line.
#[test]
fn issued_row_paints_asset_id_as_its_own_line() {
    const LINE: &str = "Asset ID";
    let markup = registry_list_markup(&one_row(), true);
    assert!(markup.contains(HEADER), "the header was dropped: {markup}");
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );

    let mainnet = registry_list_markup(&one_row(), false);
    assert!(
        !mainnet.lines().any(|line| line == LINE),
        "mainnet painted {LINE}: {mainnet}"
    );

    let empty = registry_list_markup("[]", true);
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty list painted {LINE}: {empty}"
    );

    let invalid = registry_list_markup("{not-json", true);
    assert!(
        !invalid.lines().any(|line| line == LINE),
        "invalid JSON painted {LINE}: {invalid}"
    );
}

/// One issued row paints the column header once, after Issuer domain,
/// and still paints the name and the ticker value.
#[test]
fn issued_row_paints_the_asset_id_header_once() {
    let markup = registry_list_markup(&one_row(), true);
    assert_one(&markup, HEADER);
    // The th and the whole line each contain the words.
    assert_eq!(markup.matches("Asset ID").count(), 2, "{markup}");
    assert_one(&markup, "<th>Ticker</th>");
    assert_one(&markup, "<th>Issuer domain</th>");
    assert_one(&markup, "Test Asset");
    assert_one(&markup, "TEST");
    assert!(
        markup.find("<th>Ticker</th>").unwrap() < markup.find("<th>Issuer domain</th>").unwrap(),
        "Ticker stays before Issuer domain: {markup}"
    );
    assert!(
        markup.find("<th>Issuer domain</th>").unwrap() < markup.find(HEADER).unwrap(),
        "Issuer domain stays before Asset ID: {markup}"
    );
    assert!(
        markup.find(HEADER).unwrap() < markup.find("Test Asset").unwrap(),
        "the header belongs before the row: {markup}"
    );
    assert!(
        markup.contains(&format!("/asset/{ASSET}")),
        "the asset link was dropped: {markup}"
    );
    assert_other_headers_stay_off(&markup);
    assert_absent(&registry_list_markup(&one_row(), false), "Asset ID");
    assert_absent(&registry_list_markup(&one_row(), false), "<th>");
}

/// Two rows still paint the header once, not once per asset.
#[test]
fn two_rows_paint_the_asset_id_header_once() {
    let markup = registry_list_markup(&two_rows(), true);
    assert_one(&markup, HEADER);
    // The th and the whole line each contain the words.
    assert_eq!(markup.matches("Asset ID").count(), 2, "{markup}");
    assert_one(&markup, "<th>Ticker</th>");
    assert_one(&markup, "<th>Issuer domain</th>");
    assert_one(&markup, "Test Asset");
    assert_one(&markup, "Other Asset");
    assert_one(&markup, "TEST");
    assert_one(&markup, "OTHR");
    assert_other_headers_stay_off(&markup);
}

/// No rows means no column header. The empty list sentence stays.
#[test]
fn empty_list_omits_the_asset_id_header() {
    let empty = registry_list_markup("[]", true);
    assert_eq!(empty, "<p class=\"empty\">There are no assets.</p>");
    assert_absent(&empty, "Asset ID");
    assert_absent(&empty, "<th>");

    let blank = registry_list_markup("", true);
    assert_eq!(blank, "");
    assert_absent(&blank, "Asset ID");

    let invalid = registry_list_markup("{not-json", true);
    assert_eq!(invalid, "");
    assert_absent(&invalid, "Asset ID");

    let notice = registry_list_markup("not-served", true);
    assert!(notice.contains("not-served"), "{notice}");
    assert_absent(&notice, "Asset ID");
    assert_absent(&notice, "<th>");
}
