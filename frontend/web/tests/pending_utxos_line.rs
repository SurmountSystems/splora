//! Whole line `Pending UTXOs 3` on the address page.
//!
//! GPUI `address_screen_paints_pending_utxos_3` requires one line whose
//! entire text equals `Pending UTXOs 3`. That number is the mempool funded
//! count minus the mempool spent count. The pending-utxos paragraph stays
//! and is the longer line that still contains those words. Chain funded
//! count 4 is not this line. Folding chain counts in is not this line.
//! A missing mempool funded count does not paint the line and does not
//! contain `Pending UTXOs`. No scripthash GPUI test uses this const LINE.
//! The shipped address fixture already paints the pending paragraph. This
//! test does not treat that existing text as this line.

use splora_web::address_rows_markup;

const LINE: &str = "Pending UTXOs 3";

fn paint(body: &str) -> String {
    address_rows_markup(body, "", "")
}

#[test]
fn address_page_paints_pending_utxos_3_as_its_own_line() {
    // Mempool funded 5 minus spent 2. Chain funded 4 must not paint
    // Pending UTXOs 4. Folding chain counts in must not paint Pending UTXOs 7.
    let markup = paint(
        r#"{"address":"bc1qpendingutxos3","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":5,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":250}}"#,
    );
    let paragraph = "<p class=\"pending-utxos\">Pending UTXOs 3</p>";
    assert!(
        markup.contains(paragraph),
        "the pending utxos paragraph was dropped or lost the words: {markup}"
    );
    assert!(
        markup
            .lines()
            .filter(|line| line.contains(paragraph))
            .all(|line| line != LINE),
        "the pending utxos paragraph line is the whole sentence: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    for forbidden in ["Pending UTXOs +3", "Pending UTXOs 4", "Pending UTXOs 7"] {
        assert!(
            !markup.contains(forbidden),
            "{forbidden} was painted: {markup}"
        );
    }

    let omitted = paint(
        r#"{"address":"bc1qnopending3","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":250}}"#,
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing mempool funded count painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains("Pending UTXOs"),
        "a missing mempool funded count painted Pending UTXOs: {omitted}"
    );
}
