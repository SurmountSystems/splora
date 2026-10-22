//! Block page heading from the loaded `GET /block/:hash` body.
//!
//! Angular `block.component.html` puts one `h1` around the title word and
//! the height digits. The digits sit in an anchor with class `block-link`.
//! Height above 0 uses the title word Block. Height 0 uses Genesis. The
//! class name is lowercase and is not that title word. Raw JSON `stale:
//! true` uses the title words Stale Block, including height 0. A missing
//! `stale` key stays Block or Genesis. The indexer struct has no stale field.

use splora_web::parse_block_json;

const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HEIGHT: u32 = 123;

fn block_json(height: u32) -> String {
    format!(
        r#"{{"id":"{ID}","height":{height},"version":1,"timestamp":1,"tx_count":1,"size":1,"weight":4,"merkle_root":"merkle","mediantime":1}}"#
    )
}

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

/// Body inside the function braces, not the next function.
fn function_body<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("fn {name}(");
    let start = src.find(&marker)?;
    let after_name = &src[start..];
    let brace_rel = after_name.find('{')?;
    let body_start = start + brace_rel + 1;
    let mut depth = 1i32;
    for (offset, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&src[body_start..body_start + offset]);
                }
            }
            _ => {}
        }
    }
    None
}

fn component_body<'a>(src: &'a str, name: &str) -> &'a str {
    function_body(src, name).unwrap_or_else(|| panic!("{name} is missing from views.rs"))
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

/// The format string that contains `block-link`, with Rust escapes applied.
fn anchor_template(body: &str) -> Option<String> {
    let pos = body.find("block-link")?;
    let bytes = body.as_bytes();
    let mut start = None;
    let mut index = pos;
    while index > 0 {
        index -= 1;
        if bytes[index] == b'"' && (index == 0 || bytes[index - 1] != b'\\') {
            start = Some(index + 1);
            break;
        }
    }
    let start = start?;
    let mut end = None;
    let mut cursor = pos;
    while cursor < bytes.len() {
        if bytes[cursor] == b'"' && (cursor == 0 || bytes[cursor - 1] != b'\\') {
            end = Some(cursor);
            break;
        }
        cursor += 1;
    }
    let end = end?;
    Some(body[start..end].replace("\\\"", "\"").replace("\\\\", "\\"))
}

/// Heading markup the block page paints for this loaded body.
///
/// Until the page builds that heading, the markup is the literal `h1` in
/// `BlockPage`. An `h1` whose only text is Block fails the digit assert.
fn painted(block_json: &str) -> String {
    let src = views_source();
    let page = component_body(src, "BlockPage");
    if let Some(markup) = wired_heading(src, page, block_json) {
        return markup;
    }
    first_h1(page)
}

/// Line and block comments drop out. String literals stay, so a comment
/// that merely names Stale Block does not count as the heading read.
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

/// GPUI reads `stale` from the raw block JSON, not from `wire::Block`.
/// The title words have to appear before the height-0 arm, or height 0
/// would stay Genesis.
fn heading_reads_raw_stale(body: &str) -> bool {
    let code = compact(&code_without_comments(body));
    let reads = code.contains("serde_json::from_str::<serde_json::Value>")
        && code.contains(".get(\"stale\")")
        && code.contains("as_bool()")
        && code.contains("==Some(true)");
    let stale_title = code.find("\"StaleBlock\"");
    let genesis = code.find("Some(0)=>\"Genesis\"");
    match (reads, stale_title, genesis) {
        (true, Some(stale_at), Some(genesis_at)) => stale_at < genesis_at,
        _ => false,
    }
}

fn raw_json_stale_is_true(block_json: &str) -> bool {
    compact(block_json).contains("\"stale\":true")
}

