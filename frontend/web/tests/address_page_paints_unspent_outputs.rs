//! Address heading above the unspent-output rows.
//!
//! Angular `address.component.html` paints
//! `<h2>Unspent Outputs</h2>` when `GET /address/:addr/utxo` has more than
//! two rows. The address route has no query string. Two rows, an empty
//! list, or a body that does not parse leaves the rows unchanged.

use splora_web::{address_rows_markup, address_unspent_outputs_markup, parse_utxos_json};

const LABEL: &str = "<h2>Unspent Outputs</h2>";
const UTXO_TABLE: &str = "<table class=\"utxo-rows\">";
const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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
/// that merely names Unspent Outputs does not count as the heading.
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

fn one_utxo(vout: u32, value: u64) -> String {
    format!(
        r#"{{"txid":"{TXID}","vout":{vout},"status":{{"confirmed":true,"block_height":1}},"value":{value}}}"#
    )
}

fn utxos_json(count: usize) -> String {
    let rows: Vec<String> = (0..count)
        .map(|index| one_utxo(index as u32, 1000 + index as u64))
        .collect();
    format!("[{}]", rows.join(","))
}

/// Rows the address page paints, with Unspent Outputs inserted the way
/// `address_unspent_outputs_markup` writes it. Missing the call, the parse,
/// or the heading yields the rows without that heading.
fn painted_address(utxo_json: &str) -> String {
    let rows = address_rows_markup("", "[]", utxo_json);
    let src = views_source();
    let page = compact(&code_without_comments(component_body(src, "AddressPage")));
    if !page.contains("address_unspent_outputs_markup(&painted,&utxos)") {
        return rows;
    }
    let Some(body) = function_body(src, "address_unspent_outputs_markup") else {
        return rows;
    };
    let code = code_without_comments(body);
    let literals = string_literals(&code);
    if !literals.iter().any(|lit| lit.contains(LABEL)) {
        return rows;
    }
    if !code.contains("parse_utxos_json(") {
        return rows;
    }
    let Ok(utxos) = parse_utxos_json(utxo_json) else {
        return rows;
    };
    if utxos.len() <= 2 {
        return rows;
    }
    let Some(at) = rows.find(UTXO_TABLE) else {
        return rows;
    };
    let mut out = String::with_capacity(rows.len() + LABEL.len());
    out.push_str(&rows[..at]);
    out.push_str(LABEL);
    out.push_str(&rows[at..]);
    out
}

#[test]
fn address_page_paints_unspent_outputs() {
    let markup = painted_address(&utxos_json(3));
    assert!(
        markup.contains(LABEL),
        "Unspent Outputs is absent: {markup}"
    );
    assert_eq!(
        markup.matches(LABEL).count(),
        1,
        "Unspent Outputs was not painted once: {markup}"
    );
    let table_at = markup.find(UTXO_TABLE).expect("the utxo table was dropped");
    let heading_at = markup.find(LABEL).expect("heading");
    assert!(
        heading_at < table_at,
        "Unspent Outputs was not above the utxo rows: {markup}"
    );
    assert!(
        markup.contains("<td class=\"value\">1000</td>"),
        "the first raw value was dropped: {markup}"
    );
    assert!(
        markup.contains("<td class=\"value\">1002</td>"),
        "the third raw value was dropped: {markup}"
    );
    let two = painted_address(&utxos_json(2));
    assert!(
        !two.contains(LABEL),
        "two utxos painted Unspent Outputs: {two}"
    );
    assert!(
        two.contains("<td class=\"value\">1001</td>"),
        "the two-row value was dropped: {two}"
    );
    let empty = painted_address("[]");
    assert!(
        !empty.contains(LABEL),
        "an empty utxo list painted Unspent Outputs: {empty}"
    );
    let unparsed = painted_address("not-json");
    assert!(
        !unparsed.contains(LABEL),
        "an unparsed body painted Unspent Outputs: {unparsed}"
    );
}

/// Whole line `Unspent Outputs` when more than two utxos parse.
///
/// GPUI `address_screen_paints_unspent_outputs` requires one line equal to
/// Unspent Outputs. The h2 stays. Two rows, an empty list, and an unparsed
/// body omit the line. This is not Unspent TXOs.
#[test]
fn address_page_paints_unspent_outputs_as_its_own_line() {
    const LINE: &str = "Unspent Outputs";
    let body = utxos_json(3);
    let markup = address_unspent_outputs_markup(&address_rows_markup("", "[]", &body), &body);
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(markup.contains(LABEL), "the heading was dropped: {markup}");
    assert!(
        !markup
            .lines()
            .any(|line| line.starts_with("Unspent Outputs ")),
        "{LINE} was painted as a longer sentence: {markup}"
    );
    assert_ne!(LINE, "Unspent TXOs");
    let line_at = markup.lines().position(|line| line == LINE);
    let table_at = markup.lines().position(|line| line.contains(UTXO_TABLE));
    assert!(
        line_at.is_some() && table_at.is_some() && line_at < table_at,
        "{LINE} was not above the utxo rows: {markup}"
    );
    let two_body = utxos_json(2);
    let two = address_unspent_outputs_markup(&address_rows_markup("", "[]", &two_body), &two_body);
    assert!(
        !two.lines().any(|line| line == LINE),
        "two utxos painted {LINE}: {two}"
    );
    assert!(
        !address_unspent_outputs_markup(&address_rows_markup("", "[]", "[]"), "[]")
            .lines()
            .any(|line| line == LINE)
    );
    assert!(
        !address_unspent_outputs_markup(&address_rows_markup("", "[]", "not-json"), "not-json")
            .lines()
            .any(|line| line == LINE)
    );
}
