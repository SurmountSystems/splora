//! Confirmed balance on the address page is chain funded minus chain spent.
//!
//! A missing chain sum is not zero. Mempool sums are not that balance.

use splora_web::{address_fields_text, address_rows_markup};

fn element_text(markup: &str, class: &str) -> String {
    let open = format!("<p class=\"{class}\">");
    let start = markup
        .find(&open)
        .unwrap_or_else(|| panic!("missing {class} in {markup}"));
    assert_eq!(
        markup.matches(&open).count(),
        1,
        "more than one {class} in {markup}"
    );
    let after = &markup[start + open.len()..];
    let end = after
        .find("</p>")
        .unwrap_or_else(|| panic!("{class} is not closed: {markup}"));
    after[..end].to_string()
}

fn paint(body: &str) -> (String, String) {
    (
        address_rows_markup(body, "", ""),
        address_fields_text(body, "", ""),
    )
}

fn has_pair(text: &str, label: &str, value: &str) -> bool {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    tokens
        .windows(2)
        .any(|pair| pair[0] == label && pair[1] == value)
}

fn assert_balance(markup: &str, text: &str, amount: &str) {
    let phrase = format!("Confirmed balance {amount}");
    assert!(markup.contains(&phrase), "{markup}");
    assert!(text.contains(&phrase), "{text}");
    assert!(!markup.contains('¥'), "{markup}");
    assert!(!text.contains('¥'), "{text}");
    assert!(!phrase.contains(','));
}

fn assert_no_balance(markup: &str, text: &str) {
    assert!(!markup.contains("Confirmed balance"), "{markup}");
    assert!(!text.contains("Confirmed balance"), "{text}");
}

fn address_body(chain_sums: &str, mempool_sums: &str) -> String {
    format!(
        r#"{{"address":"bc1qexample","chain_stats":{{"tx_count":4,"funded_txo_count":4,"spent_txo_count":1{chain_sums}}},"mempool_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0{mempool_sums}}}}}"#
    )
}

#[test]
fn confirmed_balance_is_chain_funded_minus_chain_spent() {
    let body = address_body(
        r#","funded_txo_sum":5000,"spent_txo_sum":1000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":0"#,
    );
    let (markup, text) = paint(&body);
    assert_balance(&markup, &text, "4000");
    assert_eq!(element_text(&markup, "funded-sum"), "5000");
    assert_eq!(element_text(&markup, "chain-spent-sum"), "1000");
    assert!(has_pair(&text, "funded_txo_sum", "5000"), "{text}");
    assert!(has_pair(&text, "chain_spent_txo_sum", "1000"), "{text}");
    assert!(has_pair(&text, "mempool_funded_txo_sum", "20"), "{text}");
    assert!(has_pair(&text, "mempool_spent_txo_sum", "0"), "{text}");
}

#[test]
fn confirmed_balance_zero_when_both_chain_sums_are_zero() {
    let body = address_body(
        r#","funded_txo_sum":0,"spent_txo_sum":0"#,
        r#","funded_txo_sum":0,"spent_txo_sum":0"#,
    );
    let (markup, text) = paint(&body);
    assert_balance(&markup, &text, "0");
    assert_eq!(element_text(&markup, "funded-sum"), "0");
    assert_eq!(element_text(&markup, "chain-spent-sum"), "0");
    assert!(has_pair(&text, "funded_txo_sum", "0"), "{text}");
    assert!(has_pair(&text, "chain_spent_txo_sum", "0"), "{text}");
}

#[test]
fn confirmed_balance_is_negative_when_spent_exceeds_funded() {
    let body = address_body(
        r#","funded_txo_sum":1000,"spent_txo_sum":5000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":0"#,
    );
    let (markup, text) = paint(&body);
    assert_balance(&markup, &text, "-4000");
}

#[test]
fn omitted_chain_spent_sum_hides_confirmed_balance() {
    let body = address_body(
        r#","funded_txo_sum":5000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":0"#,
    );
    let (markup, text) = paint(&body);
    assert_no_balance(&markup, &text);
    assert_eq!(element_text(&markup, "funded-sum"), "5000");
    assert!(has_pair(&text, "funded_txo_sum", "5000"), "{text}");
    assert!(
        !markup.contains("<p class=\"chain-spent-sum\">"),
        "{markup}"
    );
    assert!(!has_pair(&text, "chain_spent_txo_sum", "0"), "{text}");
    assert!(
        !text
            .split_whitespace()
            .any(|token| token == "chain_spent_txo_sum"),
        "{text}"
    );
}

#[test]
fn omitted_chain_funded_sum_hides_confirmed_balance() {
    let body = address_body(
        r#","spent_txo_sum":1000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":0"#,
    );
    let (markup, text) = paint(&body);
    assert_no_balance(&markup, &text);
    assert_eq!(element_text(&markup, "chain-spent-sum"), "1000");
    assert!(has_pair(&text, "chain_spent_txo_sum", "1000"), "{text}");
    assert!(!markup.contains("<p class=\"funded-sum\">"), "{markup}");
    assert!(!has_pair(&text, "funded_txo_sum", "0"), "{text}");
    assert!(
        !text
            .split_whitespace()
            .any(|token| token == "funded_txo_sum"),
        "{text}"
    );
}

#[test]
fn null_chain_sum_hides_confirmed_balance() {
    let body = address_body(
        r#","funded_txo_sum":5000,"spent_txo_sum":null"#,
        r#","funded_txo_sum":20,"spent_txo_sum":0"#,
    );
    let (markup, text) = paint(&body);
    assert_no_balance(&markup, &text);
    assert_eq!(element_text(&markup, "funded-sum"), "5000");
    assert!(has_pair(&text, "funded_txo_sum", "5000"), "{text}");
    assert!(
        !markup.contains("<p class=\"chain-spent-sum\">"),
        "{markup}"
    );
    assert!(
        !text
            .split_whitespace()
            .any(|token| token == "chain_spent_txo_sum"),
        "{text}"
    );

    let funded_null = address_body(
        r#","funded_txo_sum":null,"spent_txo_sum":1000"#,
        r#","funded_txo_sum":20,"spent_txo_sum":0"#,
    );
    let (markup, text) = paint(&funded_null);
    assert_no_balance(&markup, &text);
    assert!(!markup.contains("<p class=\"funded-sum\">"), "{markup}");
    assert!(
        !text
            .split_whitespace()
            .any(|token| token == "funded_txo_sum"),
        "{text}"
    );
}

#[test]
fn mempool_sums_alone_do_not_paint_confirmed_balance() {
    let body = address_body("", r#","funded_txo_sum":20,"spent_txo_sum":7"#);
    let (markup, text) = paint(&body);
    assert_no_balance(&markup, &text);
    assert_eq!(element_text(&markup, "mempool-funded-sum"), "20");
    assert_eq!(element_text(&markup, "mempool-spent-sum"), "7");
    assert!(has_pair(&text, "mempool_funded_txo_sum", "20"), "{text}");
    assert!(has_pair(&text, "mempool_spent_txo_sum", "7"), "{text}");
    assert!(!markup.contains("<p class=\"funded-sum\">"), "{markup}");
    assert!(
        !markup.contains("<p class=\"chain-spent-sum\">"),
        "{markup}"
    );
    assert!(
        !text
            .split_whitespace()
            .any(|token| token == "funded_txo_sum"),
        "{text}"
    );
    assert!(
        !text
            .split_whitespace()
            .any(|token| token == "chain_spent_txo_sum"),
        "{text}"
    );
}
