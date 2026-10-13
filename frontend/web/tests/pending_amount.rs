//! Pending amount on the address page and the scripthash page.
//!
//! Pending is mempool `funded_txo_sum` minus mempool `spent_txo_sum`.
//! Both sums must be present. `Some(0)` minus `Some(0)` is `Pending 0`.
//! A missing key and JSON null omit the line. A missing sum is not zero.
//! Pending UTXOs stays the mempool count difference. Confirmed balance
//! stays the chain sums.

use splora_web::{address_rows_markup, scripthash_markup};

const CHAIN_SUMS: &str = r#","funded_txo_sum":5000,"spent_txo_sum":1000"#;

fn stats_body(
    key: &str,
    value: &str,
    mempool_funded_count: u64,
    mempool_spent_count: u64,
    mempool_sums: &str,
) -> String {
    format!(
        r#"{{"{key}":"{value}","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1{CHAIN_SUMS}}},"mempool_stats":{{"tx_count":1,"funded_txo_count":{mempool_funded_count},"spent_txo_count":{mempool_spent_count}{mempool_sums}}}}}"#
    )
}

fn address_body(mempool_funded_count: u64, mempool_spent_count: u64, mempool_sums: &str) -> String {
    stats_body(
        "address",
        "bc1qexample",
        mempool_funded_count,
        mempool_spent_count,
        mempool_sums,
    )
}

fn scripthash_body(
    mempool_funded_count: u64,
    mempool_spent_count: u64,
    mempool_sums: &str,
) -> String {
    stats_body(
        "scripthash",
        "deadbeef",
        mempool_funded_count,
        mempool_spent_count,
        mempool_sums,
    )
}

fn address_markup(body: &str) -> String {
    address_rows_markup(body, "", "")
}

fn assert_pending_amount(markup: &str, amount: &str) {
    let phrase = format!("Pending {amount}");
    let paragraph = format!("<p class=\"pending-amount\">{phrase}</p>");
    assert!(markup.contains(&paragraph), "{markup}");
    assert_eq!(
        markup.matches("<p class=\"pending-amount\">").count(),
        1,
        "{markup}"
    );
    assert!(!phrase.contains(','), "{phrase}");
}

fn assert_no_pending_amount(markup: &str) {
    assert!(!markup.contains("pending-amount"), "{markup}");
    assert!(
        !markup.contains("<p class=\"pending-amount\">Pending 0</p>"),
        "{markup}"
    );
    assert!(
        !markup.contains(">Pending 0</p>"),
        "missing mempool sum was painted as Pending 0: {markup}"
    );
}

fn assert_utxos_and_balance(markup: &str) {
    assert!(
        markup.contains("<p class=\"pending-utxos\">Pending UTXOs 3</p>"),
        "{markup}"
    );
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
}

fn assert_utxos_before_amount(markup: &str) {
    let utxos_at = markup
        .find("<p class=\"pending-utxos\">")
        .unwrap_or_else(|| panic!("pending utxos missing: {markup}"));
    let amount_at = markup
        .find("<p class=\"pending-amount\">")
        .unwrap_or_else(|| panic!("pending amount missing: {markup}"));
    assert!(
        utxos_at < amount_at,
        "Pending UTXOs does not come before the pending amount: {markup}"
    );
}

