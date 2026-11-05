//! The rendered block page calls `block_transactions_markup_for_viewport`
//! for its pagination list.
//!
//! The helper chooses that list. This test does not call the helper and
//! does not assert a child count.

fn block_page_body() -> &'static str {
    let src = include_str!("../src/views.rs");
    let marker = "fn BlockPage(";
    let start = src
        .find(marker)
        .unwrap_or_else(|| panic!("BlockPage is missing from views.rs"));
    let after_name = &src[start..];
    let brace_rel = after_name
        .find('{')
        .unwrap_or_else(|| panic!("BlockPage has no body"));
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

#[test]
fn block_page_calls_block_transactions_markup_for_viewport() {
    let body = block_page_body();
    assert!(
        body.contains("block_transactions_markup_for_viewport("),
        "BlockPage does not call block_transactions_markup_for_viewport for the pagination list"
    );
    assert!(
        !body.contains("let _ = crate::block_transactions_markup_for_viewport(")
            && !body.contains("let _ = block_transactions_markup_for_viewport("),
        "BlockPage discards block_transactions_markup_for_viewport"
    );
}
