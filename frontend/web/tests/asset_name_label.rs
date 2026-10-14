//! Name row on the Liquid asset page.
//!
//! `GET /asset/:id` already returns the issued name and ticker. The row is
//! the escaped name, plus the escaped ticker in parentheses when that ticker
//! is present. A missing, null, or blank name omits the row. A pegged asset
//! omits it. The loose name text above the table stays.

use splora_web::{asset_markup, asset_page_markup};

const NATIVE_ASSET: &str = "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
const ISSUED: &str = "ac3e0ff248c5051ffd61e00155b7122e5ebc04fd397a0ecbdd4f4e4a56232926";
const ISSUANCE_TX: &str = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
const PREV_TX: &str = "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef";
const REISSUE: &str = "1111111111111111111111111111111111111111111111111111111111111111";

enum Field<'a> {
    Omit,
    Null,
    Text(&'a str),
}

fn json_string_field(key: &str, field: Field<'_>) -> String {
    match field {
        Field::Omit => String::new(),
        Field::Null => format!(r#""{key}":null,"#),
        Field::Text(value) => format!(r#""{key}":"{value}","#),
    }
}

fn issued_document(name: Field<'_>, ticker: Field<'_>) -> String {
    let name_field = json_string_field("name", name);
    let ticker_field = json_string_field("ticker", ticker);
    format!(
        r#"{{"asset_id":"{ISSUED}",{name_field}{ticker_field}"precision":2,"entity":{{"domain":"registry.example"}},"contract":null,"issuance_txin":{{"txid":"{ISSUANCE_TX}","vin":0}},"issuance_prevout":{{"txid":"{PREV_TX}","vout":1}},"reissuance_token":"{REISSUE}","status":{{"confirmed":true}},"chain_stats":{{"tx_count":3,"issuance_count":1,"issued_amount":5000,"burned_amount":200,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":9,"issuance_count":4,"issued_amount":999999,"burned_amount":888888,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}"#
    )
}

fn pegged_document() -> String {
    format!(
        r#"{{"asset_id":"{NATIVE_ASSET}","chain_stats":{{"tx_count":4,"peg_in_count":2,"peg_in_amount":1000,"peg_out_count":1,"peg_out_amount":200,"burn_count":1,"burned_amount":50}},"mempool_stats":{{"tx_count":8,"peg_in_count":3,"peg_in_amount":777777,"peg_out_count":2,"peg_out_amount":666666,"burn_count":1,"burned_amount":555555}}}}"#
    )
}

fn page(id: &str, document: &str) -> String {
    let path = format!("/assets/asset/{id}");
    asset_page_markup(&path, document, "[]")
        .unwrap_or_else(|| panic!("asset page did not open for {id}"))
}

fn assert_one_name_row(markup: &str, value: &str) {
    let row = format!("<tr><td>Name</td><td>{value}</td></tr>");
    assert_eq!(markup.matches(&row).count(), 1, "{markup}");
}

fn assert_no_name_row(markup: &str) {
    assert!(!markup.contains("<td>Name</td>"), "{markup}");
}

/// Issued name and ticker, through the asset page, are one Name row.
#[test]
fn issued_name_and_ticker_row_matches_the_page() {
    let document = issued_document(Field::Text("Test Asset"), Field::Text("TEST"));
    let path = format!("/assets/asset/{ISSUED}");
    let markup = asset_page_markup(&path, &document, "[]").expect("asset page");
    let direct = asset_markup(&document, "[]", true);
    assert_eq!(direct, markup);
    assert_one_name_row(&markup, "Test Asset (TEST)");
}

/// The name and the ticker are escaped, then joined. The raw pieces stay out.
#[test]
fn name_and_ticker_are_html_escaped_before_they_join() {
    let document = issued_document(Field::Text("a&b<c"), Field::Text("d>e"));
    let markup = page(ISSUED, &document);
    assert_one_name_row(&markup, "a&amp;b&lt;c (d&gt;e)");
    assert!(!markup.contains("a&b<c"), "{markup}");
    assert!(!markup.contains("d>e"), "{markup}");
}

/// A missing ticker is the name alone, with no parentheses.
#[test]
fn missing_ticker_paints_the_name_only() {
    let document = issued_document(Field::Text("OnlyName"), Field::Omit);
    let markup = page(ISSUED, &document);
    assert_one_name_row(&markup, "OnlyName");
    assert!(!markup.contains("OnlyName ("), "{markup}");
}

/// A blank ticker is the same as a missing ticker.
#[test]
fn blank_ticker_paints_the_name_only() {
    let document = issued_document(Field::Text("OnlyName"), Field::Text(""));
    let markup = page(ISSUED, &document);
    assert_one_name_row(&markup, "OnlyName");
    assert!(!markup.contains("OnlyName ("), "{markup}");
}

/// A ticker that is only whitespace does not add parentheses.
#[test]
fn whitespace_ticker_paints_the_name_only() {
    let document = issued_document(Field::Text("OnlyName"), Field::Text("   "));
    let markup = page(ISSUED, &document);
    assert_one_name_row(&markup, "OnlyName");
    assert!(!markup.contains("OnlyName ("), "{markup}");
}

/// A missing name key omits the Name row. The other issued rows stay.
#[test]
fn missing_name_omits_the_name_row() {
    let document = issued_document(Field::Omit, Field::Text("TEST"));
    let markup = page(ISSUED, &document);
    assert_no_name_row(&markup);
    assert!(markup.contains("<td>Precision</td>"), "{markup}");
}

/// A JSON null name omits the Name row.
#[test]
fn null_name_omits_the_name_row() {
    let document = issued_document(Field::Null, Field::Text("TEST"));
    let markup = page(ISSUED, &document);
    assert_no_name_row(&markup);
    assert!(markup.contains("<td>Precision</td>"), "{markup}");
}

/// An empty name omits the Name row.
#[test]
fn empty_name_omits_the_name_row() {
    let document = issued_document(Field::Text(""), Field::Text("TEST"));
    let markup = page(ISSUED, &document);
    assert_no_name_row(&markup);
    assert!(markup.contains("<td>Precision</td>"), "{markup}");
}

/// A name that is empty after trim omits the Name row.
#[test]
fn whitespace_name_omits_the_name_row() {
    let document = issued_document(Field::Text("   "), Field::Text("TEST"));
    let markup = page(ISSUED, &document);
    assert_no_name_row(&markup);
    assert!(markup.contains("<td>Precision</td>"), "{markup}");
}

/// A pegged asset has no Name row.
#[test]
fn pegged_asset_omits_the_name_row() {
    let document = pegged_document();
    let markup = page(NATIVE_ASSET, &document);
    assert_no_name_row(&markup);
    assert!(markup.contains("<td>Pegged in</td>"), "{markup}");
}

/// A non-liquid call stays empty and does not paint Name.
#[test]
fn non_liquid_markup_omits_the_name_row() {
    let document = issued_document(Field::Text("Test Asset"), Field::Text("TEST"));
    let markup = asset_markup(&document, "[]", false);
    assert!(markup.is_empty(), "{markup}");
    assert_no_name_row(&markup);
}
