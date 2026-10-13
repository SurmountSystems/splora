//! Address highlighting and visible OP_RETURN type text.
//!
//! Cypress hovers a block-page link and expects a tooltip. These tests require
//! the script type `op_return` as text in the row. They do not require a hover.
//! Highlighting uses `scriptpubkey_address` on the transaction JSON.

use splora_web::{address_highlight_markup, block_rows_markup, transaction_row_markup};

const PAGE: &str = "1wiz32gbHZwMzJCRHMGehJuBgsMTPdaCa";
const OTHER: &str = "bc1qother000000000000000000000000000000";

fn vin(address: &str) -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{address}","value":1}},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}"#
    )
}

fn vout(address: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"{address}","value":2}}"#
    )
}

fn tx(txid: &str, vins: &str, vouts: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vins}],"vout":[{vouts}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":2}}}}"#
    )
}

fn txs(items: &[&str]) -> String {
    format!("[{}]", items.join(","))
}

fn tx_section(markup: &str, index: usize) -> String {
    let needle = format!("data-cy=\"tx-{index}\"");
    let start = markup
        .find(&needle)
        .unwrap_or_else(|| panic!("missing {needle} in {markup}"));
    let rest = &markup[start..];
    let next = rest[needle.len()..]
        .find("data-cy=\"tx-")
        .map(|offset| needle.len() + offset)
        .unwrap_or(rest.len());
    rest[..next].to_string()
}

fn table_section(section: &str, class: &str) -> String {
    let needle = format!("<table class=\"{class}\">");
    let start = section
        .find(&needle)
        .unwrap_or_else(|| panic!("missing {class} in {section}"));
    let rest = &section[start..];
    let end = rest.find("</table>").expect("table is not closed");
    rest[..end].to_string()
}

fn highlight_texts(section: &str) -> Vec<String> {
    let needle = "<span class=\"highlight\">";
    let mut rest = section;
    let mut out = Vec::new();
    while let Some(start) = rest.find(needle) {
        let after = &rest[start + needle.len()..];
        let end = after.find("</span>").expect("highlight is not closed");
        let inner = &after[..end];
        let mut text = String::new();
        let mut chars = inner.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '<' {
                for next in chars.by_ref() {
                    if next == '>' {
                        break;
                    }
                }
            } else {
                text.push(ch);
            }
        }
        out.push(text);
        rest = &after[end..];
    }
    out
}

fn views_address_page() -> &'static str {
    let src = include_str!("../src/views.rs");
    let marker = "fn AddressPage(";
    let start = src
        .find(marker)
        .expect("AddressPage is missing from views.rs");
    let after_name = &src[start..];
    let brace_rel = after_name.find('{').expect("AddressPage has no body");
    let body_start = start + brace_rel + 1;
    let mut depth = 1i32;
    for (offset, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[body_start..body_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("AddressPage body is not closed");
}

fn op_return_tx(coinbase: bool) -> String {
    let coinbase_json = if coinbase { "true" } else { "false" };
    format!(
        r#"{{"txid":"opreturntx","version":1,"locktime":0,"vin":[{{"txid":"prev","vout":0,"prevout":null,"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","is_coinbase":{coinbase_json},"sequence":1}}],"vout":[{{"scriptpubkey":"6a026869","scriptpubkey_asm":"OP_RETURN 6869","scriptpubkey_type":"op_return","value":0}}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":2}}}}"#
    )
}

fn assert_op_return_text(markup: &str) {
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">op_return</span>"),
        "scriptpubkey_type op_return is not visible text: {markup}"
    );
    assert!(
        markup.contains("<span class=\"op-return\">OP_RETURN</span>"),
        "OP_RETURN label is not visible text: {markup}"
    );
    assert!(!markup.contains("6869"), "script payload leaked: {markup}");
    assert!(!markup.contains("deadbeef"), "script hex leaked: {markup}");
    assert!(
        !markup.contains("OP_CHECKSIG"),
        "script asm leaked: {markup}"
    );
    assert!(
        !markup.contains("title="),
        "hover title is not the visible text: {markup}"
    );
}

