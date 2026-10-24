//! Assets page heading on the liquid assets screen.
//!
//! Angular `assets-nav.component.html` paints
//! `<h1 i18n="Assets page header">Assets</h1>`. The route is `assets` with
//! no query string, and it is liquid only. `AssetsPage` paints that word
//! once as an h1 and keeps the registry rows. An h2 whose text is Assets
//! does not satisfy this test. The list load is `GET /assets/registry`
//! with no query string.

use splora_web::{asset_cards_markup, registry_list_markup, registry_list_request};

const WORD: &str = "Assets";
const H1: &str = "<h1>Assets</h1>";
const LATER_ID: &str = "e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1";
const ISSUANCE_TX: &str = "d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2";

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
/// `<h1>"Assets"</h1>` and `<h1>Assets</h1>` both yield Assets.
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

fn registry_json() -> String {
    format!(
        r#"[{{"asset_id":"{LATER_ID}","name":"Later Asset","ticker":"LATE","precision":8,"issuance_txin":{{"txid":"{ISSUANCE_TX}","vin":0}},"issuance_prevout":{{"txid":"{ISSUANCE_TX}","vout":0}},"reissuance_token":"{ISSUANCE_TX}","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":1,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}}}}]"#
    )
}

/// Heading plus the liquid registry strings the assets page already paints.
/// An h2 whose text is Assets is not copied into the heading slot.
fn painted_assets_page(page: &str, registry_json: &str) -> String {
    let headings = element_texts(page, "<h1>", "</h1>");
    let mut markup = String::new();
    if headings.iter().filter(|text| text.as_str() == WORD).count() == 1 {
        markup.push_str(H1);
    }
    if page.contains("asset_cards_markup(") {
        if let Some(cards) = asset_cards_markup(registry_json, true) {
            markup.push_str(&cards);
        }
    }
    if page.contains("registry_list_markup(") {
        markup.push_str(&registry_list_markup(registry_json, true));
    }
    markup
}

#[test]
fn assets_page_paints_assets() {
    let src = views_source();
    assert!(
        src.contains("path!(\"assets\") view=AssetsPage"),
        "path assets does not open AssetsPage"
    );
    let page = component_body(src, "AssetsPage");
    assert!(
        !page.contains('?'),
        "AssetsPage read a query string: {page}"
    );
    let h2s = element_texts(&page, "<h2>", "</h2>");
    assert!(
        !h2s.iter().any(|text| text.as_str() == WORD),
        "an h2 whose text is Assets does not satisfy the heading: {h2s:?}"
    );
    assert!(
        !page.contains("<h2>Assets</h2>") && !page.contains("<h2>\"Assets\"</h2>"),
        "the markup contains <h2>Assets</h2>: {page}"
    );
    let markup = painted_assets_page(&page, &registry_json());
    assert!(markup.contains(H1), "h1 is absent: {markup}");
    assert_eq!(
        markup.matches(H1).count(),
        1,
        "Assets was not painted once as an h1: {markup}"
    );
    assert!(
        !markup.contains("<h2>Assets</h2>"),
        "the h1 assert matched the h2: {markup}"
    );
    assert!(
        markup.contains("Later Asset"),
        "the registry row was dropped: {markup}"
    );
    assert!(
        markup.contains(LATER_ID),
        "the raw registry id was dropped: {markup}"
    );

    let req = registry_list_request(true).expect("liquid registry");
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, "/assets/registry");
    assert!(!req.path.contains('?'), "{}", req.path);
    assert!(registry_list_request(false).is_none());
    assert!(asset_cards_markup(&registry_json(), false).is_none());
    assert!(
        !registry_list_markup(&registry_json(), false).contains("Later Asset"),
        "mainnet painted the registry row"
    );
}

/// Whole line beside the assets registry from `GET /assets/registry`.
///
/// Angular paints `<h1>Assets</h1>`. A line equal to Assets passes. The
/// line sits before the Name header. An empty list and mainnet omit it.
#[test]
fn loaded_registry_paints_assets_as_its_own_line() {
    const LINE: &str = "Assets";
    let markup = registry_list_markup(&registry_json(), true);
    assert!(
        markup.contains("<th>Name</th>"),
        "the registry table was not rendered: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        markup
            .lines()
            .filter(|line| line.contains(LINE))
            .all(|line| line == LINE),
        "{LINE} was painted inside a longer sentence: {markup}"
    );
    let line_at = markup
        .lines()
        .position(|line| line == LINE)
        .expect("Assets line");
    let name_at = markup.find("<th>Name</th>").expect("Name header");
    let before_name = markup[..name_at].lines().count();
    assert!(
        line_at < before_name,
        "Assets was not before the Name header: {markup}"
    );
    let empty = registry_list_markup("[]", true);
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty list painted {LINE}: {empty}"
    );
    let mainnet = registry_list_markup(&registry_json(), false);
    assert!(
        !mainnet.lines().any(|line| line == LINE),
        "mainnet painted {LINE}: {mainnet}"
    );
}
