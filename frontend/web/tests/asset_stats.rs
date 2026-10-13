//! Chain stats on the Leptos asset page.
//!
//! `GET /asset/:id` is `asset_path`. The GPUI screen paints precision, the
//! issuer domain, the issuance transaction, and the chain amounts from that
//! document. A blinded issuance stays the word Confidential. A pegged asset
//! has no name. The integers stay the chain stats. This page does not scale
//! them by precision and does not append a ticker. This test does not mark a
//! Cypress row ported.

use splora_web::{asset_markup, asset_page_markup, asset_requests};

const NATIVE_ASSET: &str = "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
const ISSUED: &str = "ac3e0ff248c5051ffd61e00155b7122e5ebc04fd397a0ecbdd4f4e4a56232926";
const ISSUANCE_TX: &str = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
const PREV_TX: &str = "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef";
const REISSUE: &str = "1111111111111111111111111111111111111111111111111111111111111111";

const STATS_TABLE: &str = "<table class=\"table table-borderless table-striped\">";

fn issued_document(
    precision: Option<u8>,
    domain: Option<&str>,
    issuance_tx: &str,
    chain_issued: u64,
    chain_burned: u64,
    chain_blinded: bool,
    mempool_issued: u64,
    mempool_burned: u64,
    mempool_blinded: bool,
) -> String {
    let precision_field = match precision {
        Some(precision) => format!(r#""precision":{precision},"#),
        None => String::new(),
    };
    let entity_field = match domain {
        Some(domain) => format!(r#""entity":{{"domain":"{domain}"}},"#),
        None => String::new(),
    };
    format!(
        r#"{{"asset_id":"{ISSUED}","name":"Test Asset","ticker":"TEST",{precision_field}{entity_field}"contract":null,"issuance_txin":{{"txid":"{issuance_tx}","vin":0}},"issuance_prevout":{{"txid":"{PREV_TX}","vout":1}},"reissuance_token":"{REISSUE}","status":{{"confirmed":true}},"chain_stats":{{"tx_count":3,"issuance_count":1,"issued_amount":{chain_issued},"burned_amount":{chain_burned},"has_blinded_issuances":{chain_blinded},"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":9,"issuance_count":4,"issued_amount":{mempool_issued},"burned_amount":{mempool_burned},"has_blinded_issuances":{mempool_blinded},"burned_reissuance_tokens":0}}}}"#
    )
}

fn pegged_document(
    peg_in: u64,
    peg_out: u64,
    burned: u64,
    mempool_peg_in: u64,
    mempool_peg_out: u64,
    mempool_burned: u64,
) -> String {
    format!(
        r#"{{"asset_id":"{NATIVE_ASSET}","chain_stats":{{"tx_count":4,"peg_in_count":2,"peg_in_amount":{peg_in},"peg_out_count":1,"peg_out_amount":{peg_out},"burn_count":1,"burned_amount":{burned}}},"mempool_stats":{{"tx_count":8,"peg_in_count":3,"peg_in_amount":{mempool_peg_in},"peg_out_count":2,"peg_out_amount":{mempool_peg_out},"burn_count":1,"burned_amount":{mempool_burned}}}}}"#
    )
}

fn page(id: &str, document: &str) -> String {
    let path = format!("/assets/asset/{id}");
    asset_page_markup(&path, document, "[]")
        .unwrap_or_else(|| panic!("asset page did not open for {id}"))
}

fn stats_table(markup: &str) -> &str {
    let at = markup
        .find(STATS_TABLE)
        .unwrap_or_else(|| panic!("no chain-stats table: {markup}"));
    let rest = &markup[at..];
    let end = rest
        .find("</table>")
        .map(|offset| offset + "</table>".len())
        .unwrap_or(rest.len());
    &rest[..end]
}

fn cell<'a>(table: &'a str, label: &str) -> &'a str {
    let needle = format!("<tr><td>{label}</td><td>");
    let rest = table
        .split_once(&needle)
        .unwrap_or_else(|| panic!("missing {label} in {table}"))
        .1;
    rest.split_once("</td>")
        .unwrap_or_else(|| panic!("unclosed {label} in {table}"))
        .0
}

fn assert_absent(markup: &str, label: &str) {
    let needle = format!("<td>{label}</td>");
    assert!(
        !markup.contains(&needle),
        "{label} should be hidden: {markup}"
    );
}

fn assert_order(table: &str, labels: &[&str]) {
    let mut last = 0;
    for label in labels {
        let needle = format!("<td>{label}</td>");
        let found = table[last..]
            .find(&needle)
            .unwrap_or_else(|| panic!("{label} is out of order in {table}"));
        last += found + needle.len();
    }
}