#[test]
fn op_return_output_is_visible_on_the_block_transaction_row() {
    let txs = format!("[{}]", op_return_tx(false));
    let block = r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle","previousblockhash":null,"mediantime":1699990000,"nonce":1,"bits":1,"difficulty":1}"#;
    assert_op_return_text(&block_rows_markup(block, &txs));
    assert_op_return_text(&transaction_row_markup(&op_return_tx(false)));
}

#[test]
fn op_return_on_a_coinbase_transaction_is_visible_text() {
    assert_op_return_text(&transaction_row_markup(&op_return_tx(true)));
}

#[test]
fn highlights_single_input_address() {
    let body = txs(&[&tx("aa", &vin(PAGE), &vout(OTHER))]);
    let markup = address_highlight_markup(PAGE, &body);
    let vin_table = table_section(&tx_section(&markup, 0), "table-tx-vin");
    let vout_table = table_section(&tx_section(&markup, 0), "table-tx-vout");
    assert_eq!(highlight_texts(&vin_table), vec![PAGE.to_string()]);
    assert!(highlight_texts(&vout_table).is_empty(), "{vout_table}");
    assert!(markup.contains(OTHER), "{markup}");
}

#[test]
fn highlights_multiple_input_addresses() {
    let ignored = tx("aa", &vin(OTHER), &vout(OTHER));
    let also = tx("bb", &vin(OTHER), &vout(PAGE));
    let matched = tx("cc", &format!("{},{}", vin(PAGE), vin(PAGE)), &vout(OTHER));
    let markup = address_highlight_markup(PAGE, &txs(&[&ignored, &also, &matched]));
    let vin_table = table_section(&tx_section(&markup, 2), "table-tx-vin");
    assert_eq!(
        highlight_texts(&vin_table),
        vec![PAGE.to_string(), PAGE.to_string()]
    );
}

#[test]
fn highlights_input_and_output_in_the_same_transaction() {
    let address = "bc1q03u63r6hm7a3v6em58zdqtp446w2pw30nm63mv";
    let skipped = tx("aa", &vin(OTHER), &vout(OTHER));
    let matched = tx("bb", &vin(address), &vout(address));
    let markup = address_highlight_markup(address, &txs(&[&skipped, &matched]));
    let section = tx_section(&markup, 1);
    assert_eq!(
        highlight_texts(&table_section(&section, "table-tx-vin")),
        vec![address.to_string()]
    );
    assert_eq!(
        highlight_texts(&table_section(&section, "table-tx-vout")),
        vec![address.to_string()]
    );
}

#[test]
fn highlights_single_output_address() {
    let skipped = tx("aa", &vin(PAGE), &vout(OTHER));
    let matched = tx("bb", &vin(OTHER), &vout(PAGE));
    let markup = address_highlight_markup(PAGE, &txs(&[&skipped, &matched]));
    let section = tx_section(&markup, 1);
    assert_eq!(
        highlight_texts(&table_section(&section, "table-tx-vout")),
        vec![PAGE.to_string()]
    );
    assert!(
        highlight_texts(&table_section(&section, "table-tx-vin")).is_empty(),
        "{section}"
    );
}

#[test]
fn highlights_multiple_output_addresses() {
    let address = "1F3Q3sQmiGsWSqK5K6T9tYnX8yqzYRgQbe";
    let skipped = tx("aa", &vin(OTHER), &vout(OTHER));
    let matched = tx(
        "bb",
        &vin(OTHER),
        &format!("{},{}", vout(address), vout(address)),
    );
    let markup = address_highlight_markup(address, &txs(&[&skipped, &matched]));
    let vout_table = table_section(&tx_section(&markup, 1), "table-tx-vout");
    assert_eq!(
        highlight_texts(&vout_table),
        vec![address.to_string(), address.to_string()]
    );
}

#[test]
fn address_page_paints_the_highlight_markup() {
    let page = views_address_page();
    assert!(
        page.contains("address_highlight_markup("),
        "AddressPage does not highlight the route address"
    );
    assert!(!page.contains("Fiat price last updated"), "{page}");
}
