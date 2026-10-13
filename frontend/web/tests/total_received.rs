//! Total received is the chain funded sum, not funded minus spent.
//!
//! `Some(0)` is `Total received 0`. A missing key and JSON null omit the
//! line. A mempool funded sum does not create it. The line follows the
//! confirmed UTXO paragraph, and only after the wire stats parse succeeds.

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

fn assert_total(markup: &str, amount: &str) {
    let phrase = format!("Total received {amount}");
    let paragraph = format!("<p class=\"total-received\">{phrase}</p>");
    assert!(markup.contains(&paragraph), "{markup}");
    assert_eq!(
        markup.matches("<p class=\"total-received\">").count(),
        1,
        "{markup}"
    );
    assert!(!phrase.contains(','), "{phrase}");
    assert!(!markup.contains('¥'), "{markup}");
    assert!(!markup.contains("Total received </p>"), "{markup}");
    let utxos_at = markup
        .find("Confirmed UTXOs")
        .unwrap_or_else(|| panic!("confirmed UTXOs missing: {markup}"));
    let total_at = markup
        .find("Total received")
        .unwrap_or_else(|| panic!("total received missing: {markup}"));
    assert!(
        utxos_at < total_at,
        "Confirmed UTXOs does not come before Total received: {markup}"
    );
}

fn assert_no_total(markup: &str) {
    assert!(!markup.contains("Total received"), "{markup}");
    assert!(!markup.contains("<p class=\"total-received\">"), "{markup}");
    assert!(
        markup.contains("Confirmed UTXOs"),
        "page did not render, so the missing line is not meaningful: {markup}"
    );
}

#[test]
fn address_total_received_is_chain_funded_sum_after_confirmed_utxos() {
    let body = address_body(
        r#","funded_txo_sum":5000,"spent_txo_sum":1000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":0"#,
    );
    let markup = address_markup(&body);
    assert_total(&markup, "5000");
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
    assert!(!markup.contains("Total received 4000"), "{markup}");
    assert!(!markup.contains("Total received 20"), "{markup}");
}

#[test]
fn scripthash_total_received_is_chain_funded_sum() {
    let body = scripthash_body(
        r#","funded_txo_sum":5000,"spent_txo_sum":1000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":7"#,
    );
    let markup = scripthash_markup(&body);
    assert!(markup.contains("scripthash"), "{markup}");
    assert!(!markup.contains("bc1qexample"), "{markup}");
    assert_total(&markup, "5000");
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
    assert!(!markup.contains("Total received 4000"), "{markup}");
    assert!(!markup.contains("Total received 20"), "{markup}");
}

#[test]
fn explicit_zero_chain_funded_sum_paints_total_received_zero() {
    let address = address_body(
        r#","funded_txo_sum":0,"spent_txo_sum":1000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":0"#,
    );
    let scripthash = scripthash_body(
        r#","funded_txo_sum":0,"spent_txo_sum":1000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":7"#,
    );
    let address_markup = address_markup(&address);
    let scripthash_markup = scripthash_markup(&scripthash);
    assert_total(&address_markup, "0");
    assert_total(&scripthash_markup, "0");
    assert!(
        address_markup.contains("Confirmed balance -1000"),
        "{address_markup}"
    );
    assert!(
        scripthash_markup.contains("Confirmed balance -1000"),
        "{scripthash_markup}"
    );
    assert!(
        !address_markup.contains("Total received -1000"),
        "{address_markup}"
    );
    assert!(
        !scripthash_markup.contains("Total received -1000"),
        "{scripthash_markup}"
    );
}

#[test]
fn omitted_chain_funded_sum_omits_total_received() {
    let address = address_body(r#","spent_txo_sum":1000"#, "");
    let scripthash = scripthash_body(r#","spent_txo_sum":1000"#, "");
    assert_no_total(&address_markup(&address));
    assert_no_total(&scripthash_markup(&scripthash));
}

#[test]
fn null_chain_funded_sum_omits_total_received() {
    let address = address_body(r#","funded_txo_sum":null,"spent_txo_sum":1000"#, "");
    let scripthash = scripthash_body(r#","funded_txo_sum":null,"spent_txo_sum":1000"#, "");
    assert_no_total(&address_markup(&address));
    assert_no_total(&scripthash_markup(&scripthash));
}

#[test]
fn mempool_funded_sum_alone_does_not_paint_total_received() {
    let address = address_body("", r#","funded_txo_sum":20,"spent_txo_sum":0"#);
    let scripthash = scripthash_body("", r#","funded_txo_sum":20,"spent_txo_sum":7"#);
    let address_markup = address_markup(&address);
    let scripthash_markup = scripthash_markup(&scripthash);
    assert_no_total(&address_markup);
    assert_no_total(&scripthash_markup);
    assert!(
        address_markup.contains("<p class=\"mempool-funded-sum\">20</p>"),
        "{address_markup}"
    );
    assert!(
        !scripthash_markup.contains("Total received 20"),
        "{scripthash_markup}"
    );
}

#[test]
fn scripthash_wire_parse_failure_does_not_invent_total_received() {
    let body = r#"{"scripthash":"deadbeef","chain_stats":{"tx_count":4,"funded_txo_sum":5000},"mempool_stats":{"tx_count":2}}"#;
    let markup = scripthash_markup(body);
    assert!(
        markup.contains("<p class=\"funded-sum\">5000</p>"),
        "{markup}"
    );
    assert!(!markup.contains("Total received"), "{markup}");
    assert!(!markup.contains("Confirmed UTXOs"), "{markup}");
    assert!(!markup.contains("Total received 0"), "{markup}");
}
