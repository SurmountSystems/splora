//! A failed no-query block load paints Angular's span above the indexer body.
//!
//! The span is `Error loading block data.` `app-http-error` projects that
//! span into the visible paragraph. The italic line under it is the HTTP
//! body. For a missing block that body stays `Block not found`.

use splora_web::{block_load_plan, block_rows_markup, indexer_failure_text};

const SENTENCE: &str = "Error loading block data.";
const BODY: &str = "Block not found";
const SPAN: &str = "<span>Error loading block data.</span>";
const ITALIC: &str = "<i class=\"small\">(Block not found)</i>";

fn block_page_source() -> &'static str {
    let src = include_str!("../src/views.rs");
    let marker = "fn BlockPage(";
    let start = src
        .find(marker)
        .expect("BlockPage is missing from views.rs");
    let after_name = &src[start..];
    let brace_rel = after_name.find('{').expect("BlockPage has no body");
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
    panic!("BlockPage body is not closed");
}

fn assert_load_error(markup: &str) {
    assert!(
        markup.contains(SENTENCE),
        "Error loading block data. is absent: {markup}"
    );
    assert!(
        markup.contains(BODY),
        "Block not found was replaced or dropped: {markup}"
    );
    let span_at = markup.find(SENTENCE).expect("span");
    let body_at = markup.find(BODY).expect("indexer body");
    assert!(
        span_at < body_at,
        "the span did not sit above Block not found: {markup}"
    );
    assert_eq!(
        markup.matches(SENTENCE).count(),
        1,
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
        !markup.contains("title=\"Error loading block data.\""),
        "the sentence is a tooltip, not element text: {markup}"
    );
    assert!(
        !markup.contains("aria-label=\"Error loading block data.\""),
        "the sentence is an aria label, not element text: {markup}"
    );
}

#[test]
fn block_page_paints_load_error() {
    let id = "nope";
    let failed = indexer_failure_text("GET", &format!("/block/{id}"), BODY);
    assert!(failed.contains(BODY), "{failed}");
    assert!(failed.contains("indexer error GET /block/nope"), "{failed}");
    assert!(!failed.contains('?'), "{failed}");

    let plan = block_load_plan(id, None);
    assert!(
        plan.iter()
            .any(|call| call.method == "GET" && call.path == "/block/nope"),
        "block load did not GET /block/nope: {plan:?}"
    );
    assert!(
        plan.iter()
            .all(|call| !call.path.contains('?') && !call.path.contains('#')),
        "block load used a query string: {plan:?}"
    );

    let markup = block_rows_markup(&failed, "");
    assert_load_error(&markup);

    let height_failed = indexer_failure_text("GET", "/block-height/100", BODY);
    let height_plan = block_load_plan("100", None);
    assert!(
        height_plan.iter().any(|call| {
            call.method == "GET" && call.path == "/block-height/100" && !call.path.contains('?')
        }),
        "height load used a query string: {height_plan:?}"
    );
    assert_load_error(&block_rows_markup(&height_failed, ""));

    let loaded = r#"{"id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","height":1,"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}"#;
    let painted = block_rows_markup(loaded, "[]");
    assert!(
        !painted.contains(SENTENCE),
        "a loaded block painted the error span: {painted}"
    );
    assert!(painted.contains("<tr class=\"block-row\">"), "{painted}");
    assert!(!block_rows_markup("", "").contains(SENTENCE));
    assert!(!block_rows_markup("not-json", "not-json").contains(SENTENCE));
    assert!(!block_rows_markup("not-json", "not-json").contains("<tr"));

    let page = block_page_source();
    assert!(
        page.contains("block_load_error_markup("),
        "BlockPage does not paint the load error: {page}"
    );
    assert!(
        page.contains("block_rows_markup("),
        "BlockPage dropped the block rows: {page}"
    );
}
