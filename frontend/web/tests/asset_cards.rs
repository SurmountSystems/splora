//! Cypress `assets / shows the assets screen`.
//!
//! `liquid.spec.ts` visits `/assets` and expects `.featuredBox .card` to
//! have a length of at least five. That live registry is not this fixture.
//! This test does not mark that Cypress row ported. The list route is
//! `assets_registry_path` (`GET /assets/registry`). The screen does not
//! request `/assets/featured`. One card is one named issued row. A pegged
//! row is skipped. An empty name is skipped. Four named rows stay four.
//! The card is the name and the ticker. It is not a price and not an amount.
//! Mainnet does not request the registry.

use splora_web::{asset_cards_markup, assets_registry_path, registry_list_request};

const ISSUED_AMOUNT: &str = "424242";
const PRICE: &str = "9.99";
const AMOUNT: &str = "8.88";

fn issued(id: &str, name: &str, ticker: Option<&str>) -> String {
    let name_field = format!(r#","name":"{name}""#);
    let ticker_field = match ticker {
        Some(ticker) => format!(r#","ticker":"{ticker}""#),
        None => String::new(),
    };
    format!(
        r#"{{"asset_id":"{id}","issuance_txin":{{"txid":"issuance-tx","vin":0}},"issuance_prevout":{{"txid":"issuance-prev","vout":1}},"reissuance_token":"reissue-token","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":{ISSUED_AMOUNT},"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"price":"{PRICE}","amount":"{AMOUNT}"{name_field}{ticker_field}}}"#
    )
}

fn empty_name(id: &str, ticker: &str) -> String {
    format!(
        r#"{{"asset_id":"{id}","issuance_txin":{{"txid":"issuance-tx","vin":0}},"issuance_prevout":{{"txid":"issuance-prev","vout":1}},"reissuance_token":"reissue-token","status":{{"confirmed":true}},"chain_stats":{{"tx_count":1,"issuance_count":1,"issued_amount":{ISSUED_AMOUNT},"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"mempool_stats":{{"tx_count":0,"issuance_count":0,"issued_amount":0,"burned_amount":0,"has_blinded_issuances":false,"burned_reissuance_tokens":0}},"name":"","ticker":"{ticker}","price":"{PRICE}","amount":"{AMOUNT}"}}"#
    )
}

fn pegged(id: &str) -> String {
    format!(
        r#"{{"asset_id":"{id}","chain_stats":{{"tx_count":1,"peg_in_count":1,"peg_in_amount":{ISSUED_AMOUNT},"peg_out_count":0,"peg_out_amount":0,"burn_count":0,"burned_amount":0}},"mempool_stats":{{"tx_count":0,"peg_in_count":0,"peg_in_amount":0,"peg_out_count":0,"peg_out_amount":0,"burn_count":0,"burned_amount":0}},"price":"{PRICE}","amount":"{AMOUNT}"}}"#
    )
}

fn featured_names() -> [(&'static str, &'static str); 5] {
    [
        ("Tether USD", "USDt"),
        ("Liquid AUD", "L-AUD"),
        ("Liquid CAD", "L-CAD"),
        ("Liquid EUR", "EUR"),
        ("Liquid JPY", "JPY"),
    ]
}

fn asset_id(index: usize) -> String {
    format!("{index:02x}").repeat(32)
}

fn registry_of(rows: &[String]) -> String {
    format!("[{}]", rows.join(","))
}

fn five_registry() -> String {
    let rows: Vec<String> = featured_names()
        .iter()
        .enumerate()
        .map(|(index, (name, ticker))| issued(&asset_id(index), name, Some(ticker)))
        .collect();
    registry_of(&rows)
}

fn card_fragments(markup: &str) -> Vec<&str> {
    assert_eq!(
        markup.matches("class=\"featuredBox\"").count(),
        1,
        "{markup}"
    );
    let mut texts = Vec::new();
    let mut rest = markup;
    let open = "<div class=\"card\">";
    while let Some(at) = rest.find(open) {
        rest = &rest[at + open.len()..];
        let end = rest.find("</div>").unwrap_or(rest.len());
        texts.push(&rest[..end]);
        rest = &rest[end..];
    }
    assert_eq!(
        texts.len(),
        markup.matches("class=\"card\"").count(),
        "{markup}"
    );
    texts
}

fn visible(fragment: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in fragment.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn assert_no_price_or_amount(markup: &str) {
    assert!(!markup.contains("price"), "{markup}");
    assert!(!markup.contains("amount"), "{markup}");
    assert!(!markup.contains(PRICE), "{markup}");
    assert!(!markup.contains(AMOUNT), "{markup}");
    assert!(!markup.contains(ISSUED_AMOUNT), "{markup}");
    assert!(!markup.contains("/assets/featured"), "{markup}");
    assert!(!markup.contains("spinner-border"), "{markup}");
    assert!(!markup.contains("loadingGraphs"), "{markup}");
    assert!(!markup.contains("skeleton"), "{markup}");
}

fn assert_registry_request() {
    assert_eq!(assets_registry_path(), "/assets/registry");
    assert_ne!(assets_registry_path(), "/assets/featured");
    let req = registry_list_request(true).expect("liquid registry");
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, assets_registry_path());
    assert!(!req.path.contains('?'), "{}", req.path);
    assert!(!req.path.contains('#'), "{}", req.path);
    assert!(registry_list_request(false).is_none());
}

/// Five named issued rows from `GET /assets/registry` paint five cards.
#[test]
fn assets_screen_shows_five_featured_cards() {
    assert_registry_request();
    let body = five_registry();
    let markup = asset_cards_markup(&body, true).expect("five featured cards");
    let cards = card_fragments(&markup);
    assert_eq!(cards.len(), 5, "{markup}");
    for (index, (name, ticker)) in featured_names().iter().enumerate() {
        let card = cards[index];
        assert_eq!(visible(card), format!("{name} {ticker}"), "{card}");
        assert!(
            card.contains(&format!("href=\"/assets/asset/{}\"", asset_id(index))),
            "{card}"
        );
    }
    assert_no_price_or_amount(&markup);
    assert_eq!(markup.matches("class=\"card\"").count(), 5, "{markup}");
}

/// Four named rows stay four cards. The screen does not pad the list to five.
#[test]
fn four_named_assets_stay_four_cards() {
    let rows: Vec<String> = featured_names()
        .iter()
        .take(4)
        .enumerate()
        .map(|(index, (name, ticker))| issued(&asset_id(index), name, Some(ticker)))
        .collect();
    let markup = asset_cards_markup(&registry_of(&rows), true).expect("four cards");
    let cards = card_fragments(&markup);
    assert_eq!(cards.len(), 4, "{markup}");
    assert!(!markup.contains("Liquid JPY"), "{markup}");
    assert_no_price_or_amount(&markup);
}

/// A name with no ticker is still one card. The ticker is not invented.
#[test]
fn named_asset_without_ticker_is_one_card() {
    let id = asset_id(7);
    let row = issued(&id, "Bare Name", None);
    let markup = asset_cards_markup(&registry_of(&[row]), true).expect("one card");
    let cards = card_fragments(&markup);
    assert_eq!(cards.len(), 1, "{markup}");
    assert_eq!(visible(&cards[0]), "Bare Name", "{}", cards[0]);
    assert!(
        cards[0].contains(&format!("href=\"/assets/asset/{id}\"")),
        "{}",
        cards[0]
    );
    assert_no_price_or_amount(&markup);
}

/// A pegged row and an empty name are not cards. Five named rows stay five.
#[test]
fn pegged_and_empty_name_are_not_cards() {
    let mut rows: Vec<String> = featured_names()
        .iter()
        .enumerate()
        .map(|(index, (name, ticker))| issued(&asset_id(index), name, Some(ticker)))
        .collect();
    let peg_id = "cd".repeat(32);
    rows.push(pegged(&peg_id));
    rows.push(empty_name(&asset_id(8), "GHOST"));
    let markup = asset_cards_markup(&registry_of(&rows), true).expect("named cards only");
    let cards = card_fragments(&markup);
    assert_eq!(cards.len(), 5, "{markup}");
    assert!(!markup.contains(&peg_id), "{markup}");
    assert!(!markup.contains("GHOST"), "{markup}");
    assert_no_price_or_amount(&markup);
    assert!(asset_cards_markup(&registry_of(&[pegged(&peg_id)]), true).is_none());
    assert!(asset_cards_markup(&registry_of(&[empty_name(&asset_id(8), "GHOST")]), true).is_none());
    assert!(asset_cards_markup("[]", true).is_none());
    assert!(asset_cards_markup("[ ]", true).is_none());
}

/// Mainnet does not request the liquid registry and does not paint a card.
#[test]
fn mainnet_does_not_paint_featured_cards() {
    assert!(registry_list_request(false).is_none());
    assert!(asset_cards_markup(&five_registry(), false).is_none());
    let screen = include_str!("../src/views.rs");
    assert!(
        screen.contains("asset_cards_markup"),
        "assets screen does not paint featured cards"
    );
    assert!(screen.contains("registry_list_request"));
    assert!(!screen.contains("/assets/featured"));
    assert!(!screen.contains("spinner-border"));
    let liquid = include_str!("../src/liquid.rs");
    assert!(liquid.contains("assets_registry_path"));
    assert!(!liquid.contains("/assets/featured"));
}
