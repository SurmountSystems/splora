//! Sign In is a mempool.space account link, not a field the indexer serves.
//!
//! The Leptos switcher markup does not contain Sign In. A client account
//! link is not explorer data. The assertion fails if that markup still
//! contains the text Sign In.

const LABEL: &str = "Sign In";

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
fn switcher_markup_does_not_contain_sign_in() {
    let menu = component_body(views_source(), "Switcher");
    assert!(
        !menu.contains(LABEL),
        "switcher markup still contains Sign In: {menu}"
    );
}
