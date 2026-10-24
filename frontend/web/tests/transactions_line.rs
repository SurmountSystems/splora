//! Whole line `Transactions 5` on the address page and the scripthash page.
//!
//! GPUI `address_screen_paints_transactions_5` requires one line whose
//! entire text equals `Transactions 5`. That number is the chain tx count
//! plus the mempool tx count. The transactions paragraph stays and is the
//! longer line that still contains those words. A missing mempool tx count
//! omits the sentence, so the markup has no such line and does not contain
//! the word Transactions. The chain count alone is not this line, and the
//! mempool count alone is not this line. Address markup and scripthash
//! markup are separate strings. Each has count 1.

use splora_web::{address_rows_markup, scripthash_markup};

const LINE: &str = "Transactions 5";

fn stats_object(
    tx_count: Option<u64>,
    funded: u64,
    spent: u64,
    funded_sum: u64,
    spent_sum: u64,
) -> String {
    let tx = match tx_count {
        Some(count) => format!(r#""tx_count":{count},"#),
        None => String::new(),
    };
    format!(
        r#"{{{tx}"funded_txo_count":{funded},"spent_txo_count":{spent},"funded_txo_sum":{funded_sum},"spent_txo_sum":{spent_sum}}}"#
    )
}

fn page_body(key: &str, value: &str, chain_tx: Option<u64>, mempool_tx: Option<u64>) -> String {
    let chain = stats_object(chain_tx, 4, 1, 5000, 1000);
    let mempool = stats_object(mempool_tx, 2, 1, 20, 7);
    format!(r#"{{"{key}":"{value}","chain_stats":{chain},"mempool_stats":{mempool}}}"#)
}

fn assert_one_whole_line(markup: &str) {
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    let paragraph = format!("<p class=\"transactions\">{LINE}</p>");
    assert!(
        markup.contains(&paragraph),
        "the transactions paragraph was dropped or lost the words: {markup}"
    );
    assert_eq!(
        markup.matches("<p class=\"transactions\">").count(),
        1,
        "the transactions paragraph was duplicated: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line.contains(LINE) && line != LINE),
        "the longer line dropped {LINE}: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Transactions 4"),
        "the chain tx count was painted alone: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Transactions 1"),
        "the mempool tx count was painted alone: {markup}"
    );
}

#[test]
fn address_and_scripthash_paint_transactions_as_its_own_line() {
    let address = address_rows_markup(
        &page_body("address", "bc1qexample", Some(4), Some(1)),
        "",
        "",
    );
    let hash_markup = scripthash_markup(&page_body("scripthash", "deadbeef", Some(4), Some(1)));
    assert!(address.contains("bc1qexample"), "{address}");
    assert!(
        hash_markup.contains("<p class=\"scripthash\">deadbeef</p>"),
        "{hash_markup}"
    );
    assert_one_whole_line(&address);
    assert_one_whole_line(&hash_markup);
    assert_eq!(
        address.lines().filter(|line| *line == LINE).count(),
        1,
        "address markup count is not 1: {address}"
    );
    assert_eq!(
        hash_markup.lines().filter(|line| *line == LINE).count(),
        1,
        "scripthash markup count is not 1: {hash_markup}"
    );

    let address_omitted =
        address_rows_markup(&page_body("address", "bc1qexample", Some(4), None), "", "");
    let hash_omitted = scripthash_markup(&page_body("scripthash", "deadbeef", Some(4), None));
    for markup in [&address_omitted, &hash_omitted] {
        assert!(
            !markup.lines().any(|line| line == LINE),
            "a missing mempool tx count painted {LINE}: {markup}"
        );
        assert!(
            !markup.contains("Transactions"),
            "a missing mempool tx count still contains Transactions: {markup}"
        );
    }
}
