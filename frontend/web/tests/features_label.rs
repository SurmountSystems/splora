//! Features label on the transaction details row from `GET /tx/:txid`.
//!
//! Angular `transaction-details.component.html` paints the word Features when
//! the network is not liquid and feature activation is on. Unconfirmed
//! `status.confirmed` is on. A confirmed mainnet height is on at the earliest
//! activation, rbf at 399701. A missing status, a confirmed height below that,
//! an empty body, and invalid JSON omit the label. The route is `tx/:id`.
//! It has no query string and no fragment.

use splora_web::{block_rows_markup, transaction_requests, transaction_row_markup};

const SCRIPT_HEX: &str = "FEATPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "FEATSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "FEATSIGASMDEAD";
const SCRIPT_ASM: &str = "FEATASMDEAD";
const WITNESS: &str = "FEATWITDEAD";
const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BLOCK_ID: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

const LABEL: &str = "<td class=\"features-label\">Features</td>";

fn status_field(status: &str) -> String {
    if status.is_empty() {
        String::new()
    } else {
        format!(r#","status":{status}"#)
    }
}

fn tx(status: &str) -> String {
    let status_field = status_field(status);
    format!(
        r#"{{"txid":"{TXID}","version":2,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":4294967295}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":40,"sigops":0,"fee":1{status_field}}}"#
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

fn assert_route_has_no_query_or_fragment() {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
}

fn assert_scripts_stay_off(markup: &str) {
    assert_absent(markup, SCRIPT_HEX);
    assert_absent(markup, SCRIPT_SIG_HEX);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, SCRIPT_ASM);
    assert_absent(markup, WITNESS);
    assert_absent(markup, "RBF");
    assert_absent(markup, "$");
    assert_absent(markup, "BTC");
}

/// Unconfirmed status paints the label once. The value is `status.confirmed`.
#[test]
fn unconfirmed_status_paints_the_features_label_once() {
    assert_route_has_no_query_or_fragment();
    let markup = transaction_row_markup(&tx(r#"{"confirmed":false}"#));
    assert_one(&markup, LABEL);
    assert_eq!(markup.matches("Features").count(), 2, "{markup}");
    assert!(
        markup.contains("<tr class=\"tx-row\">"),
        "the label replaced the transaction row: {markup}"
    );
    assert_scripts_stay_off(&markup);
}

/// Confirmed mainnet height at rbf activation still paints the label once.
#[test]
fn confirmed_height_at_rbf_activation_paints_the_label_once() {
    let markup = transaction_row_markup(&tx(
        r#"{"confirmed":true,"block_height":399701,"block_hash":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd","block_time":1600000000}"#,
    ));
    assert_one(&markup, LABEL);
    assert_eq!(markup.matches("Features").count(), 2, "{markup}");
    assert_one(&markup, "<td class=\"block-time\">1600000000</td>");
    assert_scripts_stay_off(&markup);
}

/// A confirmed height below every mainnet activation omits the label.
#[test]
fn confirmed_height_before_activation_omits_the_label() {
    let markup = transaction_row_markup(&tx(
        r#"{"confirmed":true,"block_height":399700,"block_hash":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}"#,
    ));
    assert!(markup.contains("<tr class=\"tx-row\">"), "{markup}");
    assert_absent(&markup, "Features");
    assert_absent(&markup, "features-label");
    assert_scripts_stay_off(&markup);
}

/// No status object omits the label and keeps the transaction row.
#[test]
fn missing_status_omits_the_label() {
    let markup = transaction_row_markup(&tx(""));
    assert!(markup.contains("<tr class=\"tx-row\">"), "{markup}");
    assert_absent(&markup, "Features");
    assert_absent(&markup, "features-label");
    assert_scripts_stay_off(&markup);
}

/// Empty and invalid bodies do not invent the label.
#[test]
fn empty_and_invalid_json_omit_the_label() {
    assert_absent(&transaction_row_markup(""), "Features");
    assert_absent(&transaction_row_markup("not-json"), "Features");
    assert_absent(&transaction_row_markup("[]"), "Features");
}

/// The block list row does not gain this transaction-details label.
#[test]
fn block_list_row_does_not_gain_the_label() {
    let block = format!(
        r#"{{"id":"{BLOCK_ID}","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    );
    let txs = tx(r#"{"confirmed":false}"#);
    let markup = block_rows_markup(&block, &format!("[{txs}]"));
    assert!(markup.contains("<tr class=\"tx-row\">"), "{markup}");
    assert_absent(&markup, "Features");
    assert_absent(&markup, "features-label");
}

/// Whole line `Features` from no-query `GET /tx/:txid`.
///
/// The label cell stays. This line is the label text alone.
#[test]
fn features_paints_features_as_its_own_line() {
    const LINE: &str = "Features";
    let markup = transaction_row_markup(&tx(r#"{"confirmed":false}"#));
    assert!(
        markup.lines().any(|line| line == LINE),
        "Features is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "Features was not painted once: {markup}"
    );
}
