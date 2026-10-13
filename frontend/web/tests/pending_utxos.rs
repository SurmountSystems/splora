//! Pending UTXOs is the mempool funded count minus the mempool spent count.
//!
//! The address page and the scripthash page paint that difference after
//! Total sent. A missing mempool count fails the stats object and is not
//! zero. Explicit `0` stays `0`. The difference is signed, not saturating.
//! Confirmed UTXOs stays the chain counts only.

use splora_web::{address_rows_markup, scripthash_markup};

const CHAIN_SUMS: &str = r#","funded_txo_sum":5000,"spent_txo_sum":1000"#;
const MEMPOOL_SUMS: &str = r#","funded_txo_sum":20,"spent_txo_sum":250"#;

fn stats_body(
    key: &str,
    value: &str,
    chain_funded: u64,
    chain_spent: u64,
    mempool_funded: u64,
    mempool_spent: u64,
    chain_sums: &str,
    mempool_sums: &str,
) -> String {
    format!(
        r#"{{"{key}":"{value}","chain_stats":{{"tx_count":4,"funded_txo_count":{chain_funded},"spent_txo_count":{chain_spent}{chain_sums}}},"mempool_stats":{{"tx_count":1,"funded_txo_count":{mempool_funded},"spent_txo_count":{mempool_spent}{mempool_sums}}}}}"#
    )
}

fn address_body(
    chain_funded: u64,
    chain_spent: u64,
    mempool_funded: u64,
    mempool_spent: u64,
    chain_sums: &str,
    mempool_sums: &str,
) -> String {
    stats_body(
        "address",
        "bc1qexample",
        chain_funded,
        chain_spent,
        mempool_funded,
        mempool_spent,
        chain_sums,
        mempool_sums,
    )
}

fn scripthash_body(
    chain_funded: u64,
    chain_spent: u64,
    mempool_funded: u64,
    mempool_spent: u64,
    chain_sums: &str,
    mempool_sums: &str,
) -> String {
    stats_body(
        "scripthash",
        "deadbeef",
        chain_funded,
        chain_spent,
        mempool_funded,
        mempool_spent,
        chain_sums,
        mempool_sums,
    )
}

fn address_markup(body: &str) -> String {
    address_rows_markup(body, "", "")
}

fn assert_pending(markup: &str, count: &str) {
    let phrase = format!("Pending UTXOs {count}");
    let paragraph = format!("<p class=\"pending-utxos\">{phrase}</p>");
    assert!(markup.contains(&paragraph), "{markup}");
    assert_eq!(
        markup.matches("<p class=\"pending-utxos\">").count(),
        1,
        "{markup}"
    );
    assert!(!phrase.contains(','), "{phrase}");
}

fn assert_no_pending(markup: &str) {
    assert!(!markup.contains("Pending UTXOs"), "{markup}");
    assert!(!markup.contains("Pending UTXOs 0"), "{markup}");
}

fn assert_sent_before_pending(markup: &str) {
    let sent_at = markup
        .find("Total sent")
        .unwrap_or_else(|| panic!("total sent missing: {markup}"));
    let pending_at = markup
        .find("Pending UTXOs")
        .unwrap_or_else(|| panic!("pending utxos missing: {markup}"));
    assert!(
        sent_at < pending_at,
        "Total sent does not come before Pending UTXOs: {markup}"
    );
}

#[test]
fn address_pending_utxos_is_mempool_funded_minus_mempool_spent() {
    let body = address_body(4, 1, 5, 2, CHAIN_SUMS, MEMPOOL_SUMS);
    let markup = address_markup(&body);
    assert!(
        markup.contains("<p class=\"confirmed-utxos\">Confirmed UTXOs 3</p>"),
        "{markup}"
    );
    assert!(
        !markup.contains("Confirmed UTXOs 7"),
        "mempool counts were folded into Confirmed UTXOs: {markup}"
    );
    assert!(
        !markup.contains("Confirmed UTXOs 5"),
        "mempool funded count was painted as Confirmed UTXOs: {markup}"
    );
    assert_eq!(
        markup.matches("<p class=\"confirmed-utxos\">").count(),
        1,
        "{markup}"
    );
    assert_pending(&markup, "3");
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
    assert!(
        !markup.contains("Confirmed balance 3750"),
        "mempool spent sum was subtracted from Confirmed balance: {markup}"
    );
    assert!(markup.contains("Total sent 1250"), "{markup}");
    assert_sent_before_pending(&markup);
}

