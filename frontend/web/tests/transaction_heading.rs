//! Page heading on a no-query `GET /tx/:txid`.
//!
//! Angular `transaction.component.html` paints
//! `<h1 i18n="shared.transaction">Transaction</h1>`.
//! `TransactionPage` paints that word once as `<h1>"Transaction"</h1>`.
//! An `<h2>` whose text is Transaction does not satisfy this test.
//! The heading text is the word Transaction. It is not Transaction hex,
//! Transaction fee, Transaction index, Broadcast Transaction, or Recent
//! Transactions. The load is `GET /tx/:txid` with no query and no fragment.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const WORD: &str = "Transaction";

fn tx() -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa","value":5}},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"76a914cc88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","scriptpubkey_address":"1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2","value":4}}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

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

/// Text of one heading element. Leptos wraps the word in quotes.
/// `<h1>"Transaction"</h1>` and `<h1>Transaction</h1>` both yield Transaction.
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

#[test]
fn no_query_tx_page_paints_the_transaction_heading_once() {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));

    let page = component_body(views_source(), "TransactionPage");
    let h2s = element_texts(page, "<h2>", "</h2>");
    assert!(
        !h2s.iter().any(|text| text.as_str() == WORD),
        "an h2 whose text is Transaction does not satisfy the heading: {h2s:?}"
    );
    let headings = element_texts(page, "<h1>", "</h1>");
    assert_eq!(
        headings.iter().filter(|text| text.as_str() == WORD).count(),
        1,
        "expected one h1 whose text is Transaction, got {headings:?} in {page}"
    );
    assert!(
        headings.iter().all(|text| text.as_str() == WORD),
        "a heading was a longer sentence than Transaction: {headings:?}"
    );

    let markup = transaction_row_markup(&tx());
    assert!(
        markup.contains("<h2>Inputs & Outputs</h2>"),
        "the transaction page was not rendered: {markup}"
    );
    assert!(
        !markup.contains("Transaction hex"),
        "Transaction hex was painted: {markup}"
    );
}
