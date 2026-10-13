//! Cypress `assets / shows a specific asset ID`.
//!
//! `liquid.spec.ts` visits `/assets`, types `Liquid AUD` into
//! `.container-xl input`, and clicks the first button in
//! `ngb-typeahead-window`. `assets-nav.component.ts` navigates to
//! `/assets/asset/{id}`. The indexer read is `GET /asset/:id` in
//! `src/rest.rs`. Leptos builds the search with
//! `assets_registry_search_path`. Opening that first hit selects
//! `asset_page_markup` and the page shows that asset id.
//! No price and no amount.

use splora_web::{
    asset_page_markup, asset_requests, asset_search_markup, assets_registry_search_path,
    registry_search_request,
};

/// `q` from a path `assets_registry_search_path` built. Empty when `q` is absent.
fn query_value(path: &str) -> &str {
    path.split_once("?q=")
        .filter(|(head, _)| *head == "/assets/registry/search" && !path.contains('#'))
        .map(|(_, query)| query)
        .unwrap_or("")
}

fn windows(markup: &str) -> usize {
    markup.matches("<ngb-typeahead-window").count()
}

/// `href` on each suggestion button, in paint order. The first is the
/// Cypress `ngb-typeahead-window:nth-of-type(1) button` click.
fn button_open_paths(markup: &str) -> Vec<&str> {
    markup
        .split("<button")
        .skip(1)
        .filter_map(|button| {
            let end = button.find('>').unwrap_or(button.len());
            let tag = &button[..end];
            let key = "href=\"";
            let start = tag.find(key)? + key.len();
            let rest = &tag[start..];
            let stop = rest.find('"')?;
            Some(&rest[..stop])
        })
        .collect()
}

fn search_hits(aud: &str, tether: &str) -> String {
    format!(
        r#"[{{"asset_id":"{aud}","name":"Liquid AUD","ticker":"L-AUD","price":"2","amount":"8"}},{{"asset_id":"{tether}","name":"Tether USD","ticker":"USDt","price":"1","amount":"9"}}]"#
    )
}

fn asset_json(id: &str, name: &str, ticker: &str) -> String {
    format!(r#"{{"asset_id":"{id}","name":"{name}","ticker":"{ticker}","price":"2","amount":"8"}}"#)
}

fn assert_no_price_or_amount(text: &str) {
    assert!(!text.contains("price"), "{text}");
    assert!(!text.contains("amount"), "{text}");
}

/// Typing `Liquid AUD` builds `GET /assets/registry/search?q=` through
/// `assets_registry_search_path`. The first suggestion opens
/// `/assets/asset/{id}`, which selects the asset page and shows that id.
/// The second hit stays closed. `GET /asset/:id` is the indexer read.
#[test]
fn shows_a_specific_asset_id() {
    let aud = "34".repeat(32);
    let tether = "12".repeat(32);
    assert_eq!(aud.len(), 64);
    assert_eq!(tether.len(), 64);
    assert_ne!(aud, tether);

    let query = "Liquid AUD";
    let built = assets_registry_search_path(query);
    let q = query_value(&built);
    assert!(
        !q.is_empty(),
        "empty q would make the indexer return []: {built}"
    );
    assert_eq!(q, "Liquid%20AUD", "{built}");
    assert_ne!(built, assets_registry_search_path(""));

    let req = registry_search_request(true, query).expect("liquid search");
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, built, "request did not send the non-empty q");
    assert!(req.path.contains('?'), "{}", req.path);
    assert!(!req.path.contains('#'), "{}", req.path);

    let body = search_hits(&aud, &tether);
    let window = asset_search_markup(query, &body, true).expect("one suggestion window");
    assert_eq!(windows(&window), 1, "{window}");
    assert_eq!(window.matches("suggestion window").count(), 1, "{window}");
    assert!(window.contains("Liquid AUD"), "{window}");
    assert!(window.contains("L-AUD"), "{window}");
    assert!(window.contains("Tether USD"), "{window}");
    let aud_at = window.find("Liquid AUD").expect("Liquid AUD");
    let tether_at = window.find("Tether USD").expect("Tether USD");
    assert!(
        aud_at < tether_at,
        "Liquid AUD is not the first suggestion: {window}"
    );
    assert_no_price_or_amount(&window);

    let aud_page = format!("/assets/asset/{aud}");
    let tether_page = format!("/assets/asset/{tether}");
    let paths = button_open_paths(&window);
    assert_eq!(
        paths.first().copied(),
        Some(aud_page.as_str()),
        "the first suggestion button does not open Liquid AUD: {paths:?} {window}"
    );
    assert_eq!(
        paths.get(1).copied(),
        Some(tether_page.as_str()),
        "the second suggestion is not Tether USD: {paths:?}"
    );

    let href = paths
        .first()
        .copied()
        .expect("the first suggestion button does not open an asset page");
    assert_eq!(href, aud_page);
    assert!(!href.contains(&tether), "{href}");

    let reads = asset_requests(true, &aud);
    let asset_get = format!("/asset/{aud}");
    assert!(
        reads
            .iter()
            .any(|req| req.method == "GET" && req.path == asset_get),
        "opening the suggestion did not use GET /asset/:id: {reads:?}"
    );
    assert!(
        reads
            .iter()
            .all(|req| req.path != format!("/asset/{tether}")),
        "the second suggestion was requested: {reads:?}"
    );

    let opened = asset_page_markup(href, &asset_json(&aud, "Liquid AUD", "L-AUD"), "[]")
        .expect("opening the first hit selects the asset page");
    assert!(
        opened.contains(&aud),
        "the asset page does not show the Liquid AUD id: {opened}"
    );
    assert!(opened.contains("Liquid AUD"), "{opened}");
    assert!(opened.contains("L-AUD"), "{opened}");
    assert!(!opened.contains(&tether), "{opened}");
    assert!(
        opened.contains("<h2>Issuance and Burn Transactions</h2>"),
        "{opened}"
    );
    assert_eq!(opened.matches("<h2>").count(), 1, "{opened}");
    assert_no_price_or_amount(&opened);
    assert!(
        asset_page_markup(href, &asset_json(&tether, "Tether USD", "USDt"), "[]").is_none(),
        "the first hit selected a different asset"
    );

    assert!(asset_search_markup(query, "[]", true).is_none());
    assert!(asset_search_markup(query, &body, false).is_none());
    assert!(registry_search_request(false, query).is_none());
    assert!(asset_requests(false, &aud).is_empty());
}
