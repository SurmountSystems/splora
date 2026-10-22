//! Address-page badge for a mainnet pay-to-witness-pubkey-hash address.
//!
//! After a successful no-query `GET /address/:addr`, the page paints the
//! visible badge `P2WPKH`. The address JSON has no type field. The badge is
//! classified from the address string. The row label `Type` sits beside it.
//! A different address shape does not paint `P2WPKH`. A mainnet `bc1q` plus
//! 58 `q` characters paints `P2WSH` once and does not paint `P2WPKH`. A
//! mainnet `bc1p` plus 58 `p` characters paints `P2TR` once and does not
//! paint `P2WPKH`, `P2WSH`, `P2PKH`, or `P2SH`. Every other case in this
//! file still does not paint `P2TR`. A mainnet address that starts with `1`
//! paints `P2PKH` once and does not paint `P2WPKH`, `P2WSH`, `P2TR`, or
//! `P2SH`. A `bc1q` address, a `bc1p` address, and a mainnet address that
//! starts with `3` do not paint `P2PKH`. A mainnet address that starts with
//! `3` paints `P2SH` once and does not paint `P2WPKH`. Every other case in
//! this file still does not paint `P2SH`.

use splora_web::{address_requests, address_rows_markup};

const ADDR: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
const TYPE_AND_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WPKH</span></span>";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn bech32_lw(ch: char) -> bool {
    matches!(ch, 'a' | 'c'..='h' | 'j'..='n' | 'p'..='z' | '0'..='9')
}

fn mainnet_v0_p2wpkh_shape(address: &str) -> bool {
    let Some(suffix) = address.strip_prefix("bc1") else {
        return false;
    };
    let mut chars = suffix.chars();
    if chars.next() != Some('q') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    rest.len() == 38 && rest.iter().copied().all(bech32_lw)
}

fn address_page_source() -> &'static str {
    let src = include_str!("../src/views.rs");
    let marker = "fn AddressPage(";
    let start = src
        .find(marker)
        .expect("AddressPage is missing from views.rs");
    let after_name = &src[start..];
    let brace_rel = after_name.find('{').expect("AddressPage has no body");
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
    panic!("AddressPage body is not closed");
}

fn assert_no_other_type_badges(markup: &str) {
    assert!(!markup.contains(">P2PKH<"), "{markup}");
    assert!(!markup.contains(">P2SH<"), "{markup}");
    assert!(!markup.contains(">P2TR<"), "{markup}");
    assert!(!markup.contains(">P2WSH<"), "{markup}");
}

fn assert_no_p2wpkh_badge(address: &str) {
    let markup = address_rows_markup(&address_body(address), "", "");
    assert!(
        !markup.contains(">P2WPKH<"),
        "a different address shape painted P2WPKH: {address} {markup}"
    );
    assert_no_other_type_badges(&markup);
}

#[test]
fn address_page_paints_p2wpkh_badge_after_no_query_get() {
    assert!(
        mainnet_v0_p2wpkh_shape(ADDR),
        "fixture is not a mainnet bc1q + 38 bech32 character address"
    );
    let body = address_body(ADDR);
    assert!(body.contains("\"address\""));
    assert!(body.contains("\"chain_stats\""));
    assert!(body.contains("\"mempool_stats\""));
    assert!(
        !body.contains("\"type\""),
        "address JSON must not carry a type field: {body}"
    );
    let calls = address_requests(ADDR);
    assert!(
        calls.iter().any(|call| {
            call.method == "GET"
                && call.path == format!("/address/{ADDR}")
                && !call.path.contains('?')
        }),
        "no-query GET /address/:addr is missing: {calls:?}"
    );
    assert!(
        calls
            .iter()
            .all(|call| call.method == "GET" && !call.path.contains('?')),
        "address read added a query: {calls:?}"
    );

    let markup = address_rows_markup(&body, "", "");
    assert!(markup.contains(TYPE_AND_BADGE), "badge is absent: {markup}");
    assert_eq!(
        markup.matches(TYPE_AND_BADGE).count(),
        1,
        "P2WPKH badge count: {markup}"
    );
    assert!(
        !markup.contains("V0_P2WPKH"),
        "address badge was uppercased into the output type: {markup}"
    );
    assert!(!markup.contains("title=\"P2WPKH\""), "{markup}");
    assert!(!markup.contains("aria-label=\"P2WPKH\""), "{markup}");
    assert_no_other_type_badges(&markup);
    assert!(
        !markup.contains("output-type"),
        "output type row changed: {markup}"
    );

    let page = address_page_source();
    assert!(
        page.contains("address_rows_markup("),
        "AddressPage does not paint the address rows"
    );
    assert!(
        page.contains("address_type_badge_markup("),
        "AddressPage does not paint the P2WPKH badge from the address string: {page}"
    );
}

