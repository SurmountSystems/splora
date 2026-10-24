//! Whole line `Invalid Bitcoin address` on a failed address load.
//!
//! GPUI requires one line whose entire text equals that sentence. The italic
//! stays `(Invalid Bitcoin address)` inside the http-error div. That
//! parenthesized text is not the whole line. The `Error loading address data.`
//! line stays. A parsed address that returns stats omits the sentence. A
//! failed load does not paint a line that starts with `Type `.

use splora_web::{address_rows_markup, indexer_failure_text};

const LINE: &str = "Invalid Bitcoin address";
const ERROR_LINE: &str = "Error loading address data.";
const ITALIC: &str = "<i class=\"small\">(Invalid Bitcoin address)</i>";
const ADDR: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn failed_markup() -> String {
    let id = "not-an-address";
    let path = format!("/address/{id}");
    let failed = indexer_failure_text("GET", &path, LINE);
    address_rows_markup(&failed, "", "")
}

#[test]
fn failed_address_load_paints_invalid_bitcoin_address_as_its_own_line() {
    let markup = failed_markup();
    assert!(
        markup.contains(ITALIC),
        "the italic HTTP body dropped: {markup}"
    );
    assert_eq!(
        markup
            .split('\n')
            .filter(|line| *line == ERROR_LINE)
            .count(),
        1,
        "the Error loading address data. line left: {markup}"
    );
    assert_eq!(
        markup.split('\n').filter(|line| *line == LINE).count(),
        1,
        "no line equals {LINE}: {markup}"
    );
    let italic_at = markup.find(ITALIC).expect("italic");
    let body_at = markup.find(LINE).expect("indexer body");
    assert!(
        italic_at < body_at && body_at < italic_at + ITALIC.len(),
        "the first Invalid Bitcoin address is not inside the italic: {markup}"
    );
    let own_at = markup.find(&format!("\n{LINE}\n")).expect("own line");
    assert!(
        italic_at < own_at,
        "the whole line is not after the italic parentheses: {markup}"
    );
    assert!(
        !markup.split('\n').any(|line| line.starts_with("Type ")),
        "a failed address load painted a type badge: {markup}"
    );
    assert!(
        !markup.contains("P2PKH"),
        "a failed address load showed the P2PKH badge: {markup}"
    );

    let painted = address_rows_markup(&address_body(ADDR), "", "");
    assert!(
        !painted.split('\n').any(|line| line == LINE),
        "a parsed address that returns stats painted {LINE}: {painted}"
    );
    assert!(
        !painted.contains(LINE),
        "a parsed address that returns stats painted {LINE}: {painted}"
    );

    let bare = indexer_failure_text("GET", "/address/not-an-address", "");
    let bare_markup = address_rows_markup(&bare, "", "");
    assert!(
        !bare_markup.split('\n').any(|line| line == LINE),
        "an empty HTTP body painted {LINE}: {bare_markup}"
    );
    assert_eq!(
        bare_markup.matches("\n\n").count(),
        0,
        "an empty HTTP body pushed a blank line: {bare_markup}"
    );
    assert!(!address_rows_markup("not-json", "", "").contains(LINE));
}
