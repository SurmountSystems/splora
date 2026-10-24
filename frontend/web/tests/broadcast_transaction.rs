//! Broadcast form heading on the Leptos page.
//!
//! Angular `push-transaction.component.html` paints an h1 whose text is
//! Broadcast Transaction. This page paints that heading once, as markup.
//! A failed POST /tx stays the indexer sentence. It is not an `/tx/` link
//! and not a txid.

use splora_frontend_shared::broadcast_path;
use splora_web::{broadcast_fields_text, broadcast_transaction_heading, submission_links};

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
    assert!(
        page.contains("broadcast_transaction_heading("),
        "BroadcastPage dropped the heading path: {page}"
    );
    let markup = broadcast_transaction_heading();
    assert_eq!(
        markup.matches(HEADING).count(),
        1,
        "BroadcastPage is missing the Broadcast Transaction heading: {markup}"
    );
    // The h1 and the whole line each contain the words.
    assert_eq!(
        markup.matches("Broadcast Transaction").count(),
        2,
        "the heading and the whole line were not both present: {markup}"
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

/// Whole line beside the broadcast h1.
///
/// A line equal to Broadcast Transaction passes. The h1 stays. The failed
/// POST body is the indexer sentence and omits this line.
#[test]
fn broadcast_page_paints_broadcast_transaction_as_its_own_line() {
    const LINE: &str = "Broadcast Transaction";
    let markup = broadcast_transaction_heading();
    assert!(markup.contains(HEADING), "the h1 was dropped: {markup}");
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    let failed = broadcast_fields_text(SENTENCE);
    assert!(
        !failed.lines().any(|line| line == LINE),
        "a failed POST painted {LINE}: {failed}"
    );
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
