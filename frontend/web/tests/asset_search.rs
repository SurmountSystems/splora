//! Cypress `assets / allows searching assets`.
//!
//! `liquid.spec.ts` and `liquidtestnet.spec.ts` visit `/assets`, type
//! `Tether USD` into `.container-xl input`, and expect one
//! `ngb-typeahead-window`. The indexer arm in `src/rest.rs` reads `q`.
//! An empty `q` returns `[]`. Leptos sends `?q=` through
//! `assets_registry_search_path`. An empty body paints no window.
//! No price and no amount.

use splora_web::{asset_search_markup, assets_registry_search_path, registry_search_request};

/// `q` from a path `assets_registry_search_path` built. Empty when `q` is absent.
fn query_value(path: &str) -> &str {
    path.split_once("?q=")
        .filter(|(head, _)| *head == "/assets/registry/search" && !path.contains('#'))
        .map(|(_, query)| query)
        .unwrap_or("")
}

fn tether_hit() -> String {
    let tether = "12".repeat(32);
    format!(
        r#"[{{"asset_id":"{tether}","name":"Tether USD","ticker":"USDt","price":"1","amount":"9"}}]"#
    )
}

fn search_hits() -> String {
    let aud = "34".repeat(32);
    let tether = tether_hit();
    let tether = tether.trim_end_matches(']');
    format!(
        r#"{tether},{{"asset_id":"{aud}","name":"Liquid AUD","ticker":"L-AUD","price":"2","amount":"8"}}]"#
    )
}

fn windows(markup: &str) -> usize {
    markup.matches("<ngb-typeahead-window").count()
}

/// Typing `Tether USD` builds `GET /assets/registry/search?q=` through
/// `assets_registry_search_path` with a non-empty query. One hit paints one
/// suggestion window. Two hits stay in that same one window.
#[test]
fn typing_tether_usd_requests_the_query_and_opens_one_suggestion_window() {
    let query = "Tether USD";
    assert!(!query.is_empty());
    let built = assets_registry_search_path(query);
    let q = query_value(&built);
    assert!(
        !q.is_empty(),
        "empty q would make the indexer return []: {built}"
    );
    assert_ne!(built, assets_registry_search_path(""));
    assert_ne!(q, query_value(&assets_registry_search_path("")));
    assert_eq!(q, "Tether%20USD", "{built}");

    let req = registry_search_request(true, query).expect("liquid search");
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, built, "request did not send the non-empty q");
    assert_ne!(req.path, "/assets/registry/search");

    let one = asset_search_markup(query, &tether_hit(), true).expect("one suggestion window");
    assert_eq!(windows(&one), 1, "{one}");
    assert_eq!(one.matches("suggestion window").count(), 1, "{one}");
    assert!(one.lines().any(|line| line == "Liquid Asset"), "{one}");
    assert!(one.contains("Tether USD"), "{one}");
    assert!(one.contains("USDt"), "{one}");
    assert!(!one.contains("Liquid AUD"), "{one}");
    assert!(!one.contains("price"), "{one}");
    assert!(!one.contains("amount"), "{one}");

    let markup = asset_search_markup(query, &search_hits(), true).expect("one suggestion window");
    assert_eq!(windows(&markup), 1, "{markup}");
    assert_eq!(markup.matches("suggestion window").count(), 1, "{markup}");
    assert!(markup.contains("Tether USD"), "{markup}");
    assert!(markup.contains("Liquid AUD"), "{markup}");
    assert!(!markup.contains("price"), "{markup}");
    assert!(!markup.contains("amount"), "{markup}");
}

/// An empty search body paints no suggestion window. The request still
/// carries `?q=`.
#[test]
fn empty_registry_search_paints_no_suggestion_window() {
    let built = assets_registry_search_path("Tether USD");
    assert!(!query_value(&built).is_empty(), "{built}");
    let req = registry_search_request(true, "Tether USD").expect("query is still sent");
    assert_eq!(req.path, built);
    assert!(!req.path.contains('#'), "{}", req.path);

    assert!(asset_search_markup("Tether USD", "[]", true).is_none());
    assert!(asset_search_markup("Tether USD", "[ ]", true).is_none());
    assert!(asset_search_markup("Tether USD", "", true).is_none());
    assert!(asset_search_markup("", &search_hits(), true).is_none());
    assert!(registry_search_request(true, " ").is_none());
}

/// Mainnet does not request the liquid search and does not paint a window.
#[test]
fn mainnet_does_not_open_a_suggestion_window() {
    assert!(registry_search_request(false, "Tether USD").is_none());
    assert!(asset_search_markup("Tether USD", &search_hits(), false).is_none());
}
