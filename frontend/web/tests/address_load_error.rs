//! A failed no-query address load paints Angular's span above the indexer body.
//!
//! The span is `Error loading address data.` The italic line under it is the
//! HTTP body. For an address that does not parse, that body stays
//! `Invalid Bitcoin address`. A parsed address that returns stats does not
//! paint the span. `Error loading block data.` stays the block sentence.

use splora_web::{address_requests, address_rows_markup, indexer_failure_text};

const SENTENCE: &str = "Error loading address data.";
const BLOCK_SENTENCE: &str = "Error loading block data.";
const BODY: &str = "Invalid Bitcoin address";
const SPAN: &str = "<span>Error loading address data.</span>";
const ITALIC: &str = "<i class=\"small\">(Invalid Bitcoin address)</i>";
const ADDR: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
const TYPE_AND_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WPKH</span></span>";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn address_page_source() -> &'static str {
    let src = include_str!("../src/views.rs");
    let marker = "fn AddressPage(";
    let start = src
        .find(marker)
        .expect("AddressPage is missing from views.rs");
    let after_name = &src[start..];
    let brace_rel = after_name.find('{').expect("AddressPage has no body");
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
    panic!("AddressPage body is not closed");
}

fn assert_load_error(markup: &str) {
    assert!(
        markup.contains(SENTENCE),
        "Error loading address data. is absent: {markup}"
    );
    assert!(
        markup.contains(BODY),
        "Invalid Bitcoin address was replaced or dropped: {markup}"
    );
    let span_at = markup.find(SENTENCE).expect("span");
    let body_at = markup.find(BODY).expect("indexer body");
    assert!(
        span_at < body_at,
        "the span did not sit above Invalid Bitcoin address: {markup}"
    );
    assert_eq!(
        markup.matches(SENTENCE).count(),
        2,
        "the span was not painted once: {markup}"
    );
    assert!(
        markup.contains(SPAN),
        "the span is not element text: {markup}"
    );
    assert!(
        markup.contains("<div class=\"http-error\">"),
        "the span is not in the http-error paragraph: {markup}"
    );
    assert!(
        markup.contains("<p><b>"),
        "the span is not in the http-error paragraph: {markup}"
    );
    assert!(
        markup.contains(ITALIC),
        "the italic HTTP body dropped: {markup}"
    );
    assert!(
        !markup.contains("title=\"Error loading address data.\""),
        "the sentence is a tooltip, not element text: {markup}"
    );
    assert!(
        !markup.contains("aria-label=\"Error loading address data.\""),
        "the sentence is an aria label, not element text: {markup}"
    );
    assert!(
        !markup.contains("400 Bad Request"),
        "a 400 Bad Request prefix replaced the indexer body: {markup}"
    );
    assert!(
        !markup.contains(BLOCK_SENTENCE),
        "the address page painted the block sentence: {markup}"
    );
    assert!(
        !markup.contains(">P2WPKH<"),
        "a failed address load painted the badge: {markup}"
    );
}

#[test]
fn address_page_paints_load_error_after_no_query_get() {
    let id = "not-an-address";
    let path = format!("/address/{id}");
    let failed = indexer_failure_text("GET", &path, BODY);
    assert!(failed.contains(BODY), "{failed}");
    assert!(
        failed.contains("indexer error GET /address/not-an-address"),
        "{failed}"
    );
    assert!(!failed.contains(SENTENCE), "{failed}");
    assert!(!failed.contains('?'), "{failed}");
    assert!(!failed.contains("400 Bad Request"), "{failed}");

    let calls = address_requests(id);
    assert!(
        calls
            .iter()
            .any(|call| { call.method == "GET" && call.path == path && !call.path.contains('?') }),
        "no-query GET /address/:addr is missing: {calls:?}"
    );
    assert!(
        calls.iter().all(|call| call.method == "GET"
            && !call.path.contains('?')
            && !call.path.contains('#')),
        "address load used a query string: {calls:?}"
    );

    let markup = address_rows_markup(&failed, "", "");
    assert_load_error(&markup);

    let loaded = address_body(ADDR);
    let painted = address_rows_markup(&loaded, "", "");
    assert!(
        !painted.contains(SENTENCE),
        "a parsed address that returns stats painted the error span: {painted}"
    );
    assert!(
        painted.contains(TYPE_AND_BADGE),
        "a successful load dropped Type or P2WPKH: {painted}"
    );
    assert!(!painted.contains(BLOCK_SENTENCE), "{painted}");
    assert!(!painted.contains("400 Bad Request"), "{painted}");
    assert!(!address_rows_markup("", "", "").contains(SENTENCE));
    assert!(!address_rows_markup("not-json", "", "").contains(SENTENCE));
    assert!(!address_rows_markup("not-json", "not-json", "not-json").contains(SENTENCE));

    let page = address_page_source();
    assert!(
        page.contains("address_load_error_markup("),
        "AddressPage does not paint the load error: {page}"
    );
    assert!(
        page.contains("address_rows_markup("),
        "AddressPage dropped the address rows: {page}"
    );
    assert!(
        page.contains("address_type_badge_markup("),
        "AddressPage dropped the address badge: {page}"
    );
    assert!(
        !page.contains("block_load_error_markup("),
        "AddressPage painted the block load error: {page}"
    );
    assert!(
        !page.contains(BLOCK_SENTENCE),
        "AddressPage used the block sentence: {page}"
    );
}
