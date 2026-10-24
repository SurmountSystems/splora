//! Whole line `Unspent TXOs 3` on the address page.
//!
//! GPUI `address_screen_paints_unspent_txos_3` requires one line whose
//! entire text equals `Unspent TXOs 3`. That number is the chain funded
//! count minus the chain spent count. The unspent-txos paragraph stays
//! and is the longer line that still contains those words. Mempool funded
//! count 9 minus spent count 1 is not this line. Chain funded and spent
//! counts are plain integers, so this test does not invent a missing-count
//! omit. No scripthash GPUI test uses this const LINE.

use splora_web::address_rows_markup;

const LINE: &str = "Unspent TXOs 3";

fn paint(body: &str) -> String {
    address_rows_markup(body, "", "")
}

#[test]
fn address_paints_unspent_txos_3_as_its_own_line() {
    // Chain funded 4 minus spent 1. Mempool funded 9 and spent 1 must not
    // paint Unspent TXOs 8. Sums match the Transactions 5 fixture. tx_count
    // 4 plus 1 is what that fixture parses as Transactions 5.
    let markup = paint(
        r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":7}}"#,
    );
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    let paragraph = "<p class=\"unspent-txos\">Unspent TXOs 3</p>";
    assert!(
        markup.contains(paragraph),
        "the unspent txos paragraph was dropped or lost the words: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line.contains(LINE) && line != LINE),
        "the longer line dropped {LINE}: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Unspent TXOs 8"),
        "the mempool count difference was painted: {markup}"
    );
}
