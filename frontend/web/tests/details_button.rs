//! Details button on the loaded transaction from `GET /tx/:txid`.
//!
//! Angular paints one button in the subtitle block beside Inputs & Outputs.
//! Its visible text is Details. The click handler is `toggleDetailsFromTxPage`,
//! which opens script rows. The sibling Show diagram control stays off.
//! The load is `GET /tx/:txid` with no query and no fragment. No diagram,
//! graph, chart, or skeleton.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const WORD: &str = "Details";
const BUTTON: &str =
    "<button type=\"button\" class=\"btn btn-outline-info btn-sm\">Details</button>";

fn tx() -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa","value":5}},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"76a914cc88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2","value":4}}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

/// Text nodes between tags. Attribute values are not visible text.
fn visible_texts(markup: &str) -> Vec<String> {
    let mut texts = Vec::new();
    let mut rest = markup;
    while let Some(start) = rest.find('>') {
        rest = &rest[start + 1..];
        if rest.is_empty() || rest.starts_with('<') {
            continue;
        }
        let end = rest.find('<').unwrap_or(rest.len());
        let text = &rest[..end];
        rest = &rest[end..];
        if text.trim().is_empty() {
            continue;
        }
        texts.push(text.to_string());
    }
    texts
}

fn assert_exact_details(markup: &str) {
    let texts = visible_texts(markup);
    assert!(
        texts.iter().any(|text| text == WORD),
        "Details is absent: {markup}"
    );
    assert_eq!(
        texts.iter().filter(|text| text.as_str() == WORD).count(),
        1,
        "Details was not painted once: {markup}"
    );
    assert!(
        texts
            .iter()
            .filter(|text| text.contains(WORD))
            .all(|text| text.trim() == WORD),
        "Details was painted inside a longer sentence: {markup}"
    );
}

#[test]
fn loaded_tx_page_paints_details_once() {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));

    let markup = transaction_row_markup(&tx());
    assert!(
        markup.contains("<h2>Inputs & Outputs</h2>"),
        "the transaction page was not rendered: {markup}"
    );
    assert_exact_details(&markup);
    assert_eq!(
        markup.matches(BUTTON).count(),
        1,
        "the Details button was not painted once: {markup}"
    );
    assert!(
        !BUTTON.contains("badge-ord"),
        "Details reused the Inscription class"
    );
    let heading_at = markup.find("<h2>Inputs & Outputs</h2>").expect("heading");
    let button_at = markup.find(BUTTON).expect("Details button");
    let table_at = markup.find("<table class=\"tx-rows\"").expect("tx table");
    assert!(
        heading_at < button_at && button_at < table_at,
        "Details was not beside Inputs & Outputs and before the table: {markup}"
    );
    for needle in [
        "Show diagram",
        "skeleton",
        "chart",
        "tx-bowtie",
        "toggleGraph",
        "Transaction Details",
        "Details of this transaction",
    ] {
        assert!(!markup.contains(needle), "{needle} was painted: {markup}");
    }

    let empty = transaction_row_markup("");
    assert!(
        !visible_texts(&empty).iter().any(|text| text == WORD),
        "an empty body painted Details: {empty}"
    );
}

/// Whole line beside the Details button from `GET /tx/:txid`.
///
/// A line equal to Details passes. The button stays. An empty body omits
/// the line. Show diagram stays off.
#[test]
fn loaded_tx_page_paints_details_as_its_own_line() {
    const LINE: &str = "Details";
    let markup = transaction_row_markup(&tx());
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert_eq!(
        markup.matches(BUTTON).count(),
        1,
        "the Details button was dropped: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line.starts_with("Details ")),
        "Details was painted as a longer sentence: {markup}"
    );
    let empty = transaction_row_markup("");
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty body painted {LINE}: {empty}"
    );
}