#[test]
fn a_different_address_shape_does_not_paint_p2wpkh() {
    let p2pkh = address_rows_markup(&address_body("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"), "", "");
    assert_eq!(
        p2pkh.matches(">P2PKH<").count(),
        1,
        "P2PKH badge count: {p2pkh}"
    );
    assert!(!p2pkh.contains(">P2WPKH<"), "{p2pkh}");
    assert!(!p2pkh.contains(">P2SH<"), "{p2pkh}");
    assert!(!p2pkh.contains(">P2TR<"), "{p2pkh}");
    assert!(!p2pkh.contains(">P2WSH<"), "{p2pkh}");
    let nested = address_rows_markup(&address_body("3EktnHQD7RiAE6uzMj2ZifT9YgRrkSgzQX"), "", "");
    assert!(
        !nested.contains(">P2WPKH<"),
        "a mainnet address that starts with 3 painted P2WPKH: {nested}"
    );
    assert_eq!(
        nested.matches(">P2SH<").count(),
        1,
        "plain P2SH badge count: {nested}"
    );
    assert!(!nested.contains(">P2PKH<"), "{nested}");
    assert!(!nested.contains(">P2TR<"), "{nested}");
    assert!(!nested.contains(">P2WSH<"), "{nested}");
    assert_no_p2wpkh_badge("bc1qexample");
    assert_no_p2wpkh_badge(&format!("tb1{}", ADDR.trim_start_matches("bc1")));
    assert_no_p2wpkh_badge(&ADDR.to_ascii_uppercase());
    let p2wsh = format!("bc1q{}", "q".repeat(58));
    let p2wsh_markup = address_rows_markup(&address_body(&p2wsh), "", "");
    assert!(
        !p2wsh_markup.contains(">P2WPKH<"),
        "a P2WSH address painted P2WPKH: {p2wsh} {p2wsh_markup}"
    );
    assert_eq!(
        p2wsh_markup.matches(">P2WSH<").count(),
        1,
        "P2WSH badge count: {p2wsh_markup}"
    );
    assert!(!p2wsh_markup.contains(">P2PKH<"), "{p2wsh_markup}");
    assert!(!p2wsh_markup.contains(">P2SH<"), "{p2wsh_markup}");
    assert!(!p2wsh_markup.contains(">P2TR<"), "{p2wsh_markup}");
    let p2tr = format!("bc1p{}", "p".repeat(58));
    let p2tr_markup = address_rows_markup(&address_body(&p2tr), "", "");
    assert!(
        !p2tr_markup.contains(">P2WPKH<"),
        "a P2TR address painted P2WPKH: {p2tr} {p2tr_markup}"
    );
    assert_eq!(
        p2tr_markup.matches(">P2TR<").count(),
        1,
        "P2TR badge count: {p2tr_markup}"
    );
    assert!(!p2tr_markup.contains(">P2WSH<"), "{p2tr_markup}");
    assert!(!p2tr_markup.contains(">P2PKH<"), "{p2tr_markup}");
    assert!(!p2tr_markup.contains(">P2SH<"), "{p2tr_markup}");
    assert_no_p2wpkh_badge(&format!("bc1q{}", "b".repeat(38)));

    let typed = format!(
        r#"{{"address":"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa","type":"v0_p2wpkh","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0}}}}"#
    );
    let markup = address_rows_markup(&typed, "", "");
    assert!(
        !markup.contains(">P2WPKH<"),
        "a type field painted P2WPKH for a different address shape: {markup}"
    );
    assert_eq!(
        markup.matches(">P2PKH<").count(),
        1,
        "P2PKH badge count: {markup}"
    );
    assert!(!markup.contains(">P2SH<"), "{markup}");
    assert!(!markup.contains(">P2TR<"), "{markup}");
    assert!(!markup.contains(">P2WSH<"), "{markup}");

    assert!(!address_rows_markup("", "", "").contains(">P2WPKH<"));
    assert!(!address_rows_markup("not-json", "", "").contains(">P2WPKH<"));
}
