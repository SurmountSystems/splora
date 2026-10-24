//! Whole line `Total sent 1020` on the address page and the scripthash page.
//!
//! GPUI `address_screen_paints_total_sent_1020` requires one line whose
//! entire text equals `Total sent 1020`. That number is the chain spent
//! sum plus the mempool spent sum. The total-sent paragraph stays and is
//! the longer line that still contains those words. A missing mempool
//! spent sum omits the whole line. The chain spent sum alone is not this
//! line, and the mempool spent sum alone is not this line.
//! Address markup and scripthash markup are separate strings. Each has
//! count 1.

use splora_web::{address_rows_markup, scripthash_markup};

const LINE: &str = "Total sent 1020";

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
    let paragraph = format!("<p class=\"total-sent\">{LINE}</p>");
    assert!(
        markup.contains(&paragraph),
        "the total-sent paragraph was dropped or lost the words: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line.contains(LINE) && line != LINE),
        "the longer line dropped {LINE}: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Total sent 1000"),
        "total sent ignored the mempool spent sum: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Total sent 20"),
        "the mempool spent sum was painted alone: {markup}"
    );
}

#[test]
fn address_and_scripthash_paint_total_sent_as_its_own_line() {
    let chain = r#","funded_txo_sum":5000,"spent_txo_sum":1000"#;
    let mempool = r#","funded_txo_sum":20,"spent_txo_sum":20"#;
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

    let missing_mempool_spent = r#","funded_txo_sum":20"#;
    let address_omitted = address_rows_markup(&address_body(chain, missing_mempool_spent), "", "");
    let hash_omitted = scripthash_markup(&scripthash_body(chain, missing_mempool_spent));
    for markup in [&address_omitted, &hash_omitted] {
        assert!(
            !markup.lines().any(|line| line == LINE),
            "a missing mempool spent sum painted {LINE}: {markup}"
        );
        assert!(
            !markup.contains("Total sent"),
            "a missing mempool spent sum still contains Total sent: {markup}"
        );
    }
}
