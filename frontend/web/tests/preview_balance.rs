//! Balance on the address page and the scripthash page.
//!
//! Balance is (chain `funded_txo_sum` plus mempool `funded_txo_sum`) minus
//! (chain `spent_txo_sum` plus mempool `spent_txo_sum`). All four sums must
//! be present. `Some(0)` on all four is `Balance 0`. A missing key and JSON
//! null omit the line. A missing sum is not zero. Confirmed balance stays
//! the chain sums. Pending stays the mempool sums.

use splora_web::{address_rows_markup, scripthash_markup};

const CHAIN_COUNTS: &str = r#""tx_count":4,"funded_txo_count":4,"spent_txo_count":1"#;
const MEMPOOL_COUNTS: &str = r#""tx_count":1,"funded_txo_count":4,"spent_txo_count":1"#;
const EXAMPLE_CHAIN: &str = r#","funded_txo_sum":5000,"spent_txo_sum":1000"#;
const EXAMPLE_MEMPOOL: &str = r#","funded_txo_sum":20,"spent_txo_sum":5"#;

fn stats_body(key: &str, value: &str, chain_sums: &str, mempool_sums: &str) -> String {
    format!(
        r#"{{"{key}":"{value}","chain_stats":{{{CHAIN_COUNTS}{chain_sums}}},"mempool_stats":{{{MEMPOOL_COUNTS}{mempool_sums}}}}}"#
    )
}

fn address_body(chain_sums: &str, mempool_sums: &str) -> String {
    stats_body("address", "bc1qexample", chain_sums, mempool_sums)
}

fn scripthash_body(chain_sums: &str, mempool_sums: &str) -> String {
    stats_body("scripthash", "deadbeef", chain_sums, mempool_sums)
}

fn address_markup(body: &str) -> String {
    address_rows_markup(body, "", "")
}

fn assert_one_balance(markup: &str, amount: &str) {
    let paragraph = format!("<p class=\"balance\">Balance {amount}</p>");
    assert!(
        markup.contains(&paragraph),
        "missing balance paragraph: {markup}"
    );
    assert_eq!(
        markup.matches("<p class=\"balance\">").count(),
        1,
        "{markup}"
    );
    assert!(!paragraph.contains(','), "{paragraph}");
}

fn assert_no_balance(markup: &str) {
    assert!(
        !markup.contains("class=\"balance\""),
        "balance paragraph was painted: {markup}"
    );
    assert!(
        !markup.contains("Balance 0"),
        "missing sum was painted as Balance 0: {markup}"
    );
}

fn assert_pending_before_balance(markup: &str) {
    let pending_at = markup
        .find("<p class=\"pending-amount\">")
        .unwrap_or_else(|| panic!("pending amount missing: {markup}"));
    let balance_at = markup
        .find("<p class=\"balance\">")
        .unwrap_or_else(|| panic!("balance paragraph missing: {markup}"));
    assert!(
        pending_at < balance_at,
        "pending amount does not come before Balance: {markup}"
    );
}

#[test]
fn address_balance_is_all_four_sums() {
    let body = address_body(EXAMPLE_CHAIN, EXAMPLE_MEMPOOL);
    let markup = address_markup(&body);
    assert_one_balance(&markup, "4015");
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
    assert!(
        !markup.contains("Confirmed balance 4015"),
        "Balance was painted as Confirmed balance: {markup}"
    );
    assert!(
        markup.contains("<p class=\"pending-amount\">Pending 15</p>"),
        "{markup}"
    );
    assert_pending_before_balance(&markup);
}

#[test]
fn scripthash_balance_is_all_four_sums() {
    let body = scripthash_body(EXAMPLE_CHAIN, EXAMPLE_MEMPOOL);
    let markup = scripthash_markup(&body);
    assert!(markup.contains("scripthash"), "{markup}");
    assert!(!markup.contains("bc1qexample"), "{markup}");
    assert_one_balance(&markup, "4015");
    assert!(markup.contains("Confirmed balance 4000"), "{markup}");
    assert!(
        markup.contains("<p class=\"pending-amount\">Pending 15</p>"),
        "{markup}"
    );
}

