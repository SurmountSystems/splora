//! Whole line `Error loading address data.` on a failed address load.
//!
//! GPUI requires one line whose entire text equals that sentence. The span
//! stays inside the http-error div. The italic body stays
//! `(Invalid Bitcoin address)`. The whole line equal to `Invalid Bitcoin address`
//! is a separate check. A parsed address that returns stats omits this
//! error line. `Error loading block data.` stays the block sentence.

use splora_web::{address_rows_markup, indexer_failure_text};

const LINE: &str = "Error loading address data.";
const BLOCK_SENTENCE: &str = "Error loading block data.";
const BODY: &str = "Invalid Bitcoin address";
const SPAN: &str = "<span>Error loading address data.</span>";
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
    let failed = indexer_failure_text("GET", &path, BODY);
    address_rows_markup(&failed, "", "")
}

#[test]
fn failed_address_load_paints_error_loading_address_data_as_its_own_line() {
    let markup = failed_markup();
    assert!(
        markup.contains(SPAN),
        "the span is not element text: {markup}"
    );
    assert!(
        markup.contains("<div class=\"http-error\">"),
        "the span is not in the http-error paragraph: {markup}"
    );
    assert!(
        markup.contains(ITALIC),
        "the italic HTTP body dropped: {markup}"
    );
    assert_eq!(
        markup.split('\n').filter(|line| *line == LINE).count(),
        1,
        "no line equals {LINE}: {markup}"
    );
    assert!(
        !markup.contains(BLOCK_SENTENCE),
        "the address page painted the block sentence: {markup}"
    );
    assert!(
        !markup.split('\n').any(|line| line == BLOCK_SENTENCE),
        "the address page painted the block sentence as its own line: {markup}"
    );

    let painted = address_rows_markup(&address_body(ADDR), "", "");
    assert!(
        !painted.split('\n').any(|line| line == LINE),
        "a parsed address that returns stats painted {LINE}: {painted}"
    );
    assert!(
        !painted.contains(LINE),
        "a parsed address that returns stats painted the error span: {painted}"
    );
    assert!(!painted.contains(BLOCK_SENTENCE), "{painted}");
}
