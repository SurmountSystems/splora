//! Transactions column header on the blocks list.
//!
//! Angular `blocks-list.component.html` paints the header Transactions
//! beside `block.tx_count` when the list is not the mempool module. The
//! route is `blocks` with no query string. The value cell stays the raw
//! integer inside the block row. That row does not gain this word.

use splora_web::{blocks_list_requests, blocks_rows_markup};

const HEADER: &str = "<th class=\"txs\">Transactions</th>";
const WORD: &str = "Transactions";
const BLOCK_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HEIGHT: u32 = 842001;
const TIMESTAMP: u32 = 1_700_000_000;
const TX_COUNT: u32 = 17;

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

/// Body inside the function braces, not the next function.
fn component_body<'a>(src: &'a str, name: &str) -> &'a str {
    let marker = format!("fn {name}(");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing from views.rs"));
    let after_name = &src[start..];
    let brace_rel = after_name
        .find('{')
        .unwrap_or_else(|| panic!("{name} has no body"));
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
    panic!("{name} body is not closed");
}

/// Line and block comments drop out. String literals stay, so a comment
/// that merely names Transactions does not count as the header.
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

/// Text of one element. Leptos wraps the word in quotes.
fn element_text(element: &str, open: &str, close: &str) -> String {
    let inner = element
        .strip_prefix(open)
        .and_then(|rest| rest.strip_suffix(close))
        .unwrap_or(element);
    let inner = inner.trim();
    let inner = inner
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(inner);
    inner.trim().to_string()
}

fn element_texts(page: &str, open: &str, close: &str) -> Vec<String> {
    let mut texts = Vec::new();
    let mut rest = page;
    while let Some(start) = rest.find(open) {
        let after = &rest[start..];
        let end = after
            .find(close)
            .map(|index| index + close.len())
            .unwrap_or_else(|| panic!("{open} is not closed: {page}"));
        texts.push(element_text(&after[..end], open, close));
        rest = &after[end..];
    }
    texts
}

fn blocks_json() -> String {
    format!(
        r#"[{{"id":"{BLOCK_ID}","height":{HEIGHT},"version":1,"timestamp":{TIMESTAMP},"tx_count":{TX_COUNT},"size":20,"weight":80,"merkle_root":"mm","mediantime":8}}]"#
    )
}

/// Header plus the block row the blocks page already paints.
/// The row markup is not given this word.
fn painted_blocks_page(page: &str, blocks_json: &str) -> String {
    let headers = element_texts(page, "<th class=\"txs\">", "</th>");
    let mut markup = String::new();
    if headers.iter().filter(|text| text.as_str() == WORD).count() == 1 {
        markup.push_str(HEADER);
    }
    if page.contains("blocks_rows_markup(") {
        markup.push_str(&blocks_rows_markup(blocks_json));
    }
    markup
}

#[test]
fn blocks_page_paints_transactions() {
    let src = views_source();
    assert!(
        src.contains("path!(\"blocks\") view=BlocksPage"),
        "path blocks does not open BlocksPage"
    );
    let page = code_without_comments(component_body(src, "BlocksPage"));
    let json = blocks_json();
    let markup = painted_blocks_page(&page, &json);
    assert!(markup.contains(HEADER), "Transactions is absent: {markup}");
    assert_eq!(
        markup.matches(HEADER).count(),
        1,
        "Transactions was not painted once: {markup}"
    );
    // The header cell and the whole line each contain the word.
    assert_eq!(
        markup.matches(WORD).count(),
        2,
        "Transactions was not the header plus one line: {markup}"
    );
    let rows = blocks_rows_markup(&json);
    assert_eq!(
        rows.lines().filter(|line| *line == WORD).count(),
        1,
        "the blocks list did not paint one Transactions line: {rows}"
    );
    assert!(
        !rows.replacen("\nTransactions\n", "", 1).contains(WORD),
        "the block row gained the word Transactions: {rows}"
    );
    assert!(
        rows.contains("<td class=\"tx-count-label\">TXs</td>"),
        "TXs was dropped: {rows}"
    );
    assert!(
        markup.contains(&format!("<td class=\"tx-count\">{TX_COUNT}</td>")),
        "the raw tx_count integer was dropped: {markup}"
    );
    assert!(
        !markup.contains(&format!("{TX_COUNT},")) && !markup.contains("1,7"),
        "tx_count gained a thousands separator: {markup}"
    );
    assert!(
        markup.contains(&format!("<td class=\"time\">{TIMESTAMP}</td>")),
        "the raw timestamp was dropped: {markup}"
    );
    assert!(
        !markup.contains("ago"),
        "the timestamp became a clock: {markup}"
    );
    let calls = blocks_list_requests();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "GET");
    assert_eq!(calls[0].path, "/blocks");
    assert!(!calls[0].path.contains('?'), "{}", calls[0].path);
    assert!(!calls[0].path.contains('#'), "{}", calls[0].path);
}

/// Whole line beside the Transactions header from `GET /blocks`.
///
/// A line equal to Transactions passes. The header cell stays. An empty
/// list omits the line. The line comes before TXs.
#[test]
fn blocks_list_paints_transactions_as_its_own_line() {
    const LINE: &str = "Transactions";
    let markup = blocks_rows_markup(&blocks_json());
    assert!(
        markup.contains("<td class=\"tx-count-label\">TXs</td>"),
        "the blocks list was not rendered: {markup}"
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
        !markup.lines().any(|line| line.starts_with("Transactions ")),
        "{LINE} was painted as a longer sentence: {markup}"
    );
    let lines: Vec<&str> = markup.lines().collect();
    let header_at = lines.iter().position(|line| *line == LINE);
    let txs_at = lines.iter().position(|line| *line == "TXs");
    assert!(
        header_at.is_some() && txs_at.is_some() && header_at < txs_at,
        "{LINE} was not before TXs: {markup}"
    );
    let empty = blocks_rows_markup("[]");
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty list painted {LINE}: {empty}"
    );
    let unparsed = blocks_rows_markup("not-json");
    assert!(
        !unparsed.lines().any(|line| line == LINE),
        "an unparsed body painted {LINE}: {unparsed}"
    );
}
