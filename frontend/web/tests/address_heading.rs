//! Page heading on a loaded no-query `GET /address/:addr`.
//!
//! Angular `address.component.html` paints
//! `<h1 i18n="shared.address">Address</h1>` before the loading gate.
//! `AddressPage` paints that word once as `<h1>"Address"</h1>`.
//! An `<h2>` whose text is Address does not satisfy this test.
//! The route is `address/:addr`. The id is the `addr` path param.
//! The load is `GET /address/:addr` with no query and no fragment.
//! The heading text is the word Address. It is not the address id.

use splora_web::{address_requests, address_rows_markup};

const ADDR: &str = "1stAprEZapjCYGUACUcXohQqSHF9MU5Kj";
const WORD: &str = "Address";
const LOAD_ERROR: &str = "Error loading address data.";

fn loaded_stats() -> String {
    r#"{"chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#
        .to_string()
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
/// `<h1>"Address"</h1>` and `<h1>Address</h1>` both yield Address.
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
fn loaded_no_query_address_page_paints_the_address_heading_once() {
    let requests = address_requests(ADDR);
    assert!(
        requests.iter().any(|call| {
            call.method == "GET"
                && call.path == format!("/address/{ADDR}")
                && !call.path.contains('?')
                && !call.path.contains('#')
        }),
        "no-query GET /address/:addr is missing: {requests:?}"
    );
    assert!(
        requests.iter().all(|call| call.method == "GET"
            && !call.path.contains('?')
            && !call.path.contains('#')),
        "address read added a query: {requests:?}"
    );

    let markup = address_rows_markup(&loaded_stats(), "", "");
    assert!(
        markup.contains("<p class=\"chain-tx-count\">1</p>"),
        "the loaded address page did not paint the address rows: {markup}"
    );
    assert!(
        !markup.contains(LOAD_ERROR),
        "a successful address load painted the loading error: {markup}"
    );

    let src = views_source();
    assert!(
        src.contains("path!(\"address/:addr\") view=AddressPage"),
        "address/:addr does not open AddressPage"
    );
    let page = component_body(src, "AddressPage");
    assert!(
        page.contains("p.get(\"addr\")"),
        "AddressPage does not read the addr path param: {page}"
    );
    assert!(
        !page.contains("use_query_map"),
        "AddressPage read a query string: {page}"
    );
    assert!(
        page.contains("address_rows_markup("),
        "AddressPage does not paint the loaded address rows: {page}"
    );

    let h2s = element_texts(page, "<h2>", "</h2>");
    assert!(
        !h2s.iter().any(|text| text.as_str() == WORD),
        "an h2 whose text is Address does not satisfy the heading: {h2s:?}"
    );
    let headings = element_texts(page, "<h1>", "</h1>");
    assert_eq!(
        headings.iter().filter(|text| text.as_str() == WORD).count(),
        1,
        "expected one h1 whose text is Address, got {headings:?} in {page}"
    );
}
