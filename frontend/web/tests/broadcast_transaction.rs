//! Broadcast form heading on the Leptos page.
//!
//! Angular `push-transaction.component.html` paints an h1 whose text is
//! Broadcast Transaction. This page paints that heading once, as markup.
//! A failed POST /tx stays the indexer sentence. It is not an `/tx/` link
//! and not a txid.

use splora_frontend_shared::broadcast_path;
use splora_web::{broadcast_fields_text, submission_links};

const HEADING: &str = "<h1>Broadcast Transaction</h1>";
const SENTENCE: &str = "Transaction not found";

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
    panic!("{name} body is not closed")
}

#[test]
fn broadcast_page_paints_the_heading_once() {
    let page = component_body(views_source(), "BroadcastPage");
    assert_eq!(
        page.matches(HEADING).count(),
        1,
        "BroadcastPage is missing the Broadcast Transaction heading: {page}"
    );
    assert_eq!(
        page.matches("Broadcast Transaction").count(),
        1,
        "the heading was repeated: {page}"
    );
    for needle in [
        "Transaction hex",
        "scriptsig",
        "scriptpubkey",
        "witness",
        "Previous output script",
        "Previous Block",
        "skeleton",
        "hashrate",
        "unblinding",
    ] {
        assert!(
            !page.contains(needle),
            "{needle} leaked into the broadcast page: {page}"
        );
    }
}

#[test]
fn failed_post_tx_stays_the_indexer_sentence() {
    let text = broadcast_fields_text(SENTENCE);
    assert!(
        text.contains(SENTENCE),
        "the indexer sentence was dropped: {text}"
    );
    assert!(
        !text.contains("txid"),
        "failed POST /tx was painted as a txid: {text}"
    );
    let links = submission_links(broadcast_path(), SENTENCE);
    assert!(
        !links.contains("/tx/"),
        "failed POST /tx was painted as a link: {links}"
    );
    assert!(
        !links.contains("txid"),
        "failed POST /tx invented a txid: {links}"
    );
}
