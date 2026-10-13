//! Total sent is the chain spent sum plus the mempool spent sum.
//!
//! Both sums must be present. `Some(0)` plus `Some(0)` is `Total sent 0`.
//! A missing key and JSON null omit the line. The line follows Total
//! received when that line is present. The add is wide and does not wrap.
//! A scripthash body that fails the wire parse does not invent the line.

use splora_web::{address_rows_markup, scripthash_markup};

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

fn address_markup(body: &str) -> String {
    address_rows_markup(body, "", "")
}

fn assert_sent(markup: &str, amount: &str) {
    let phrase = format!("Total sent {amount}");
    let paragraph = format!("<p class=\"total-sent\">{phrase}</p>");
    assert!(markup.contains(&paragraph), "{markup}");
    assert_eq!(
        markup.matches("<p class=\"total-sent\">").count(),
        1,
        "{markup}"
    );
    assert!(!phrase.contains(','), "{phrase}");
    assert!(!markup.contains('¥'), "{markup}");
    assert!(!markup.contains("Total sent </p>"), "{markup}");
}

fn assert_no_sent(markup: &str) {
    assert!(!markup.contains("Total sent"), "{markup}");
    assert!(!markup.contains("<p class=\"total-sent\">"), "{markup}");
    assert!(
        markup.contains("Confirmed UTXOs"),
        "page did not render, so the missing line is not meaningful: {markup}"
    );
}

fn assert_received_before_sent(markup: &str) {
    let received_at = markup
        .find("Total received")
        .unwrap_or_else(|| panic!("total received missing: {markup}"));
    let sent_at = markup
        .find("Total sent")
        .unwrap_or_else(|| panic!("total sent missing: {markup}"));
    assert!(
        received_at < sent_at,
        "Total received does not come before Total sent: {markup}"
    );
}

#[test]
fn address_total_sent_is_chain_spent_plus_mempool_spent() {
    let body = address_body(
        r#","funded_txo_sum":5000,"spent_txo_sum":1000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":20"#,
    );
    let markup = address_markup(&body);
    assert_sent(&markup, "1020");
    assert!(markup.contains("Total received 5000"), "{markup}");
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
    assert!(
        !markup.contains("Confirmed balance 3980"),
        "mempool spent was subtracted from the confirmed balance: {markup}"
    );
    assert_received_before_sent(&markup);
}

#[test]
fn scripthash_total_sent_is_chain_spent_plus_mempool_spent() {
    let body = scripthash_body(
        r#","funded_txo_sum":5000,"spent_txo_sum":1000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":20"#,
    );
    let markup = scripthash_markup(&body);
    assert!(markup.contains("scripthash"), "{markup}");
    assert!(!markup.contains("bc1qexample"), "{markup}");
    assert_sent(&markup, "1020");
    assert!(markup.contains("Total received 5000"), "{markup}");
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
    assert!(
        !markup.contains("Confirmed balance 3980"),
        "mempool spent was subtracted from the confirmed balance: {markup}"
    );
    assert_received_before_sent(&markup);
}

#[test]
fn explicit_zero_spent_sums_paint_total_sent_zero() {
    let address = address_body(
        r#","funded_txo_sum":5000,"spent_txo_sum":0"#,
        r#","funded_txo_sum":20,"spent_txo_sum":0"#,
    );
    let scripthash = scripthash_body(
        r#","funded_txo_sum":5000,"spent_txo_sum":0"#,
        r#","funded_txo_sum":20,"spent_txo_sum":0"#,
    );
    assert_sent(&address_markup(&address), "0");
    assert_sent(&scripthash_markup(&scripthash), "0");
}

#[test]
fn omitted_chain_spent_sum_omits_total_sent() {
    let address = address_body(r#","funded_txo_sum":5000"#, r#","spent_txo_sum":20"#);
    let scripthash = scripthash_body(r#","funded_txo_sum":5000"#, r#","spent_txo_sum":20"#);
    assert_no_sent(&address_markup(&address));
    assert_no_sent(&scripthash_markup(&scripthash));
}

#[test]
fn omitted_mempool_spent_sum_omits_total_sent() {
    let address = address_body(r#","funded_txo_sum":5000,"spent_txo_sum":1000"#, "");
    let scripthash = scripthash_body(r#","funded_txo_sum":5000,"spent_txo_sum":1000"#, "");
    assert_no_sent(&address_markup(&address));
    assert_no_sent(&scripthash_markup(&scripthash));
}

#[test]
fn null_spent_sum_omits_total_sent() {
    let chain_null = (
        r#","funded_txo_sum":5000,"spent_txo_sum":null"#,
        r#","spent_txo_sum":20"#,
    );
    let mempool_null = (
        r#","funded_txo_sum":5000,"spent_txo_sum":1000"#,
        r#","spent_txo_sum":null"#,
    );
    assert_no_sent(&address_markup(&address_body(chain_null.0, chain_null.1)));
    assert_no_sent(&scripthash_markup(&scripthash_body(
        chain_null.0,
        chain_null.1,
    )));
    assert_no_sent(&address_markup(&address_body(
        mempool_null.0,
        mempool_null.1,
    )));
    assert_no_sent(&scripthash_markup(&scripthash_body(
        mempool_null.0,
        mempool_null.1,
    )));
}

#[test]
fn scripthash_wire_parse_failure_does_not_invent_total_sent() {
    let body = r#"{"scripthash":"deadbeef","chain_stats":{"tx_count":4,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":2,"spent_txo_sum":20}}"#;
    let markup = scripthash_markup(body);
    assert!(
        markup.contains("<p class=\"funded-sum\">5000</p>"),
        "{markup}"
    );
    assert!(!markup.contains("Total sent"), "{markup}");
    assert!(!markup.contains("<p class=\"total-sent\">"), "{markup}");
    assert!(!markup.contains("Confirmed UTXOs"), "{markup}");
    assert!(!markup.contains("Total sent 0"), "{markup}");
}

#[test]
fn total_sent_add_does_not_wrap() {
    let chain_spent = u64::MAX;
    let mempool_spent = 1u64;
    let sum = (chain_spent as u128) + (mempool_spent as u128);
    let chain = format!(r#","funded_txo_sum":0,"spent_txo_sum":{chain_spent}"#);
    let mempool = format!(r#","funded_txo_sum":0,"spent_txo_sum":{mempool_spent}"#);
    let address = address_markup(&address_body(&chain, &mempool));
    let scripthash = scripthash_markup(&scripthash_body(&chain, &mempool));
    assert_sent(&address, &sum.to_string());
    assert_sent(&scripthash, &sum.to_string());
    assert!(!address.contains("Total sent 0"), "{address}");
    assert!(!scripthash.contains("Total sent 0"), "{scripthash}");
}
