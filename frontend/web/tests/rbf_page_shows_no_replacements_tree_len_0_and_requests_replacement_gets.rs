//! RBF page opening state, before a websocket update.
//!
//! Cypress `mainnet.spec.ts` it "RBF page gets updated over websockets"
//! starts with no replacements on the page, a replacement tree length of
//! 0, and requests for `GET /api/v1/replacements` and
//! `GET /api/v1/fullrbf/replacements`. `RbfPage` paints that opening
//! state. A comment that names the paths does not count.

const GET_REPLACEMENTS: &str = "GET /api/v1/replacements";
const GET_FULLRBF: &str = "GET /api/v1/fullrbf/replacements";
const PATH_REPLACEMENTS: &str = "/api/v1/replacements";
const PATH_FULLRBF: &str = "/api/v1/fullrbf/replacements";

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
/// that merely names the opening state does not count.
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

/// Uncommented `RbfPage` body. An absent page is empty, which is not
/// the opening state.
fn rbf_opening_page(src: &str) -> String {
    const NAME: &str = "RbfPage";
    let marker = format!("fn {NAME}(");
    if !src.contains(&marker) {
        return String::new();
    }
    code_without_comments(component_body(src, NAME))
}

fn shows_no_replacements(page: &str) -> bool {
    page.to_ascii_lowercase().contains("no replacements")
}

fn line_has_isolated_zero(line: &str) -> bool {
    let bytes = line.as_bytes();
    for (index, byte) in bytes.iter().copied().enumerate() {
        if byte != b'0' {
            continue;
        }
        let prev_digit = index > 0 && bytes[index - 1].is_ascii_digit();
        let next_digit = index + 1 < bytes.len() && bytes[index + 1].is_ascii_digit();
        if !prev_digit && !next_digit {
            return true;
        }
    }
    false
}

/// `Some(0)` only when a line names the tree and an isolated 0.
/// A bare 0 on some other line does not count. An absent page is `None`.
fn replacement_tree_len(page: &str) -> Option<u32> {
    for line in page.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("tree") && line_has_isolated_zero(&lower) {
            return Some(0);
        }
    }
    None
}

fn page_requests(page: &str, path: &str, get: &str) -> bool {
    page.contains(get) || page.contains(&format!("\"{path}\""))
}

fn replacement_gets(page: &str) -> Vec<&'static str> {
    let mut requests = Vec::new();
    if page_requests(page, PATH_REPLACEMENTS, GET_REPLACEMENTS) {
        requests.push(GET_REPLACEMENTS);
    }
    if page_requests(page, PATH_FULLRBF, GET_FULLRBF) {
        requests.push(GET_FULLRBF);
    }
    requests
}

#[test]
fn rbf_page_shows_no_replacements_tree_len_0_and_requests_replacement_gets() {
    let page = rbf_opening_page(views_source());
    let showed = shows_no_replacements(&page);
    let tree_len = replacement_tree_len(&page);
    let requests = replacement_gets(&page);
    assert_eq!(
        (showed, tree_len, requests.as_slice()),
        (
            true,
            Some(0),
            [
                "GET /api/v1/replacements",
                "GET /api/v1/fullrbf/replacements",
            ]
            .as_slice()
        ),
        "RBF page opening state does not show no replacements, a tree length of 0, and the two replacement GETs: {page}"
    );
}