fn assert_no_scaled_amount(markup: &str) {
    assert!(!markup.contains("0.00100000"), "{markup}");
    assert!(!markup.contains("LBTC"), "{markup}");
    assert!(!markup.contains("L-BTC"), "{markup}");
}

fn assert_plain_asset_paths(id: &str) {
    let reads = asset_requests(true, id);
    let path = format!("/asset/{id}");
    assert!(
        reads
            .iter()
            .any(|req| req.method == "GET" && req.path == path),
        "asset stats did not request asset_path: {reads:?}"
    );
    assert!(
        reads
            .iter()
            .all(|req| !req.path.contains('?') && !req.path.contains('#')),
        "{reads:?}"
    );
    assert!(
        reads
            .iter()
            .all(|req| !req.path.contains("/assets/registry/search")),
        "asset stats must not search the registry: {reads:?}"
    );
    assert!(asset_requests(false, id).is_empty());
    let src = include_str!("../src/screens.rs");
    let start = src.find("pub fn asset_requests(").expect("asset_requests");
    let body = &src[start..];
    assert!(
        body.contains("asset_path("),
        "asset_requests does not call asset_path: {body}"
    );
    assert!(
        !body.contains("assets_registry_search_path"),
        "asset_requests searches the registry: {body}"
    );
}

/// An issued asset paints precision, the issuer, the issuance transaction,
/// and the chain issued, burned, and circulating integers.
#[test]
fn issued_asset_paints_precision_issuer_issuance_and_chain_amounts() {
    let document = issued_document(
        Some(2),
        Some("example.com"),
        ISSUANCE_TX,
        5000,
        200,
        false,
        999_999,
        888_888,
        false,
    );
    let markup = page(ISSUED, &document);
    let direct = asset_markup(&document, "[]", true);
    assert_eq!(direct, markup);
    let table = stats_table(&markup);
    assert_eq!(markup.matches(STATS_TABLE).count(), 1, "{markup}");
    assert_order(
        table,
        &[
            "Precision",
            "Issuer",
            "Issuance TX",
            "Issued amount",
            "Burned amount",
            "Circulating amount",
        ],
    );
    assert_eq!(cell(table, "Precision"), "2");
    assert_eq!(cell(table, "Issuer"), "example.com");
    assert_eq!(
        cell(table, "Issuance TX"),
        format!("<a href=\"/tx/{ISSUANCE_TX}\">{ISSUANCE_TX}</a>")
    );
    assert_eq!(cell(table, "Issued amount"), "5000");
    assert_eq!(cell(table, "Burned amount"), "200");
    assert_eq!(cell(table, "Circulating amount"), "4800");
    assert!(markup.contains("Test Asset"), "{markup}");
    assert!(markup.contains("TEST"), "{markup}");
    assert!(!markup.contains("999999"), "{markup}");
    assert!(!markup.contains("888888"), "{markup}");
    assert!(!markup.contains("1004999"), "{markup}");
    assert!(!markup.contains("889088"), "{markup}");
    assert!(!markup.contains("115911"), "{markup}");
    assert!(!markup.contains("Confidential"), "{markup}");
    assert_no_scaled_amount(&markup);
    assert!(
        markup.contains("<h2>Issuance and Burn Transactions</h2>"),
        "{markup}"
    );
    assert_eq!(markup.matches("<h2>").count(), 1, "{markup}");
    let stats_at = markup.find(STATS_TABLE).expect("table");
    let heading_at = markup.find("<h2>").expect("heading");
    assert!(stats_at < heading_at, "{markup}");
    assert!(asset_markup(&document, "[]", false).is_empty());
    assert_plain_asset_paths(ISSUED);
}

/// A blinded issuance keeps Confidential on the amount rows and does not
/// paint the integers. Chain or mempool `has_blinded_issuances` is enough.
#[test]
fn blinded_issuance_paints_confidential_not_the_integers() {
    for (chain_blinded, mempool_blinded) in [(true, false), (false, true), (true, true)] {
        let document = issued_document(
            Some(2),
            Some("example.com"),
            ISSUANCE_TX,
            424_242,
            111,
            chain_blinded,
            999_999,
            888_888,
            mempool_blinded,
        );
        let markup = page(ISSUED, &document);
        let table = stats_table(&markup);
        assert_eq!(cell(table, "Issued amount"), "Confidential", "{markup}");
        assert_eq!(cell(table, "Burned amount"), "Confidential", "{markup}");
        assert_eq!(
            cell(table, "Circulating amount"),
            "Confidential",
            "{markup}"
        );
        assert!(!markup.contains("424242"), "{markup}");
        assert!(!markup.contains("111"), "{markup}");
        assert!(!markup.contains("999999"), "{markup}");
        assert!(!markup.contains("888888"), "{markup}");
        assert!(!markup.contains("424131"), "{markup}");
        assert_no_scaled_amount(&markup);
        assert_eq!(cell(table, "Precision"), "2");
        assert_eq!(cell(table, "Issuer"), "example.com");
    }
}

