//! Maximum burn amount label on the Leptos broadcast page.
//!
//! Angular `push-transaction.component.html` paints the label text
//! Maximum burn amount (sats). This page paints that label once, as markup.
//! The words are visible. They do not send a query or a fragment.

const LABEL: &str = "Maximum burn amount (sats)";

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
fn broadcast_page_paints_maximum_burn_amount_once() {
    let page = component_body(views_source(), "BroadcastPage");
    assert_eq!(
        page.matches(LABEL).count(),
        1,
        "BroadcastPage is missing Maximum burn amount (sats): {page}"
    );
    for needle in [
        "Submit Package",
        "Maximum fee rate",
        "Transaction hex",
        "Coldcard",
        "Fiat price last updated",
    ] {
        assert!(
            !page.contains(needle),
            "{needle} leaked into the broadcast page: {page}"
        );
    }
}
