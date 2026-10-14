//! Transactions is the chain tx count plus the mempool tx count.
//!
//! The address page and the scripthash page paint that sum after the
//! pending amount and Balance. A missing tx count fails the stats object
//! and is not zero. Explicit `0` stays `0`. The add is `u128` so it does
//! not wrap. The separate chain and mempool count paragraphs stay.
//! A scripthash body that fails the wire parse does not invent the line
//! from those separate strings.

use splora_web::{address_rows_markup, scripthash_markup};

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

fn address_body(chain_tx: Option<u64>, mempool_tx: Option<u64>) -> String {
    page_body("address", "bc1qexample", chain_tx, mempool_tx)
}

fn scripthash_body(chain_tx: Option<u64>, mempool_tx: Option<u64>) -> String {
    page_body("scripthash", "deadbeef", chain_tx, mempool_tx)
}

fn address_markup(body: &str) -> String {
    address_rows_markup(body, "", "")
}

fn assert_transactions(markup: &str, count: &str) {
    let phrase = format!("Transactions {count}");
    let paragraph = format!("<p class=\"transactions\">{phrase}</p>");
    assert!(markup.contains(&paragraph), "{markup}");
    assert_eq!(
        markup.matches("<p class=\"transactions\">").count(),
        1,
        "{markup}"
    );
    let pending = markup
        .find("<p class=\"pending-utxos\">")
        .unwrap_or_else(|| panic!("pending utxos missing: {markup}"));
    let transactions = markup
        .find("<p class=\"transactions\">")
        .unwrap_or_else(|| panic!("transactions missing: {markup}"));
    assert!(
        pending < transactions,
        "Pending UTXOs does not come before Transactions: {markup}"
    );
    let pending_end = markup[pending..]
        .find("</p>")
        .map(|offset| pending + offset + "</p>".len())
        .unwrap_or_else(|| panic!("pending paragraph did not close: {markup}"));
    assert!(
        markup[pending_end..].starts_with("<p class=\"pending-amount\">"),
        "pending amount is not immediately after Pending UTXOs: {markup}"
    );
    let amount_end = markup[pending_end..]
        .find("</p>")
        .map(|offset| pending_end + offset + "</p>".len())
        .unwrap_or_else(|| panic!("pending amount paragraph did not close: {markup}"));
    assert!(
        markup[amount_end..].starts_with("<p class=\"balance\">"),
        "Balance is not immediately after the pending amount: {markup}"
    );
    let transactions_at = markup[pending_end..]
        .find("<p class=\"transactions\">")
        .map(|offset| pending_end + offset)
        .unwrap_or_else(|| panic!("transactions missing after Pending UTXOs: {markup}"));
    let before_transactions = &markup[pending_end..transactions_at];
    assert!(
        !markup[pending_end..].starts_with("<p class=\"transactions\">"),
        "Transactions must not be the next bytes after Pending UTXOs: {markup}"
    );
    assert!(
        before_transactions.contains("<p class=\"pending-amount\">"),
        "the slice before Transactions is missing the pending-amount paragraph: {markup}"
    );
    assert!(
        before_transactions.contains("<p class=\"balance\">"),
        "the slice before Transactions is missing the Balance paragraph: {markup}"
    );
    let balance_end = markup[amount_end..]
        .find("</p>")
        .map(|offset| amount_end + offset + "</p>".len())
        .unwrap_or_else(|| panic!("Balance paragraph did not close: {markup}"));
    assert!(
        balance_end <= transactions_at,
        "Balance does not close before Transactions: {markup}"
    );
}

fn assert_separate_counts(markup: &str, chain: &str, mempool: &str) {
    assert!(
        markup.contains(&format!("<p class=\"chain-tx-count\">{chain}</p>")),
        "{markup}"
    );
    assert!(
        markup.contains(&format!("<p class=\"mempool-tx-count\">{mempool}</p>")),
        "{markup}"
    );
}

fn assert_sum_not_in_separate_counts(markup: &str, sum: &str) {
    assert!(
        !markup.contains(&format!("<p class=\"chain-tx-count\">{sum}</p>")),
        "chain-tx-count used the sum: {markup}"
    );
    assert!(
        !markup.contains(&format!("<p class=\"mempool-tx-count\">{sum}</p>")),
        "mempool-tx-count used the sum: {markup}"
    );
}

