//! Blocks page heading on the blocks list.
//!
//! Angular `blocks-list.component.html` paints
//! `<h1 i18n="master-page.blocks">Blocks</h1>` when the list is not a widget.
//! The route is `blocks` with no query string. `BlocksPage` paints that word
//! once as an h1 and keeps the block rows from `GET /blocks`. An h2 whose
//! text is Blocks does not satisfy this test. A comment that names the word
//! does not count.

use splora_web::{blocks_list_requests, blocks_rows_markup};

const WORD: &str = "Blocks";
const H1: &str = "<h1>Blocks</h1>";
const BLOCK_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HEIGHT: u32 = 842001;
const TIMESTAMP: u32 = 1_700_000_000;

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
/// that merely names Blocks does not count as the heading.
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

/// Text of one heading element. Leptos wraps the word in quotes.
/// `<h1>"Blocks"</h1>` and `<h1>Blocks</h1>` both yield Blocks.
fn heading_text(element: &str, open: &str, close: &str) -> String {
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
        texts.push(heading_text(&after[..end], open, close));
        rest = &after[end..];
    }
    texts
}

fn blocks_json() -> String {
    format!(
        r#"[{{"id":"{BLOCK_ID}","height":{HEIGHT},"version":1,"timestamp":{TIMESTAMP},"tx_count":17,"size":20,"weight":80,"merkle_root":"mm","mediantime":8}}]"#
    )
}

/// Heading plus the block row the blocks page already paints.
/// An h2 whose text is Blocks is not copied into the heading slot.
fn painted_blocks_page(page: &str, blocks_json: &str) -> String {
    let headings = element_texts(page, "<h1>", "</h1>");
    let mut markup = String::new();
    if headings.iter().filter(|text| text.as_str() == WORD).count() == 1 {
        markup.push_str(H1);
    }
    if page.contains("blocks_rows_markup(") {
        markup.push_str(&blocks_rows_markup(blocks_json));
    }
    markup
}

#[test]
fn blocks_page_paints_blocks() {
    let src = views_source();
    assert!(
        src.contains("path!(\"blocks\") view=BlocksPage"),
        "path blocks does not open BlocksPage"
    );
    let page = code_without_comments(component_body(src, "BlocksPage"));
    assert!(
        !page.contains('?'),
        "BlocksPage read a query string: {page}"
    );
    let json = blocks_json();
    let markup = painted_blocks_page(&page, &json);
    assert!(markup.contains(H1), "Blocks is absent: {markup}");
    assert_eq!(
        markup.matches(H1).count(),
        1,
        "Blocks was not painted once as an h1: {markup}"
    );
    let h2s = element_texts(&page, "<h2>", "</h2>");
    assert!(
        !h2s.iter().any(|text| text.as_str() == WORD),
        "an h2 whose text is Blocks does not satisfy the heading: {h2s:?}"
    );
    assert!(
        !page.contains("<h2>Blocks</h2>") && !page.contains("<h2>\"Blocks\"</h2>"),
        "the markup contains <h2>Blocks</h2>: {page}"
    );
    assert!(
        !markup.contains("<h2>Blocks</h2>"),
        "the h1 assert matched the h2: {markup}"
    );
    let height = format!("<a href=\"/block/{BLOCK_ID}\">{HEIGHT}</a>");
    assert!(
        markup.contains(&height),
        "the block row was dropped: {markup}"
    );
    assert!(
        markup.contains(&format!("<td class=\"time\">{TIMESTAMP}</td>")),
        "the raw timestamp was dropped: {markup}"
    );
    assert!(
        !markup.contains("ago"),
        "the timestamp became a clock: {markup}"
    );
    let rows = blocks_rows_markup(&json);
    assert!(
        !rows.contains(H1),
        "the block row gained the heading: {rows}"
    );

    let calls = blocks_list_requests();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "GET");
    assert_eq!(calls[0].path, "/blocks");
    assert!(!calls[0].path.contains('?'), "{}", calls[0].path);
}
