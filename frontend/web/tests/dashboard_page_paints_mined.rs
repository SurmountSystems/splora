//! Dashboard Mined label beside the block timestamp.
//!
//! Angular `dashboard.component.html` paints
//! `<th class="table-cell-mined">Mined</th>` beside `block.timestamp` from
//! `GET /blocks`. The dashboard route has no query string. The value cell
//! stays the existing raw timestamp. A body that does not parse, or an empty
//! list, leaves the rows unchanged. This is not a clock, not Size, and not
//! the blocks-list Timestamp label.

use splora_web::{blocks_rows_markup, dashboard_rows_markup, parse_blocks_json};

const LABEL: &str = "<td class=\"table-cell-mined\">Mined</td>";
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
/// that merely names Mined does not count as the label.
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

/// Rows the dashboard paints, with Mined inserted the way
/// `dashboard_mined_markup` writes it. Missing the call, the parse, or the
/// label cell yields an empty string. That is the absent label.
fn painted_dashboard(blocks_json: &str) -> String {
    let src = views_source();
    let page = compact(&code_without_comments(component_body(src, "Dashboard")));
    if !page.contains("dashboard_mined_markup(&dashboard_rows_markup(&blocks,&recent),&blocks,)") {
        return String::new();
    }
    let Some(body) = function_body(src, "dashboard_mined_markup") else {
        return String::new();
    };
    let code = code_without_comments(body);
    let literals = string_literals(&code);
    if !literals.iter().any(|lit| lit.contains(LABEL)) {
        return String::new();
    }
    if !literals.iter().any(|lit| lit == "<td class=\"time\">") {
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
    let needle = "<td class=\"time\">";
    let Some(at) = rows.find(needle) else {
        return rows;
    };
    let mut out = String::with_capacity(rows.len() + LABEL.len());
    out.push_str(&rows[..at]);
    out.push_str(LABEL);
    out.push_str(&rows[at..]);
    out
}

#[test]
fn dashboard_page_paints_mined() {
    let json = blocks_json();
    let markup = painted_dashboard(&json);
    assert!(markup.contains(LABEL), "Mined is absent: {markup}");
    assert_eq!(
        markup.matches(LABEL).count(),
        1,
        "Mined was not painted once: {markup}"
    );
    let time = format!("<td class=\"time\">{TIMESTAMP}</td>");
    assert!(
        markup.contains(&format!("{LABEL}{time}")),
        "Mined was not beside the raw timestamp: {markup}"
    );
    assert!(
        markup.contains("<td class=\"time\">1699990000</td>"),
        "the second raw timestamp was dropped: {markup}"
    );
    assert!(
        !markup.contains("Mined 1700000000") && !markup.contains("ago"),
        "Mined became a clock: {markup}"
    );
    assert!(
        !markup.contains(">Size<") && !markup.contains(">Timestamp<"),
        "Mined pulled in Size or Timestamp: {markup}"
    );
    let rows = blocks_rows_markup(&json);
    assert!(
        !rows.contains("Mined"),
        "the blocks list row gained Mined: {rows}"
    );

    let empty = painted_dashboard("[]");
    assert!(
        !empty.contains("Mined"),
        "an empty block list painted Mined: {empty}"
    );
    let bad = painted_dashboard("not-json");
    assert!(
        !bad.contains("Mined"),
        "an unreadable block list painted Mined: {bad}"
    );
}