fn assert_no_transactions(markup: &str) {
    assert!(!markup.contains("Transactions"), "{markup}");
    assert!(!markup.contains("Transactions 0"), "{markup}");
    assert!(!markup.contains("<p class=\"transactions\">"), "{markup}");
}

#[test]
fn address_transactions_is_chain_count_plus_mempool_count() {
    let markup = address_markup(&address_body(Some(4), Some(1)));
    assert_transactions(&markup, "5");
    assert_separate_counts(&markup, "4", "1");
    assert_sum_not_in_separate_counts(&markup, "5");
}

#[test]
fn scripthash_transactions_is_chain_count_plus_mempool_count() {
    let markup = scripthash_markup(&scripthash_body(Some(4), Some(1)));
    assert!(markup.contains("scripthash"), "{markup}");
    assert!(
        markup.contains("<p class=\"scripthash\">deadbeef</p>"),
        "{markup}"
    );
    assert!(!markup.contains("bc1qexample"), "{markup}");
    assert_transactions(&markup, "5");
    assert_separate_counts(&markup, "4", "1");
    assert_sum_not_in_separate_counts(&markup, "5");
}

#[test]
fn explicit_zero_tx_counts_paint_transactions_zero() {
    let address = address_markup(&address_body(Some(0), Some(0)));
    let scripthash = scripthash_markup(&scripthash_body(Some(0), Some(0)));
    assert_transactions(&address, "0");
    assert_transactions(&scripthash, "0");
    assert_separate_counts(&address, "0", "0");
    assert_separate_counts(&scripthash, "0", "0");
    assert!(
        address.contains("Transactions 0"),
        "explicit zero was omitted: {address}"
    );
    assert!(
        scripthash.contains("Transactions 0"),
        "explicit zero was omitted: {scripthash}"
    );
}

#[test]
fn omitted_chain_tx_count_omits_transactions() {
    let address = address_markup(&address_body(None, Some(1)));
    let scripthash = scripthash_markup(&scripthash_body(None, Some(1)));
    assert_no_transactions(&address);
    assert_no_transactions(&scripthash);
}

#[test]
fn omitted_mempool_tx_count_omits_transactions() {
    let address = address_markup(&address_body(Some(4), None));
    let scripthash = scripthash_markup(&scripthash_body(Some(4), None));
    assert_no_transactions(&address);
    assert_no_transactions(&scripthash);
}

#[test]
fn scripthash_wire_parse_failure_does_not_invent_transactions() {
    let body = r#"{"scripthash":"deadbeef","chain_stats":{"tx_count":4,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":2,"spent_txo_sum":20}}"#;
    let markup = scripthash_markup(body);
    assert!(markup.contains("scripthash"), "{markup}");
    assert!(
        markup.contains("<p class=\"scripthash\">deadbeef</p>"),
        "{markup}"
    );
    assert!(
        markup.contains("<p class=\"funded-sum\">5000</p>"),
        "scripthash page did not render, so the missing line is not meaningful: {markup}"
    );
    assert_separate_counts(&markup, "4", "2");
    assert_no_transactions(&markup);
    assert!(
        !markup.contains("Transactions 6"),
        "separate counts were added after the wire parse failed: {markup}"
    );
}

#[test]
fn transactions_add_does_not_wrap() {
    let chain = u64::MAX;
    let mempool = 1u64;
    let sum = (chain as u128) + (mempool as u128);
    let painted = sum.to_string();
    let address = address_markup(&address_body(Some(chain), Some(mempool)));
    let scripthash = scripthash_markup(&scripthash_body(Some(chain), Some(mempool)));
    assert_transactions(&address, &painted);
    assert_transactions(&scripthash, &painted);
    assert_separate_counts(&address, &chain.to_string(), "1");
    assert_separate_counts(&scripthash, &chain.to_string(), "1");
    assert_sum_not_in_separate_counts(&address, &painted);
    assert_sum_not_in_separate_counts(&scripthash, &painted);
    assert!(!address.contains("Transactions 0"), "{address}");
    assert!(!scripthash.contains("Transactions 0"), "{scripthash}");
}
