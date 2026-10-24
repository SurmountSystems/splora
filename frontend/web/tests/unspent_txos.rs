//! Unspent TXOs on the address preview is chain funded count minus chain
//! spent count.
//!
//! Mempool counts are not that line. A body that fails address stats parse
//! omits the row. A missing body is not zero. Confirmed UTXOs stays.

use splora_web::address_rows_markup;

fn paint(body: &str) -> String {
    address_rows_markup(body, "", "")
}

/// Same address body shape as `address_stats_json` in `confirmed_utxos.rs`.
/// Chain counts are the arguments. Mempool funded count is 9 and spent count
/// is 1, so a mempool difference is 8, not the chain difference.
fn chain_counts(funded: u64, spent: u64) -> String {
    format!(
        r#"{{"address":"bc1qexample","chain_stats":{{"tx_count":4,"funded_txo_count":{funded},"spent_txo_count":{spent},"funded_txo_sum":5000,"spent_txo_sum":1000}},"mempool_stats":{{"tx_count":1,"funded_txo_count":9,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":0}}}}"#
    )
}

#[test]
fn unspent_txos_are_chain_funded_minus_chain_spent() {
    let markup = paint(&chain_counts(4, 1));
    let phrase = "Unspent TXOs 3";
    let paragraph = "<p class=\"unspent-txos\">Unspent TXOs 3</p>";
    // The paragraph and the whole line each contain the words.
    assert_eq!(markup.matches(phrase).count(), 2, "{markup}");
    assert!(markup.contains(paragraph), "{markup}");
    let transactions = markup
        .find("Transactions")
        .unwrap_or_else(|| panic!("transactions missing: {markup}"));
    let unspent = markup
        .find("Unspent TXOs")
        .unwrap_or_else(|| panic!("unspent txos missing: {markup}"));
    assert!(
        unspent > transactions,
        "Unspent TXOs does not come after Transactions: {markup}"
    );
    assert!(
        !markup.contains("Unspent TXOs 8"),
        "mempool count difference painted: {markup}"
    );
    assert!(
        !markup.contains("Unspent TXOs 5"),
        "mempool count was folded into Unspent TXOs: {markup}"
    );
    assert!(
        markup.contains("<p class=\"confirmed-utxos\">Confirmed UTXOs 3</p>"),
        "{markup}"
    );
}

#[test]
fn unspent_txos_zero_when_both_chain_counts_are_zero() {
    let markup = paint(&chain_counts(0, 0));
    let phrase = "Unspent TXOs 0";
    let paragraph = "<p class=\"unspent-txos\">Unspent TXOs 0</p>";
    assert!(markup.contains(phrase), "{markup}");
    assert!(markup.contains(paragraph), "{markup}");
    assert!(
        !markup.contains("Unspent TXOs 8"),
        "mempool count difference painted: {markup}"
    );
}

#[test]
fn failed_address_stats_parse_omits_unspent_txos() {
    let markup = paint("not-json");
    assert!(!markup.contains("Unspent TXOs"), "{markup}");
    assert!(!markup.contains("Unspent TXOs 0"), "{markup}");
}
