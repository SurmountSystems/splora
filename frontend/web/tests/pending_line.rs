//! Whole line `Pending 15` on the address page.
//!
//! GPUI `address_screen_paints_pending_15` requires one line whose entire
//! text equals `Pending 15`. That number is the mempool funded sum minus
//! the mempool spent sum. The pending-amount paragraph stays and is the
//! longer line that still contains those words. Chain sums are not this
//! line. A plus is not this line. A missing mempool spent sum does not
//! paint the line and does not contain `Pending 15`. No scripthash GPUI
//! test uses this const LINE. The shipped address fixture already paints
//! the pending-amount paragraph. This test does not treat that existing
//! text as this line.

use splora_web::address_rows_markup;

const LINE: &str = "Pending 15";

fn paint(body: &str) -> String {
    address_rows_markup(body, "", "")
}

#[test]
fn address_page_paints_pending_15_as_its_own_line() {
    // Mempool funded sum 20 minus spent sum 5. Chain sums must not paint
    // this line. A plus is not this line.
    let markup = paint(
        r#"{"address":"bc1qpending15","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":5}}"#,
    );
    let paragraph = "<p class=\"pending-amount\">Pending 15</p>";
    assert!(
        markup.contains(paragraph),
        "the pending amount paragraph was dropped or lost the words: {markup}"
    );
    assert!(
        markup
            .lines()
            .filter(|line| line.contains(paragraph))
            .all(|line| line != LINE),
        "the pending amount paragraph line is the whole sentence: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    for forbidden in ["Pending +15", "Pending 20", "Pending 5", "Pending -15"] {
        assert!(
            !markup.lines().any(|line| line == forbidden),
            "{forbidden} was painted: {markup}"
        );
    }

    let omitted = paint(
        r#"{"address":"bc1qnopending15","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":20}}"#,
    );
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "a missing mempool spent sum painted {LINE}: {omitted}"
    );
    assert!(
        !omitted.contains("Pending 15"),
        "a missing mempool spent sum painted Pending 15: {omitted}"
    );
    assert!(
        !omitted.lines().any(|line| line == "Pending 0"),
        "a missing mempool spent sum painted Pending 0: {omitted}"
    );
}