/// A zero chain issued amount hides issued and circulating. Burned still
/// shows the chain integer. Mempool issued does not bring those rows back.
#[test]
fn zero_issued_amount_hides_issued_and_circulating() {
    let document = issued_document(
        Some(2),
        Some("example.com"),
        ISSUANCE_TX,
        0,
        40,
        false,
        999_999,
        888_888,
        false,
    );
    let markup = page(ISSUED, &document);
    let table = stats_table(&markup);
    assert_absent(&markup, "Issued amount");
    assert_absent(&markup, "Circulating amount");
    assert_eq!(cell(table, "Burned amount"), "40");
    assert!(!markup.contains("999999"), "{markup}");
    assert!(!markup.contains("888888"), "{markup}");
    assert_eq!(cell(table, "Precision"), "2");
    assert_eq!(cell(table, "Issuer"), "example.com");
    assert!(markup.contains(ISSUANCE_TX), "{markup}");
}

/// Burned above issued stays a circulating zero. The subtraction does not wrap.
#[test]
fn circulating_amount_saturates_at_zero() {
    let document = issued_document(
        Some(2),
        Some("example.com"),
        ISSUANCE_TX,
        100,
        250,
        false,
        0,
        0,
        false,
    );
    let markup = page(ISSUED, &document);
    let table = stats_table(&markup);
    assert_eq!(cell(table, "Issued amount"), "100");
    assert_eq!(cell(table, "Burned amount"), "250");
    assert_eq!(cell(table, "Circulating amount"), "0");
}

/// A zero burned amount still has its row. Circulating is the chain issued amount.
#[test]
fn zero_burned_amount_still_shows() {
    let document = issued_document(
        Some(8),
        Some("example.com"),
        ISSUANCE_TX,
        100_000_000,
        0,
        false,
        1,
        1,
        false,
    );
    let markup = page(ISSUED, &document);
    let table = stats_table(&markup);
    assert_eq!(cell(table, "Precision"), "8");
    assert_eq!(cell(table, "Issued amount"), "100000000");
    assert_eq!(cell(table, "Burned amount"), "0");
    assert_eq!(cell(table, "Circulating amount"), "100000000");
    assert!(!markup.contains("100000000 TEST"), "{markup}");
    assert!(!markup.contains("1.00000000"), "{markup}");
    assert!(!markup.contains("100000001"), "{markup}");
    assert_no_scaled_amount(&markup);
}

/// A missing precision, an empty issuer domain, and an empty issuance
/// transaction are not blank rows.
#[test]
fn blank_precision_issuer_and_issuance_are_omitted() {
    let document = issued_document(None, Some(""), "", 5000, 200, false, 0, 0, false);
    let markup = page(ISSUED, &document);
    let table = stats_table(&markup);
    assert_absent(&markup, "Precision");
    assert_absent(&markup, "Issuer");
    assert_absent(&markup, "Issuance TX");
    assert_eq!(cell(table, "Issued amount"), "5000");
    assert_eq!(cell(table, "Burned amount"), "200");
    assert_eq!(cell(table, "Circulating amount"), "4800");
    let no_entity = issued_document(None, None, ISSUANCE_TX, 5000, 200, false, 0, 0, false);
    let markup = page(ISSUED, &no_entity);
    assert_absent(&markup, "Issuer");
    assert!(markup.contains(ISSUANCE_TX), "{markup}");
}

/// A pegged asset document has no name. Pegged in, pegged out, burned, and
/// circulating come from `chain_stats`.
#[test]
fn pegged_asset_without_a_name_paints_pegged_and_circulating_amounts() {
    let document = pegged_document(1000, 200, 50, 777_777, 666_666, 555_555);
    let markup = page(NATIVE_ASSET, &document);
    let table = stats_table(&markup);
    assert_eq!(markup.matches(STATS_TABLE).count(), 1, "{markup}");
    assert_order(
        table,
        &[
            "Pegged in",
            "Pegged out",
            "Burned amount",
            "Circulating amount",
        ],
    );
    assert_eq!(cell(table, "Pegged in"), "1000");
    assert_eq!(cell(table, "Pegged out"), "200");
    assert_eq!(cell(table, "Burned amount"), "50");
    assert_eq!(cell(table, "Circulating amount"), "750");
    assert_absent(&markup, "Issued amount");
    assert_absent(&markup, "Issuance TX");
    assert_absent(&markup, "Precision");
    assert_absent(&markup, "Issuer");
    assert!(!markup.contains("Confidential"), "{markup}");
    assert!(!markup.contains("777777"), "{markup}");
    assert!(!markup.contains("666666"), "{markup}");
    assert!(!markup.contains("555555"), "{markup}");
    assert!(!markup.contains("778777"), "{markup}");
    assert!(!markup.contains("asset name"), "{markup}");
    assert!(!markup.contains("ticker "), "{markup}");
    assert!(markup.contains(NATIVE_ASSET), "{markup}");
    assert!(
        markup.contains("<h2>Peg In/Out and Burn Transactions</h2>"),
        "{markup}"
    );
    assert!(
        !markup.contains("Issuance and Burn Transactions"),
        "{markup}"
    );
    assert_no_scaled_amount(&markup);
    assert_plain_asset_paths(NATIVE_ASSET);
}

