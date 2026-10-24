//! Whole line `Confirmed balance 4000` on the address page and the scripthash page.
//!
//! GPUI `scripthash_screen_paints_confirmed_balance_line` requires one line
//! whose entire text equals `Confirmed balance 4000`. That number is chain
//! funded minus chain spent. The confirmed-balance paragraph stays and is
//! the longer line that still contains those words. A missing chain funded
//! sum omits the whole line. Mempool sums are not this difference.
//! Address markup and scripthash markup are separate strings. Each has
//! count 1.

use splora_web::{address_rows_markup, scripthash_markup};

const LINE: &str = "Confirmed balance 4000";

const CHAIN_COUNTS: &str = r#""tx_count":4,"funded_txo_count":4,"spent_txo_count":1"#;
const MEMPOOL_COUNTS: &str = r#""tx_count":2,"funded_txo_count":9,"spent_txo_count":1"#;

fn body(key: &str, value: &str, chain_sums: &str, mempool_sums: &str) -> String {
    format!(
        r#"{{"{key}":"{value}","chain_stats":{{{CHAIN_COUNTS}{chain_sums}}},"mempool_stats":{{{MEMPOOL_COUNTS}{mempool_sums}}}}}"#
    )
}

fn address_markup(chain_sums: &str, mempool_sums: &str) -> String {
    address_rows_markup(
        &body("address", "bc1qexample", chain_sums, mempool_sums),
        "",
        "",
    )
}

fn scripthash(chain_sums: &str, mempool_sums: &str) -> String {
    scripthash_markup(&body("scripthash", "deadbeef", chain_sums, mempool_sums))
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
    let paragraph = format!("<p class=\"confirmed-balance\">{LINE}</p>");
    assert!(
        markup.contains(&paragraph),
        "the confirmed-balance paragraph was dropped or lost the words: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line.contains(LINE) && line != LINE),
        "the longer line dropped {LINE}: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line.contains(LINE) && line.len() > LINE.len()),
        "no longer line contains {LINE}: {markup}"
    );
    assert!(
        !markup.contains("Confirmed balance 13"),
        "mempool sum difference painted as Confirmed balance: {markup}"
    );
    assert!(
        !markup.contains("Confirmed balance 4013"),
        "mempool sums were folded into Confirmed balance: {markup}"
    );
}

#[test]
fn address_and_scripthash_paint_confirmed_balance_as_its_own_line() {
    let chain = r#","funded_txo_sum":5000,"spent_txo_sum":1000"#;
    let mempool = r#","funded_txo_sum":20,"spent_txo_sum":7"#;
    let address = address_markup(chain, mempool);
    let scripthash_markup = scripthash(chain, mempool);
    assert_one_whole_line(&address);
    assert_one_whole_line(&scripthash_markup);
    assert_eq!(
        address.lines().filter(|line| *line == LINE).count(),
        1,
        "address markup count is not 1: {address}"
    );
    assert_eq!(
        scripthash_markup
            .lines()
            .filter(|line| *line == LINE)
            .count(),
        1,
        "scripthash markup count is not 1: {scripthash_markup}"
    );

    let missing_funded = r#","spent_txo_sum":1000"#;
    for markup in [
        address_markup(missing_funded, mempool),
        scripthash(missing_funded, mempool),
    ] {
        assert!(
            !markup.lines().any(|line| line == LINE),
            "a missing chain funded sum painted {LINE}: {markup}"
        );
        assert!(
            !markup.contains(LINE),
            "a missing chain funded sum still contains {LINE}: {markup}"
        );
    }
}
