//! Fees card title on the non-liquid dashboard.
//!
//! Angular `dashboard.component.html` paints the exact visible string
//! `Transaction Fees` when the network is not liquid. That dashboard loads
//! `GET /fee-estimates` with no query string. Liquid uses the same view
//! function and must not paint the heading.

use splora_web::{dashboard_paths, transaction_fees_heading};

const HEADING: &str = "Transaction Fees";
const MARKUP: &str = "<h2>Transaction Fees</h2>\nTransaction Fees\n";

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

/// Line and block comments drop out. String literals stay, so a comment
/// that merely names Transaction Fees does not count as the heading.
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

fn balanced_inside(code: &str, open_at: usize) -> Option<&str> {
    let mut depth = 0i32;
    for (offset, ch) in code[open_at..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&code[open_at + 1..open_at + offset]);
                }
            }
            _ => {}
        }
    }
    None
}

/// `if liquid { ... } else { ... }`. The liquid flag selects the arm.
fn branch_body(code: &str, liquid: bool) -> &str {
    let Some(if_at) = code.find("if liquid") else {
        return "";
    };
    let Some(open_rel) = code[if_at..].find('{') else {
        return "";
    };
    let open_at = if_at + open_rel;
    let Some(liquid_inner) = balanced_inside(code, open_at) else {
        return "";
    };
    if liquid {
        return liquid_inner;
    }
    let after_liquid = open_at + liquid_inner.len() + 2;
    if after_liquid > code.len() {
        return "";
    }
    let rest = &code[after_liquid..];
    let Some(else_rel) = rest.find("else") else {
        return "";
    };
    let Some(else_open_rel) = rest[else_rel..].find('{') else {
        return "";
    };
    balanced_inside(rest, else_rel + else_open_rel).unwrap_or("")
}

fn string_literals(src: &str) -> String {
    let mut out = String::new();
    let mut chars = src.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '"' {
            continue;
        }
        while let Some(next) = chars.next() {
            if next == '\\' {
                if let Some(escaped) = chars.next() {
                    out.push(escaped);
                }
                continue;
            }
            if next == '"' {
                break;
            }
            out.push(next);
        }
    }
    out
}

/// Markup from the one heading helper `Dashboard` calls.
///
/// Missing the fee-estimates load, or missing the liquid gate, yields an
/// empty string. That is the absent heading, not a second view.
fn dashboard_fees_markup(liquid: bool) -> String {
    let src = views_source();
    let page = code_without_comments(component_body(src, "Dashboard"));
    let compact: String = page.chars().filter(|ch| !ch.is_whitespace()).collect();
    if !compact.contains("fee_estimates_path(") {
        return String::new();
    }
    let call = "transaction_fees_heading(network.get()==Network::Liquid)";
    if !compact.contains(call) {
        return String::new();
    }
    let Some(body) = function_body(src, "transaction_fees_heading") else {
        return String::new();
    };
    let code = code_without_comments(body);
    if branch_body(&code, false).is_empty() {
        return String::new();
    }
    transaction_fees_heading(liquid)
}

fn heading_texts(markup: &str) -> Vec<String> {
    let mut texts = Vec::new();
    let mut rest = markup;
    while let Some(start) = rest.find("<h2>") {
        let after = &rest[start..];
        let Some(end) = after.find("</h2>") else {
            break;
        };
        let inner = after["<h2>".len()..end].trim();
        texts.push(inner.to_string());
        rest = &after[end + "</h2>".len()..];
    }
    texts
}

#[test]
fn nonliquid_dashboard_paints_transaction_fees_heading() {
    let paths = dashboard_paths();
    assert!(
        paths.iter().any(|path| *path == "/fee-estimates"),
        "the dashboard does not load GET /fee-estimates: {paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.contains('?') && !path.contains('#')),
        "a dashboard path gained a query or a fragment: {paths:?}"
    );
    let page = component_body(views_source(), "Dashboard");
    assert!(
        code_without_comments(page).contains("fee_estimates_path("),
        "the non-liquid dashboard does not load fee estimates"
    );

    let nonliquid = dashboard_fees_markup(false);
    // The h2 and the whole line each contain the words.
    assert_eq!(
        nonliquid.matches(HEADING).count(),
        2,
        "Transaction Fees is absent: {nonliquid}"
    );
    assert_eq!(
        nonliquid, MARKUP,
        "heading text was not exactly Transaction Fees: {nonliquid}"
    );
    assert_eq!(
        heading_texts(&nonliquid),
        vec![HEADING.to_string()],
        "expected one h2 whose text is Transaction Fees: {nonliquid}"
    );
    assert!(
        !nonliquid.contains("Transaction fee"),
        "the details row Transaction fee was painted: {nonliquid}"
    );
    assert!(
        !nonliquid.contains('.'),
        "the heading gained a period: {nonliquid}"
    );
    assert!(
        !nonliquid.contains("sat/vB"),
        "Transaction Fees gained sat/vB: {nonliquid}"
    );

    let liquid = dashboard_fees_markup(true);
    assert!(
        !liquid.contains(HEADING),
        "liquid painted Transaction Fees: {liquid}"
    );
    assert!(
        heading_texts(&liquid).is_empty(),
        "liquid painted a fees heading: {liquid}"
    );
}

/// Whole line beside the fees h2 from non-liquid `GET /fee-estimates`.
///
/// A line equal to Transaction Fees passes. The h2 stays. Liquid omits
/// the line. This is not the details row Transaction fee.
#[test]
fn nonliquid_dashboard_paints_transaction_fees_as_its_own_line() {
    const LINE: &str = "Transaction Fees";
    let markup = transaction_fees_heading(false);
    assert!(
        markup.contains("<h2>Transaction Fees</h2>"),
        "the h2 was dropped: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line.starts_with("Transaction Fees ")),
        "{LINE} was painted as a longer sentence: {markup}"
    );
    assert!(
        !markup.contains("Transaction fee"),
        "the details row Transaction fee was painted: {markup}"
    );
    assert!(!markup.contains("sat/vB"), "{LINE} gained sat/vB: {markup}");
    let liquid = transaction_fees_heading(true);
    assert!(
        !liquid.lines().any(|line| line == LINE),
        "liquid painted {LINE}: {liquid}"
    );
}