#[test]
fn scripthash_pending_utxos_is_mempool_funded_minus_mempool_spent() {
    let body = scripthash_body(4, 1, 5, 2, CHAIN_SUMS, MEMPOOL_SUMS);
    let markup = scripthash_markup(&body);
    assert!(markup.contains("scripthash"), "{markup}");
    assert!(
        markup.contains("<p class=\"scripthash\">deadbeef</p>"),
        "{markup}"
    );
    assert!(!markup.contains("bc1qexample"), "{markup}");
    assert!(markup.contains("Confirmed UTXOs 3"), "{markup}");
    assert!(
        !markup.contains("Confirmed UTXOs 7"),
        "mempool counts were folded into Confirmed UTXOs: {markup}"
    );
    assert!(
        !markup.contains("Confirmed UTXOs 5"),
        "mempool funded count was painted as Confirmed UTXOs: {markup}"
    );
    assert_pending(&markup, "3");
}

#[test]
fn explicit_zero_mempool_counts_paint_pending_utxos_zero() {
    let address = address_body(4, 1, 0, 0, CHAIN_SUMS, MEMPOOL_SUMS);
    let scripthash = scripthash_body(4, 1, 0, 0, CHAIN_SUMS, MEMPOOL_SUMS);
    let address_markup = address_markup(&address);
    let scripthash_markup = scripthash_markup(&scripthash);
    assert_pending(&address_markup, "0");
    assert_pending(&scripthash_markup, "0");
    assert!(
        address_markup.contains("Pending UTXOs 0"),
        "explicit zero was omitted: {address_markup}"
    );
    assert!(
        scripthash_markup.contains("Pending UTXOs 0"),
        "explicit zero was omitted: {scripthash_markup}"
    );
}

#[test]
fn omitted_mempool_funded_count_omits_pending_utxos() {
    let address = r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":250}}"#;
    let scripthash = r#"{"scripthash":"deadbeef","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"spent_txo_count":2,"funded_txo_sum":20,"spent_txo_sum":250}}"#;
    let address_markup = address_markup(address);
    let scripthash_markup = scripthash_markup(scripthash);
    assert_no_pending(&address_markup);
    assert_no_pending(&scripthash_markup);
    assert!(
        scripthash_markup.contains("<p class=\"funded-sum\">5000</p>"),
        "scripthash page did not render, so the missing line is not meaningful: {scripthash_markup}"
    );
}

#[test]
fn omitted_mempool_spent_count_omits_pending_utxos() {
    let address = r#"{"address":"bc1qexample","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":5,"funded_txo_sum":20,"spent_txo_sum":250}}"#;
    let scripthash = r#"{"scripthash":"deadbeef","chain_stats":{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":5,"funded_txo_sum":20,"spent_txo_sum":250}}"#;
    let address_markup = address_markup(address);
    let scripthash_markup = scripthash_markup(scripthash);
    assert_no_pending(&address_markup);
    assert_no_pending(&scripthash_markup);
    assert!(
        scripthash_markup.contains("<p class=\"funded-sum\">5000</p>"),
        "scripthash page did not render, so the missing line is not meaningful: {scripthash_markup}"
    );
}

#[test]
fn pending_utxos_still_paints_when_total_sent_is_omitted() {
    let chain_sums = r#","funded_txo_sum":5000"#;
    let mempool_sums = r#","funded_txo_sum":20"#;
    let address = address_markup(&address_body(4, 1, 4, 1, chain_sums, mempool_sums));
    let scripthash = scripthash_markup(&scripthash_body(4, 1, 4, 1, chain_sums, mempool_sums));
    assert!(!address.contains("Total sent"), "{address}");
    assert!(!scripthash.contains("Total sent"), "{scripthash}");
    assert_pending(&address, "3");
    assert_pending(&scripthash, "3");
}

#[test]
fn scripthash_missing_counts_do_not_invent_pending_utxos() {
    let body = r#"{"scripthash":"deadbeef","chain_stats":{"tx_count":4,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":2,"spent_txo_sum":20}}"#;
    let markup = scripthash_markup(body);
    assert!(
        markup.contains("<p class=\"funded-sum\">5000</p>"),
        "{markup}"
    );
    assert!(markup.contains("scripthash"), "{markup}");
    assert_no_pending(&markup);
    assert!(!markup.contains("Confirmed UTXOs"), "{markup}");
}

#[test]
fn pending_utxo_difference_is_signed() {
    let address = address_markup(&address_body(4, 1, 1, 4, CHAIN_SUMS, MEMPOOL_SUMS));
    let scripthash = scripthash_markup(&scripthash_body(4, 1, 1, 4, CHAIN_SUMS, MEMPOOL_SUMS));
    assert_pending(&address, "-3");
    assert_pending(&scripthash, "-3");
    assert!(!address.contains("Pending UTXOs 0"), "{address}");
    assert!(!scripthash.contains("Pending UTXOs 0"), "{scripthash}");
}
