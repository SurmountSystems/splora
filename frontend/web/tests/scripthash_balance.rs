//! Confirmed balance and confirmed UTXO count on the scripthash page.
//!
//! Chain funded minus chain spent, same words as the address summary.
//! A missing chain sum is not zero. A missing chain count is not zero.
//! Mempool sums and mempool counts are not those lines.
//! An address body is not this function's success path.

use splora_web::scripthash_markup;

const SCRIPT_HASH: &str = "deadbeef";

fn paint(body: &str) -> String {
    scripthash_markup(body)
}

fn chain_stats(counts: &str, sums: &str) -> String {
    format!(r#""chain_stats":{{"tx_count":4,{counts}{sums}}}"#)
}

fn mempool_stats(counts: &str, sums: &str) -> String {
    format!(r#""mempool_stats":{{"tx_count":2,{counts}{sums}}}"#)
}

fn scripthash_body(chain: &str, mempool: &str) -> String {
    format!(r#"{{"scripthash":"{SCRIPT_HASH}",{chain},{mempool}}}"#)
}

fn full_counts() -> &'static str {
    r#""funded_txo_count":4,"spent_txo_count":1"#
}

fn mempool_counts() -> &'static str {
    r#""funded_txo_count":9,"spent_txo_count":1"#
}

fn assert_keeps_scripthash(markup: &str) {
    assert!(
        markup.contains(&format!("<p class=\"scripthash\">{SCRIPT_HASH}</p>")),
        "{markup}"
    );
    assert!(
        markup.contains("<p class=\"chain-tx-count\">4</p>"),
        "{markup}"
    );
    assert!(
        markup.contains("<p class=\"mempool-tx-count\">2</p>"),
        "{markup}"
    );
}

#[test]
fn scripthash_paints_confirmed_balance_and_utxo_count() {
    let body = scripthash_body(
        &chain_stats(
            full_counts(),
            r#","funded_txo_sum":5000,"spent_txo_sum":1000"#,
        ),
        &mempool_stats(
            mempool_counts(),
            r#","funded_txo_sum":20,"spent_txo_sum":7"#,
        ),
    );
    let markup = paint(&body);
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
    assert!(markup.contains("Confirmed UTXOs 3"), "{markup}");
    assert_keeps_scripthash(&markup);
    assert!(
        markup.contains("<p class=\"funded-sum\">5000</p>"),
        "{markup}"
    );
    assert!(
        !markup.contains("Confirmed UTXOs 8"),
        "mempool count difference painted: {markup}"
    );
    assert!(
        !markup.contains("Confirmed balance 13"),
        "mempool sum difference painted: {markup}"
    );
}

#[test]
fn scripthash_paints_zero_confirmed_balance_and_utxo_count() {
    let body = scripthash_body(
        &chain_stats(
            r#""funded_txo_count":0,"spent_txo_count":0"#,
            r#","funded_txo_sum":0,"spent_txo_sum":0"#,
        ),
        &mempool_stats(
            r#""funded_txo_count":3,"spent_txo_count":1"#,
            r#","funded_txo_sum":20,"spent_txo_sum":7"#,
        ),
    );
    let markup = paint(&body);
    assert!(markup.contains("Confirmed balance 0"), "{markup}");
    assert!(markup.contains("Confirmed UTXOs 0"), "{markup}");
    assert!(
        markup.contains(&format!("<p class=\"scripthash\">{SCRIPT_HASH}</p>")),
        "{markup}"
    );
    assert!(
        !markup.contains("Confirmed UTXOs 2"),
        "mempool count difference painted: {markup}"
    );
    assert!(
        !markup.contains("Confirmed balance 13"),
        "mempool sum difference painted: {markup}"
    );
}

#[test]
fn omitted_chain_spent_sum_hides_balance_and_keeps_utxos() {
    let body = scripthash_body(
        &chain_stats(full_counts(), r#","funded_txo_sum":5000"#),
        &mempool_stats(
            mempool_counts(),
            r#","funded_txo_sum":20,"spent_txo_sum":7"#,
        ),
    );
    let markup = paint(&body);
    assert!(!markup.contains("Confirmed balance"), "{markup}");
    assert!(markup.contains("Confirmed UTXOs 3"), "{markup}");
    assert!(
        markup.contains("<p class=\"funded-sum\">5000</p>"),
        "{markup}"
    );
    assert!(
        !markup.contains("<p class=\"chain-spent-sum\">"),
        "{markup}"
    );
    assert!(
        !markup.contains(">0</p>"),
        "spent sum painted as 0: {markup}"
    );
    assert_keeps_scripthash(&markup);
}

#[test]
fn null_chain_funded_sum_hides_confirmed_balance() {
    let body = scripthash_body(
        &chain_stats(
            full_counts(),
            r#","funded_txo_sum":null,"spent_txo_sum":1000"#,
        ),
        &mempool_stats(
            mempool_counts(),
            r#","funded_txo_sum":20,"spent_txo_sum":7"#,
        ),
    );
    let markup = paint(&body);
    assert!(!markup.contains("Confirmed balance"), "{markup}");
    assert!(markup.contains("Confirmed UTXOs 3"), "{markup}");
    assert!(!markup.contains("<p class=\"funded-sum\">"), "{markup}");
    assert!(
        !markup.contains(">0</p>"),
        "null funded sum painted as 0: {markup}"
    );
    assert_keeps_scripthash(&markup);
}

#[test]
fn omitted_funded_txo_count_hides_utxos_and_does_not_invent_balance() {
    let body = scripthash_body(
        &chain_stats(
            r#""spent_txo_count":1"#,
            r#","funded_txo_sum":5000,"spent_txo_sum":1000"#,
        ),
        &mempool_stats(
            mempool_counts(),
            r#","funded_txo_sum":20,"spent_txo_sum":7"#,
        ),
    );
    let markup = paint(&body);
    assert!(!markup.contains("Confirmed UTXOs"), "{markup}");
    assert!(!markup.contains("Confirmed balance"), "{markup}");
    assert!(!markup.contains("Confirmed UTXOs 0"), "{markup}");
    assert!(!markup.contains("Confirmed balance 0"), "{markup}");
    assert!(
        markup.contains("<p class=\"funded-sum\">5000</p>"),
        "{markup}"
    );
    assert_keeps_scripthash(&markup);
}

#[test]
fn mempool_sums_alone_do_not_paint_confirmed_balance() {
    let body = scripthash_body(
        &chain_stats(full_counts(), ""),
        &mempool_stats(
            mempool_counts(),
            r#","funded_txo_sum":20,"spent_txo_sum":7"#,
        ),
    );
    let markup = paint(&body);
    assert!(!markup.contains("Confirmed balance"), "{markup}");
    assert!(markup.contains("Confirmed UTXOs 3"), "{markup}");
    assert!(
        !markup.contains("Confirmed UTXOs 8"),
        "mempool count difference painted: {markup}"
    );
    assert!(!markup.contains("<p class=\"funded-sum\">"), "{markup}");
    assert_keeps_scripthash(&markup);
}

#[test]
fn address_body_is_not_scripthash_markup_success() {
    let body = format!(
        r#"{{"address":"bc1qexample",{chain},{mempool}}}"#,
        chain = chain_stats(
            full_counts(),
            r#","funded_txo_sum":5000,"spent_txo_sum":1000"#
        ),
        mempool = mempool_stats(
            mempool_counts(),
            r#","funded_txo_sum":20,"spent_txo_sum":7"#
        ),
    );
    let markup = paint(&body);
    assert!(!markup.contains("Confirmed balance"), "{markup}");
    assert!(!markup.contains("Confirmed UTXOs"), "{markup}");
    assert!(!markup.contains("bc1qexample"), "{markup}");
    assert!(!markup.contains(SCRIPT_HASH), "{markup}");
}
