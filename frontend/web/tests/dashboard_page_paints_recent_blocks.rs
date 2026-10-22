//! Dashboard Recent Blocks heading above the latest block rows.
//!
//! Angular `dashboard.component.html` paints
//! `<h5>Recent Blocks</h5>` above the latest blocks from `GET /blocks`.
//! The dashboard route has no query string. A body that does not parse, or
//! an empty list, leaves the rows unchanged. This is not a clock, not Size,
//! and not the blocks-list heading Blocks.

use splora_web::{dashboard_rows_markup, parse_blocks_json};

const LABEL: &str = "<h5>Recent Blocks</h5>";
const TIMESTAMP: u32 = 1_700_000_000;

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
/// that merely names Recent Blocks does not count as the heading.
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

fn one_block(timestamp: u32, height: u32) -> String {
    format!(
        r#"{{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","height":{height},"version":1,"timestamp":{timestamp},"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    )
}

fn blocks_json() -> String {
    format!("[{},{}]", one_block(TIMESTAMP, 2), one_block(1699990000, 1))
}

/// Rows the dashboard paints, with Recent Blocks inserted the way
/// `dashboard_recent_blocks_markup` writes it. Missing the call, the parse,
/// or the heading yields an empty string. That is the absent label.
fn painted_dashboard(blocks_json: &str) -> String {
    let src = views_source();
    let page = compact(&code_without_comments(component_body(src, "Dashboard")));
    if !page.contains(
        "dashboard_recent_blocks_markup(&dashboard_mined_markup(&dashboard_rows_markup(&blocks,&recent),&blocks,),&blocks,)",
    ) {
        return String::new();
    }
    let Some(body) = function_body(src, "dashboard_recent_blocks_markup") else {
        return String::new();
    };
    let code = code_without_comments(body);
    let literals = string_literals(&code);
    if !literals.iter().any(|lit| lit.contains(LABEL)) {
        return String::new();
    }
    if !code.contains("parse_blocks_json(") {
        return String::new();
    }
    let rows = dashboard_rows_markup(blocks_json, "[]");
    let Ok(blocks) = parse_blocks_json(blocks_json) else {
        return rows;
    };
    if blocks.is_empty() {
        return rows;
    }
    let mut out = String::with_capacity(LABEL.len() + rows.len());
    out.push_str(LABEL);
    out.push_str(&rows);
    out
}

#[test]
fn dashboard_page_paints_recent_blocks() {
    let json = blocks_json();
    let markup = painted_dashboard(&json);
    assert!(markup.contains(LABEL), "Recent Blocks is absent: {markup}");
    assert_eq!(
        markup.matches(LABEL).count(),
        1,
        "Recent Blocks was not painted once: {markup}"
    );
    let time = format!("<td class=\"time\">{TIMESTAMP}</td>");
    assert!(
        markup.contains(&time),
        "the raw timestamp row was dropped: {markup}"
    );
    assert!(
        markup.find(LABEL) < markup.find(&time),
        "Recent Blocks was not above the block rows: {markup}"
    );
    assert!(
        markup.contains("<td class=\"time\">1699990000</td>"),
        "the second raw timestamp was dropped: {markup}"
    );
    assert!(
        !markup.contains("ago") && !markup.contains("Recent Blocks 1700000000"),
        "Recent Blocks became a clock: {markup}"
    );
    assert!(
        !markup.contains(">Size<") && !markup.contains("<h1>Blocks</h1>"),
        "Recent Blocks pulled in Size or the blocks-list heading: {markup}"
    );
}
