//! Address heading above the loaded transaction rows.
//!
//! Angular `address.component.html` paints
//! `<h2>Balance History</h2>` when `GET /address/:addr/txs` has more than
//! two rows. The address route has no query string. Two rows, an empty
//! list, or a body that does not parse leaves the rows unchanged.

use splora_web::{address_rows_markup, parse_transactions_json};

const LABEL: &str = "<h2>Balance History</h2>";
const TX_TABLE: &str = "<table class=\"tx-rows\">";

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

/// Body inside the function braces, not the next function.
fn function_body<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("fn {name}(");
    let start = src.find(&marker)?;
    let after_name = &src[start..];
    let brace_rel = after_name.find('{')?;
    let body_start = start + brace_rel + 1;
    let mut depth = 1i32;
    for (offset, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&src[body_start..body_start + offset]);
                }
            }
            _ => {}
        }
    }
    None
}

fn component_body<'a>(src: &'a str, name: &str) -> &'a str {
    function_body(src, name).unwrap_or_else(|| panic!("{name} is missing from views.rs"))
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

/// Line and block comments drop out. String literals stay, so a comment
/// that merely names Balance History does not count as the heading.
fn code_without_comments(src: &str) -> String {
    let mut out = String::new();
    let mut chars = src.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for next in chars.by_ref() {
                if next == '\n' {
                    out.push('\n');
                    break;
                }
            }
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(next) = chars.next() {
                if next == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
            }
            continue;
        }
        if ch == '"' {
            out.push('"');
            while let Some(next) = chars.next() {
                out.push(next);
                if next == '\\' {
                    if let Some(escaped) = chars.next() {
                        out.push(escaped);
                    }
                    continue;
                }
                if next == '"' {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn string_literals(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '"' {
            continue;
        }
        let mut literal = String::new();
        while let Some(next) = chars.next() {
            if next == '\\' {
                if let Some(escaped) = chars.next() {
                    literal.push(escaped);
                }
                continue;
            }
            if next == '"' {
                break;
            }
            literal.push(next);
        }
        out.push(literal);
    }
    out
}

fn one_tx(index: u64) -> String {
    let txid = format!("{:064x}", index + 1);
    let fee = index + 1;
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[],"vout":[],"size":10,"weight":40,"sigops":0,"fee":{fee},"status":{{"confirmed":true,"block_height":2}}}}"#
    )
}

fn txs_json(count: usize) -> String {
    let rows: Vec<String> = (0..count as u64).map(one_tx).collect();
    format!("[{}]", rows.join(","))
}

/// Rows the address page paints, with Balance History inserted the way
/// `address_balance_history_markup` writes it. Missing the call, the parse,
/// or the heading yields the rows without that heading.
fn painted_address(txs_json: &str) -> String {
    let rows = address_rows_markup("", txs_json, "[]");
    let src = views_source();
    let page = compact(&code_without_comments(component_body(src, "AddressPage")));
    if !page.contains("address_balance_history_markup(&with_outputs,&txs)") {
        return rows;
    }
    let Some(body) = function_body(src, "address_balance_history_markup") else {
        return rows;
    };
    let code = code_without_comments(body);
    let literals = string_literals(&code);
    if !literals.iter().any(|lit| lit.contains(LABEL)) {
        return rows;
    }
    if !code.contains("parse_transactions_json(") {
        return rows;
    }
    let Ok(txs) = parse_transactions_json(txs_json) else {
        return rows;
    };
    if txs.len() <= 2 {
        return rows;
    }
    let Some(at) = rows.find(TX_TABLE) else {
        return rows;
    };
    let mut out = String::with_capacity(rows.len() + LABEL.len());
    out.push_str(&rows[..at]);
    out.push_str(LABEL);
    out.push_str(&rows[at..]);
    out
}

#[test]
fn address_page_paints_balance_history() {
    let markup = painted_address(&txs_json(3));
    assert!(
        markup.contains(LABEL),
        "Balance History is absent: {markup}"
    );
    assert_eq!(
        markup.matches(LABEL).count(),
        1,
        "Balance History was not painted once: {markup}"
    );
    let table_at = markup
        .find(TX_TABLE)
        .expect("the transaction table was dropped");
    let heading_at = markup.find(LABEL).expect("heading");
    assert!(
        heading_at < table_at,
        "Balance History was not above the transaction rows: {markup}"
    );
    assert!(
        markup.contains("<td class=\"fee\">1</td>"),
        "the first raw fee was dropped: {markup}"
    );
    assert!(
        markup.contains("<td class=\"fee\">3</td>"),
        "the third raw fee was dropped: {markup}"
    );
    let two = painted_address(&txs_json(2));
    assert!(
        !two.contains(LABEL),
        "two transactions painted Balance History: {two}"
    );
    assert!(
        two.contains("<td class=\"fee\">2</td>"),
        "the two-row fee was dropped: {two}"
    );
    let empty = painted_address("[]");
    assert!(
        !empty.contains(LABEL),
        "an empty transaction list painted Balance History: {empty}"
    );
    let unparsed = painted_address("not-json");
    assert!(
        !unparsed.contains(LABEL),
        "an unparsed body painted Balance History: {unparsed}"
    );
}