fn stale_block_json(height: u32) -> String {
    let mut body = block_json(height);
    assert!(body.ends_with('}'));
    body.pop();
    body.push_str(r#","stale":true}"#);
    body
}

fn wired_heading(src: &str, page: &str, block_json: &str) -> Option<String> {
    if !page.contains("block_page_heading_markup(") || !page.contains("block_body") {
        return None;
    }
    let body = function_body(src, "block_page_heading_markup")?;
    let compact = compact(body);
    if !compact.contains("Some(0)=>\"Genesis\"") || !compact.contains("_=>\"Block\"") {
        return None;
    }
    let template = anchor_template(body)?;
    if !template.contains("class=\"block-link\"") || template.contains(',') {
        return None;
    }
    let block = parse_block_json(block_json).ok()?;
    let title = if heading_reads_raw_stale(body) && raw_json_stale_is_true(block_json) {
        "Stale Block"
    } else if block.height == 0 {
        "Genesis"
    } else {
        "Block"
    };
    Some(
        template
            .replace("{title}", title)
            .replace("{id}", &block.id)
            .replace("{height}", &block.height.to_string()),
    )
}

fn first_h1(page: &str) -> String {
    let Some(start) = page.find("<h1>").or_else(|| page.find("<h1 ")) else {
        return String::new();
    };
    let rest = &page[start..];
    let Some(end) = rest.find("</h1>").map(|index| index + "</h1>".len()) else {
        return String::new();
    };
    rest[..end].to_string()
}

fn expected_h1(title: &str, height: u32) -> String {
    format!("<h1>{title} <a class=\"block-link\" href=\"/block/{ID}\">{height}</a></h1>")
}

fn title_word(h1: &str) -> String {
    let inner = h1
        .trim()
        .strip_prefix("<h1>")
        .and_then(|rest| rest.strip_suffix("</h1>"))
        .unwrap_or(h1);
    let before = inner.split("<a").next().unwrap_or(inner).trim();
    before
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(before)
        .trim()
        .to_string()
}

fn anchor_text(h1: &str) -> String {
    let Some(start) = h1.find("<a ") else {
        return String::new();
    };
    let rest = &h1[start..];
    let Some(open_end) = rest.find('>') else {
        return String::new();
    };
    let text = &rest[open_end + 1..];
    let Some(close) = text.find("</a>") else {
        return String::new();
    };
    text[..close].to_string()
}

fn h1_count(markup: &str) -> usize {
    markup.matches("<h1>").count()
}

#[test]
fn height_123_paints_one_h1_with_block_and_those_digits() {
    let markup = painted(&block_json(HEIGHT));
    assert_eq!(
        markup,
        expected_h1("Block", HEIGHT),
        "height digits absent from the h1: {markup}"
    );
    assert_eq!(h1_count(&markup), 1, "{markup}");
    assert_eq!(title_word(&markup), "Block", "{markup}");
    assert_eq!(
        anchor_text(&markup),
        "123",
        "height digits absent from the h1: {markup}"
    );
    assert!(
        markup.contains("class=\"block-link\""),
        "block-link anchor missing: {markup}"
    );
    let visible = format!("{} {}", title_word(&markup), anchor_text(&markup));
    assert_ne!(
        visible.trim(),
        "Block",
        "an h1 whose only text is Block: {markup}"
    );
    assert!(!markup.contains(','), "thousands separator in {markup}");
    let page = component_body(views_source(), "BlockPage");
    assert!(
        !page.contains("<h1>\"Block\"</h1>"),
        "an h1 whose only text is Block"
    );
}

#[test]
fn height_0_paints_one_h1_with_genesis_and_the_digit_0() {
    let markup = painted(&block_json(0));
    assert_eq!(
        anchor_text(&markup),
        "0",
        "height digits absent from the h1: {markup}"
    );
    assert_eq!(markup, expected_h1("Genesis", 0), "{markup}");
    assert_eq!(h1_count(&markup), 1, "{markup}");
    assert_eq!(title_word(&markup), "Genesis", "{markup}");
    assert_ne!(title_word(&markup), "Block", "{markup}");
    assert_ne!(title_word(&markup).to_ascii_lowercase(), "block-link");
    assert!(
        markup.contains("class=\"block-link\""),
        "block-link anchor missing: {markup}"
    );
    assert!(!markup.contains(','), "thousands separator in {markup}");
}

#[test]
fn stale_true_paints_one_h1_with_stale_block_and_the_height_digits() {
    let markup = painted(&stale_block_json(HEIGHT));
    assert_eq!(
        markup,
        expected_h1("Stale Block", HEIGHT),
        "Stale Block is absent: {markup}"
    );
    assert_eq!(h1_count(&markup), 1, "{markup}");
    assert_eq!(title_word(&markup), "Stale Block", "{markup}");
    assert_ne!(title_word(&markup), "Block", "{markup}");
    assert_ne!(title_word(&markup), "Genesis", "{markup}");
    assert_eq!(
        anchor_text(&markup),
        "123",
        "height digits absent from the h1: {markup}"
    );
    assert!(
        markup.contains("class=\"block-link\""),
        "block-link anchor missing: {markup}"
    );
    assert!(!markup.contains(','), "thousands separator in {markup}");

    let genesis = painted(&stale_block_json(0));
    assert_eq!(
        genesis,
        expected_h1("Stale Block", 0),
        "Stale Block is absent: {genesis}"
    );
    assert_eq!(title_word(&genesis), "Stale Block", "{genesis}");
    assert_ne!(title_word(&genesis), "Genesis", "{genesis}");
    assert_ne!(title_word(&genesis), "Block", "{genesis}");
    assert_eq!(
        anchor_text(&genesis),
        "0",
        "height digits absent from the h1: {genesis}"
    );
    assert!(
        genesis.contains("class=\"block-link\""),
        "block-link anchor missing: {genesis}"
    );
    assert!(!genesis.contains(','), "thousands separator in {genesis}");
}
