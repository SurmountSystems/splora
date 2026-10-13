//! Confirmed UTXO count on the address page is chain funded count minus
//! chain spent count.
//!
//! Mempool counts are not that line. A missing chain count is not zero.
//! The count is independent of the confirmed balance.

use splora_web::{address_fields_text, address_rows_markup};

fn paint(body: &str) -> (String, String) {
    (
        address_rows_markup(body, "", ""),
        address_fields_text(body, "", ""),
    )
}

fn assert_utxos(markup: &str, text: &str, count: &str) {
    let phrase = format!("Confirmed UTXOs {count}");
    assert!(markup.contains(&phrase), "{markup}");
    assert!(text.contains(&phrase), "{text}");
    assert!(!phrase.contains(','));
}

fn assert_no_utxos(markup: &str, text: &str) {
    assert!(!markup.contains("Confirmed UTXOs"), "{markup}");
    assert!(!text.contains("Confirmed UTXOs"), "{text}");
    assert!(!markup.contains("Confirmed UTXOs 0"), "{markup}");
    assert!(!text.contains("Confirmed UTXOs 0"), "{text}");
}

/// Same address body as `address_stats_json` in `screen_fields.rs`.
/// Chain funded count 4, spent count 1, funded sum 5000, spent sum 1000.
/// Mempool funded count 1 and spent count 0 are not the confirmed count.
fn address_stats_json() -> String {
    r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":20,"spent_txo_sum":0}}"#
        .to_string()
}

fn chain_counts(funded: u64, spent: u64) -> String {
    format!(
        r#"{{"address":"bc1qexample","chain_stats":{{"tx_count":4,"funded_txo_count":{funded},"spent_txo_count":{spent},"funded_txo_sum":5000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":0}}}}"#
    )
}

fn omitted_chain_count(funded_key: bool) -> String {
    let chain = if funded_key {
        r#""tx_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000"#
    } else {
        r#""tx_count":4,"funded_txo_count":4,"funded_txo_sum":5000,"spent_txo_sum":1000"#
    };
    format!(
        r#"{{"address":"bc1qexample","chain_stats":{{{chain}}},"mempool_stats":{{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":0}}}}"#
    )
}

#[test]
fn confirmed_utxos_follow_confirmed_balance_from_chain_counts() {
    let (markup, text) = paint(&address_stats_json());
    assert_utxos(&markup, &text, "3");
    let balance = "Confirmed balance 4000";
    assert!(markup.contains(balance), "{markup}");
    assert!(text.contains(balance), "{text}");
    let utxos = "Confirmed UTXOs 3";
    let text_balance = text.find(balance).expect("balance");
    let text_utxos = text.find(utxos).expect("utxos");
    assert!(
        text_utxos > text_balance,
        "utxo words do not follow the balance in {text}"
    );
    assert!(
        markup.find(utxos).expect("markup utxos") > markup.find(balance).expect("markup balance"),
        "utxo words do not follow the balance in {markup}"
    );
    assert!(
        !markup.contains("Confirmed UTXOs 1"),
        "mempool count difference painted: {markup}"
    );
    assert!(
        !text.contains("Confirmed UTXOs 1"),
        "mempool count difference painted: {text}"
    );
}

#[test]
fn confirmed_utxos_zero_when_both_chain_counts_are_zero() {
    let (markup, text) = paint(&chain_counts(0, 0));
    assert_utxos(&markup, &text, "0");
    assert!(
        !markup.contains("Confirmed UTXOs 8"),
        "mempool count difference painted: {markup}"
    );
    assert!(
        !text.contains("Confirmed UTXOs 8"),
        "mempool count difference painted: {text}"
    );
}

#[test]
fn omitted_funded_txo_count_does_not_paint_confirmed_utxos() {
    let (markup, text) = paint(&omitted_chain_count(true));
    assert_no_utxos(&markup, &text);
}

#[test]
fn omitted_spent_txo_count_does_not_paint_confirmed_utxos() {
    let (markup, text) = paint(&omitted_chain_count(false));
    assert_no_utxos(&markup, &text);
}

#[test]
fn confirmed_utxos_paint_when_chain_sums_are_omitted() {
    let body = r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1},"mempool_stats":{"tx_count":1,"funded_txo_count":9,"spent_txo_count":2}}"#;
    let (markup, text) = paint(body);
    assert_utxos(&markup, &text, "3");
    assert!(!markup.contains("Confirmed balance"), "{markup}");
    assert!(!text.contains("Confirmed balance"), "{text}");
    assert!(!markup.contains("Confirmed UTXOs 7"), "{markup}");
    assert!(!text.contains("Confirmed UTXOs 7"), "{text}");
}

#[test]
fn confirmed_utxos_are_negative_when_spent_count_exceeds_funded() {
    let (markup, text) = paint(&chain_counts(1, 4));
    assert_utxos(&markup, &text, "-3");
    assert!(!markup.contains("Confirmed UTXOs 0"), "{markup}");
    assert!(!text.contains("Confirmed UTXOs 0"), "{text}");
}
