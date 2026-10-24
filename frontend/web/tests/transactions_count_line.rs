//! Whole line `Transactions 5` on the address page and the scripthash page.
//!
//! GPUI `address_screen_paints_transactions_5` requires one line whose
//! entire text equals `Transactions 5`. That number is the chain tx count
//! plus the mempool tx count. The transactions paragraph stays and is the
//! longer line that still contains those words. A missing mempool tx count
//! does not paint the line. The chain tx count alone is not this line, and
//! the mempool tx count alone is not this line.
//! Address markup and scripthash markup are separate strings. Each has
//! count 1.

use splora_web::{address_rows_markup, scripthash_markup};

const LINE: &str = "Transactions 5";

fn address_body(chain_tx: &str, mempool_tx: &str) -> String {
    format!(
        r#"{{"address":"bc1qexample","chain_stats":{{{chain_tx}"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000}},"mempool_stats":{{{mempool_tx}"funded_txo_count":2,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
    )
}

fn scripthash_body(chain_tx: &str, mempool_tx: &str) -> String {
    format!(
        r#"{{"scripthash":"deadbeef","chain_stats":{{{chain_tx}"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000}},"mempool_stats":{{{mempool_tx}"funded_txo_count":2,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}}}"#
    )
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
fn address_and_scripthash_paint_transactions_5_as_its_own_line() {
    let chain = r#""tx_count":4,"#;
    let mempool = r#""tx_count":1,"#;
    let address = address_rows_markup(&address_body(chain, mempool), "", "");
    let hash_markup = scripthash_markup(&scripthash_body(chain, mempool));
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

    let address_omitted = address_rows_markup(&address_body(chain, ""), "", "");
    let hash_omitted = scripthash_markup(&scripthash_body(chain, ""));
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