#[test]
fn explicit_zero_sums_paint_balance_zero() {
    let zeros = r#","funded_txo_sum":0,"spent_txo_sum":0"#;
    let address = address_markup(&address_body(zeros, zeros));
    let scripthash = scripthash_markup(&scripthash_body(zeros, zeros));
    assert_one_balance(&address, "0");
    assert_one_balance(&scripthash, "0");
    assert!(
        address.contains("<p class=\"balance\">Balance 0</p>"),
        "explicit zero was omitted: {address}"
    );
    assert!(
        scripthash.contains("<p class=\"balance\">Balance 0</p>"),
        "explicit zero was omitted: {scripthash}"
    );
    assert!(address.contains("Confirmed balance 0"), "{address}");
    assert!(scripthash.contains("Confirmed balance 0"), "{scripthash}");
    assert!(
        address.contains("<p class=\"pending-amount\">Pending 0</p>"),
        "{address}"
    );
    assert!(
        scripthash.contains("<p class=\"pending-amount\">Pending 0</p>"),
        "{scripthash}"
    );
}

#[test]
fn omitted_chain_funded_sum_omits_balance() {
    let chain = r#","spent_txo_sum":1000"#;
    let address = address_markup(&address_body(chain, EXAMPLE_MEMPOOL));
    let scripthash = scripthash_markup(&scripthash_body(chain, EXAMPLE_MEMPOOL));
    for markup in [&address, &scripthash] {
        assert_no_balance(markup);
        assert!(
            !markup.contains("Confirmed balance"),
            "Confirmed balance painted without chain funded sum: {markup}"
        );
        assert!(
            markup.contains("<p class=\"pending-amount\">Pending 15</p>"),
            "{markup}"
        );
    }
}

#[test]
fn omitted_mempool_spent_sum_omits_balance() {
    let mempool = r#","funded_txo_sum":20"#;
    let address = address_markup(&address_body(EXAMPLE_CHAIN, mempool));
    let scripthash = scripthash_markup(&scripthash_body(EXAMPLE_CHAIN, mempool));
    for markup in [&address, &scripthash] {
        assert_no_balance(markup);
        assert!(markup.contains("Confirmed balance 4000"), "{markup}");
        assert!(
            !markup.contains("pending-amount"),
            "Pending painted without mempool spent sum: {markup}"
        );
        assert!(
            !markup.contains(">Pending 0</p>"),
            "missing mempool sum was painted as Pending 0: {markup}"
        );
    }
}

#[test]
fn null_sum_omits_balance() {
    let cases = [
        (
            r#","funded_txo_sum":null,"spent_txo_sum":1000"#,
            EXAMPLE_MEMPOOL,
        ),
        (
            r#","funded_txo_sum":5000,"spent_txo_sum":null"#,
            EXAMPLE_MEMPOOL,
        ),
        (EXAMPLE_CHAIN, r#","funded_txo_sum":null,"spent_txo_sum":5"#),
        (
            EXAMPLE_CHAIN,
            r#","funded_txo_sum":20,"spent_txo_sum":null"#,
        ),
    ];
    for (chain, mempool) in cases {
        let address = address_markup(&address_body(chain, mempool));
        let scripthash = scripthash_markup(&scripthash_body(chain, mempool));
        for markup in [&address, &scripthash] {
            assert_no_balance(markup);
            assert!(
                !markup.contains("Balance"),
                "null sum still painted a Balance label: {markup}"
            );
        }
    }
}

#[test]
fn balance_difference_is_signed() {
    let chain = r#","funded_txo_sum":1,"spent_txo_sum":4"#;
    let mempool = r#","funded_txo_sum":1,"spent_txo_sum":4"#;
    let address = address_markup(&address_body(chain, mempool));
    let scripthash = scripthash_markup(&scripthash_body(chain, mempool));
    assert_one_balance(&address, "-6");
    assert_one_balance(&scripthash, "-6");
    assert!(!address.contains("Balance 0"), "{address}");
    assert!(!scripthash.contains("Balance 0"), "{scripthash}");
    assert!(address.contains("Confirmed balance -3"), "{address}");
    assert!(scripthash.contains("Confirmed balance -3"), "{scripthash}");
    assert!(
        address.contains("<p class=\"pending-amount\">Pending -3</p>"),
        "{address}"
    );
    assert!(
        scripthash.contains("<p class=\"pending-amount\">Pending -3</p>"),
        "{scripthash}"
    );
}

#[test]
fn scripthash_wire_parse_failure_does_not_invent_balance() {
    let body = r#"{"scripthash":"deadbeef","chain_stats":{"tx_count":4,"funded_txo_sum":5000,"spent_txo_sum":1000},"mempool_stats":{"tx_count":1,"funded_txo_count":4,"spent_txo_count":1,"funded_txo_sum":20,"spent_txo_sum":5}}"#;
    let markup = scripthash_markup(body);
    assert!(markup.contains("scripthash"), "{markup}");
    assert!(
        markup.contains("<p class=\"funded-sum\">5000</p>"),
        "{markup}"
    );
    assert_no_balance(&markup);
}