#[test]
fn address_pending_amount_is_mempool_funded_minus_mempool_spent() {
    let body = address_body(4, 1, r#","funded_txo_sum":20,"spent_txo_sum":5"#);
    let markup = address_markup(&body);
    assert_pending_amount(&markup, "15");
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
    assert!(
        !markup.contains("Confirmed balance 15"),
        "pending amount was painted as Confirmed balance: {markup}"
    );
    assert!(
        !markup.contains("Confirmed balance 3985"),
        "mempool sums were folded into Confirmed balance: {markup}"
    );
    assert!(
        markup.contains("<p class=\"pending-utxos\">Pending UTXOs 3</p>"),
        "{markup}"
    );
    assert!(
        !markup.contains("Pending UTXOs 15"),
        "pending amount was painted as Pending UTXOs: {markup}"
    );
    assert_utxos_before_amount(&markup);
}

#[test]
fn scripthash_pending_amount_is_mempool_funded_minus_mempool_spent() {
    let body = scripthash_body(4, 1, r#","funded_txo_sum":20,"spent_txo_sum":5"#);
    let markup = scripthash_markup(&body);
    assert!(markup.contains("scripthash"), "{markup}");
    assert!(!markup.contains("bc1qexample"), "{markup}");
    assert_pending_amount(&markup, "15");
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
    assert!(
        markup.contains("<p class=\"pending-utxos\">Pending UTXOs 3</p>"),
        "{markup}"
    );
}

#[test]
fn explicit_zero_mempool_sums_paint_pending_zero() {
    let sums = r#","funded_txo_sum":0,"spent_txo_sum":0"#;
    let address = address_markup(&address_body(4, 1, sums));
    let scripthash = scripthash_markup(&scripthash_body(4, 1, sums));
    assert_pending_amount(&address, "0");
    assert_pending_amount(&scripthash, "0");
    assert!(
        address.contains("<p class=\"pending-amount\">Pending 0</p>"),
        "explicit zero was omitted: {address}"
    );
    assert!(
        scripthash.contains("<p class=\"pending-amount\">Pending 0</p>"),
        "explicit zero was omitted: {scripthash}"
    );
}

#[test]
fn omitted_mempool_funded_sum_omits_pending_amount() {
    let sums = r#","spent_txo_sum":5"#;
    let address = address_markup(&address_body(4, 1, sums));
    let scripthash = scripthash_markup(&scripthash_body(4, 1, sums));
    assert_no_pending_amount(&address);
    assert_no_pending_amount(&scripthash);
    assert_utxos_and_balance(&address);
    assert_utxos_and_balance(&scripthash);
}

#[test]
fn omitted_mempool_spent_sum_omits_pending_amount() {
    let sums = r#","funded_txo_sum":20"#;
    let address = address_markup(&address_body(4, 1, sums));
    let scripthash = scripthash_markup(&scripthash_body(4, 1, sums));
    assert_no_pending_amount(&address);
    assert_no_pending_amount(&scripthash);
    assert_utxos_and_balance(&address);
    assert_utxos_and_balance(&scripthash);
}

#[test]
fn null_mempool_sum_omits_pending_amount() {
    let funded_null = r#","funded_txo_sum":null,"spent_txo_sum":5"#;
    let spent_null = r#","funded_txo_sum":20,"spent_txo_sum":null"#;
    for sums in [funded_null, spent_null] {
        let address = address_markup(&address_body(4, 1, sums));
        let scripthash = scripthash_markup(&scripthash_body(4, 1, sums));
        assert_no_pending_amount(&address);
        assert_no_pending_amount(&scripthash);
        assert_utxos_and_balance(&address);
        assert_utxos_and_balance(&scripthash);
    }
}

#[test]
fn pending_amount_difference_is_signed() {
    let sums = r#","funded_txo_sum":1,"spent_txo_sum":4"#;
    let address = address_markup(&address_body(4, 1, sums));
    let scripthash = scripthash_markup(&scripthash_body(4, 1, sums));
    assert_pending_amount(&address, "-3");
    assert_pending_amount(&scripthash, "-3");
    assert!(
        !address.contains("<p class=\"pending-amount\">Pending 0</p>"),
        "{address}"
    );
    assert!(
        !scripthash.contains("<p class=\"pending-amount\">Pending 0</p>"),
        "{scripthash}"
    );
    assert!(address.contains("Confirmed balance 4000"), "{address}");
    assert!(
        scripthash.contains("Confirmed balance 4000"),
        "{scripthash}"
    );
}

#[test]
fn scripthash_wire_parse_failure_does_not_invent_pending_amount() {
    let body = r#"{"scripthash":"deadbeef","chain_stats":{"tx_count":4,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":5}}"#;
    let markup = scripthash_markup(body);
    assert!(
        markup.contains("<p class=\"funded-sum\">5000</p>"),
        "{markup}"
    );
    assert!(markup.contains("scripthash"), "{markup}");
    assert_no_pending_amount(&markup);
    assert!(!markup.contains("Pending 0"), "{markup}");
}