/// A zero pegged-in amount hides pegged in and circulating. A zero pegged-out
/// amount hides that row. Burned still shows the chain integer.
#[test]
fn zero_pegged_amounts_are_hidden_and_burned_still_shows() {
    let neither = pegged_document(0, 0, 50, 1000, 200, 9);
    let markup = page(NATIVE_ASSET, &neither);
    let table = stats_table(&markup);
    assert_absent(&markup, "Pegged in");
    assert_absent(&markup, "Pegged out");
    assert_absent(&markup, "Circulating amount");
    assert_eq!(cell(table, "Burned amount"), "50");
    assert!(!markup.contains("1000"), "{markup}");
    assert!(!markup.contains("200"), "{markup}");
    let peg_out_zero = pegged_document(1000, 0, 50, 0, 666_666, 0);
    let markup = page(NATIVE_ASSET, &peg_out_zero);
    let table = stats_table(&markup);
    assert_eq!(cell(table, "Pegged in"), "1000");
    assert_absent(&markup, "Pegged out");
    assert_eq!(cell(table, "Burned amount"), "50");
    assert_eq!(cell(table, "Circulating amount"), "950");
    assert!(!markup.contains("666666"), "{markup}");
    let wrapped = pegged_document(100, 80, 50, 0, 0, 0);
    let markup = page(NATIVE_ASSET, &wrapped);
    let table = stats_table(&markup);
    assert_eq!(cell(table, "Circulating amount"), "0");
}

/// A zero burned peg amount still shows. It does not hide the row.
#[test]
fn zero_pegged_burn_still_shows() {
    let document = pegged_document(1000, 200, 0, 0, 0, 555_555);
    let markup = page(NATIVE_ASSET, &document);
    let table = stats_table(&markup);
    assert_eq!(cell(table, "Burned amount"), "0");
    assert_eq!(cell(table, "Circulating amount"), "800");
    assert!(!markup.contains("555555"), "{markup}");
}

/// A name and a ticker, with no chain stats, stay the open page. No amount
/// is invented.
#[test]
fn name_and_ticker_only_does_not_invent_an_amount() {
    let id = "34".repeat(32);
    let document = format!(r#"{{"asset_id":"{id}","name":"Liquid AUD","ticker":"L-AUD"}}"#);
    let markup = page(&id, &document);
    assert!(markup.contains(&id), "{markup}");
    assert!(markup.contains("Liquid AUD"), "{markup}");
    assert!(markup.contains("L-AUD"), "{markup}");
    assert!(!markup.contains(STATS_TABLE), "{markup}");
    assert!(!markup.contains("amount"), "{markup}");
    assert!(!markup.contains("Pegged"), "{markup}");
    assert!(!markup.contains("Precision"), "{markup}");
    assert!(!markup.contains("Issuer"), "{markup}");
    assert!(!markup.contains("Confidential"), "{markup}");
    assert!(!markup.contains("Issuance TX"), "{markup}");
    assert_no_scaled_amount(&markup);
    assert!(
        markup.contains("<h2>Issuance and Burn Transactions</h2>"),
        "{markup}"
    );
    assert_plain_asset_paths(&id);
}

/// Chain blinded with a zero issued amount still hides issued and circulating.
/// Burned is Confidential, not the integer.
#[test]
fn blinded_zero_issued_hides_issued_and_paints_burned_confidential() {
    let document = issued_document(
        Some(2),
        Some("example.com"),
        ISSUANCE_TX,
        0,
        111,
        false,
        424_242,
        333,
        true,
    );
    let markup = page(ISSUED, &document);
    let table = stats_table(&markup);
    assert_absent(&markup, "Issued amount");
    assert_absent(&markup, "Circulating amount");
    assert_eq!(cell(table, "Burned amount"), "Confidential");
    assert!(!markup.contains("111"), "{markup}");
    assert!(!markup.contains("424242"), "{markup}");
    assert!(!markup.contains("333"), "{markup}");
}
