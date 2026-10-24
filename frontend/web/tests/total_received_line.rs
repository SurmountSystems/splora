//! Whole line `Total received 5000` on the address page and the scripthash page.
//!
//! GPUI `address_screen_paints_total_received_5000` requires one line whose
//! entire text equals `Total received 5000`. That number is the chain funded
//! sum. The total-received paragraph stays and is the longer line that still
//! contains those words. A missing chain funded sum omits the whole line.
//! Confirmed balance and the mempool funded sum are not this line.
//! Address markup and scripthash markup are separate strings. Each has
//! count 1.

use splora_web::{address_rows_markup, scripthash_markup};

const LINE: &str = "Total received 5000";

fn address_body(chain_sums: &str, mempool_sums: &str) -> String {
    format!(
        r#"{{"address":"bc1qexample","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1{chain_sums}}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0{mempool_sums}}}}}"#
    )
}

fn scripthash_body(chain_sums: &str, mempool_sums: &str) -> String {
    format!(
        r#"{{"scripthash":"deadbeef","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1{chain_sums}}},"mempool_stats":{{"tx_count":2,"funded_txo_count":9,"spent_txo_count":1{mempool_sums}}}}}"#
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
    let paragraph = format!("<p class=\"total-received\">{LINE}</p>");
    assert!(
        markup.contains(&paragraph),
        "the total-received paragraph was dropped or lost the words: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line.contains(LINE) && line != LINE),
        "the longer line dropped {LINE}: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Total received 4000"),
        "confirmed balance was painted as total received: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Total received 20"),
        "mempool funded sum was painted as total received: {markup}"
    );
}

#[test]
fn address_and_scripthash_paint_total_received_as_its_own_line() {
    let chain = r#","funded_txo_sum":5000,"spent_txo_sum":1000"#;
    let mempool = r#","funded_txo_sum":20,"spent_txo_sum":0"#;
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

    let missing_funded = r#","spent_txo_sum":1000"#;
    let address_omitted = address_rows_markup(&address_body(missing_funded, mempool), "", "");
    let hash_omitted = scripthash_markup(&scripthash_body(missing_funded, mempool));
    for markup in [&address_omitted, &hash_omitted] {
        assert!(
            !markup.lines().any(|line| line == LINE),
            "a missing chain funded sum painted {LINE}: {markup}"
        );
        assert!(
            !markup.contains("Total received"),
            "a missing chain funded sum still contains Total received: {markup}"
        );
    }
}
