//! Block-details Weight label on the block page.
//!
//! Angular `block.component.html` paints `<td i18n="block.weight">Weight</td>`
//! beside the block weight. That cell is a block-details label, not the
//! transaction cell `Weight N WU`. The fixture weight is 4, the same integer
//! `block_heading.rs` already sends. The value cell keeps that integer.

use splora_web::{
    block_details_markup, block_fields_text, block_page_weight_markup, block_rows_markup,
    parse_block_json,
};

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const WEIGHT: u32 = 4;
const LABEL: &str = "<td>Weight</td>";

fn block_json() -> String {
    format!(
        r#"{{"id":"{ID}","height":123,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":{WEIGHT},"merkle_root":"merkle","mediantime":1}}"#
    )
}

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
/// that merely names Weight does not count as the label.
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

/// Details table the block page paints, with the Weight row inserted the
/// way `block_page_weight_markup` writes it. Missing the call, or missing
/// the label cell, yields an empty string. That is the absent label.
fn painted_details(block_json: &str) -> String {
    let src = views_source();
    let page = compact(&code_without_comments(component_body(src, "BlockPage")));
    if !page.contains("block_page_weight_markup(&block_details_markup(") {
        return String::new();
    }
    let Some(body) = function_body(src, "block_page_weight_markup") else {
        return String::new();
    };
    let code = code_without_comments(body);
    let literals = string_literals(&code);
    let Some(template) = literals.iter().find(|lit| lit.contains(LABEL)) else {
        return String::new();
    };
    if !template.contains("{}") || !literals.iter().any(|lit| lit == "</tbody>") {
        return String::new();
    }
    let Ok(block) = parse_block_json(block_json) else {
        return String::new();
    };
    let row = template.replace("{}", &block.weight.to_string());
    let details = block_details_markup(block_json, true);
    let Some(at) = details.rfind("</tbody>") else {
        return String::new();
    };
    let mut out = String::with_capacity(details.len() + row.len());
    out.push_str(&details[..at]);
    out.push_str(&row);
    out.push_str(&details[at..]);
    out
}

#[test]
fn block_page_paints_weight() {
    let json = block_json();
    let markup = painted_details(&json);
    assert!(markup.contains(LABEL), "Weight is absent: {markup}");
    assert!(
        markup.contains("id=\"details\""),
        "Weight was not in the block details table: {markup}"
    );
    assert_eq!(
        markup.matches(LABEL).count(),
        1,
        "Weight was not painted once: {markup}"
    );
    let value = format!("<td>{WEIGHT}</td>");
    assert!(
        markup.contains(&value),
        "the raw weight integer was dropped: {markup}"
    );
    assert!(
        markup.contains(&format!("{LABEL}{value}")),
        "Weight replaced the raw integer: {markup}"
    );
    assert!(
        !markup.contains("Weight 4") && !markup.contains("Weight 7 WU"),
        "the transaction weight line was painted: {markup}"
    );
    assert!(
        markup.contains("<td>Version</td>"),
        "Weight was not beside the other block detail labels: {markup}"
    );

    let fields = block_fields_text(&json, "[]");
    assert!(
        fields.contains(&format!("weight {WEIGHT}")),
        "the raw weight integer was dropped: {fields}"
    );
    let rows = block_rows_markup(&json, "[]");
    assert!(
        !rows.contains("Weight"),
        "the block row weight cell gained the word Weight: {rows}"
    );
}

/// Whole line from `block.component.html` after block details open.
///
/// A line equal to Weight passes. The label cell stays. Closed details omit
/// the line and keep the cell. An unreadable body omits the line. The block
/// row does not gain the word. This is not a grouped `Weight ` label.
#[test]
fn block_page_paints_weight_as_its_own_line() {
    const LINE: &str = "Weight";
    let json = block_json();
    let open_details = block_details_markup(&json, true);
    let open = block_page_weight_markup(&open_details, &json);
    assert!(
        open.lines().any(|line| line == LINE),
        "no line equals {LINE}: {open}"
    );
    assert_eq!(
        open.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {open}"
    );
    assert!(
        open.contains("<td>Weight</td>"),
        "open details dropped the Weight cell: {open}"
    );
    assert!(
        open.contains(&format!("<td>{WEIGHT}</td>")),
        "open details dropped the raw weight: {open}"
    );
    assert!(
        !open.lines().any(|line| line.starts_with("Weight ")),
        "a grouped Weight label was painted: {open}"
    );
    let rows = block_rows_markup(&json, "[]");
    assert!(!rows.contains(LINE), "the block row gained {LINE}: {rows}");
    let closed_details = block_details_markup(&json, false);
    let closed = block_page_weight_markup(&closed_details, &json);
    assert!(
        closed.contains("<td>Weight</td>"),
        "closed details dropped the Weight cell: {closed}"
    );
    assert!(
        !closed.lines().any(|line| line == LINE),
        "closed details painted {LINE}: {closed}"
    );
    let unreadable = block_details_markup("not-json", true);
    let omitted = block_page_weight_markup(&unreadable, "not-json");
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "an unreadable body painted {LINE}: {omitted}"
    );
}
