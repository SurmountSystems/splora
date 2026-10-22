//! Address-page badge for a mainnet pay-to-witness-script-hash address.
//!
//! After a successful no-query `GET /address/:addr`, a mainnet address that
//! starts with `bc1q` and then has 58 bech32 characters paints the visible
//! badge `P2WSH` beside the row word `Type`. The address JSON has no type
//! field. The badge is classified from the address string. The shorter
//! `bc1q` shape (`q` plus 38 bech32 characters) stays `P2WPKH` and does not
//! paint `P2WSH`. The badge is not the output type `V0_P2WSH`.

use splora_web::{address_requests, address_rows_markup};

const P2WSH_ADDR: &str = "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3";
const P2WPKH_ADDR: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
const P2WSH_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WSH</span></span>";
const P2WPKH_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WPKH</span></span>";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn bech32_lw(ch: char) -> bool {
    matches!(ch, 'a' | 'c'..='h' | 'j'..='n' | 'p'..='z' | '0'..='9')
}

fn mainnet_bc1q_shape(address: &str, data_len: usize) -> bool {
    let Some(suffix) = address.strip_prefix("bc1") else {
        return false;
    };
    let mut chars = suffix.chars();
    if chars.next() != Some('q') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    rest.len() == data_len && rest.iter().copied().all(bech32_lw)
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

#[test]
fn address_page_paints_p2wsh_badge_after_no_query_get() {
    assert!(
        mainnet_bc1q_shape(P2WSH_ADDR, 58),
        "fixture is not a mainnet bc1q address with 58 bech32 characters"
    );
    let body = address_body(P2WSH_ADDR);
    assert!(body.contains("\"address\""));
    assert!(body.contains("\"chain_stats\""));
    assert!(body.contains("\"mempool_stats\""));
    assert!(
        !body.contains("\"type\""),
        "address JSON must not carry a type field: {body}"
    );
    let calls = address_requests(P2WSH_ADDR);
    assert!(
        calls.iter().any(|call| {
            call.method == "GET"
                && call.path == format!("/address/{P2WSH_ADDR}")
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
    assert!(
        markup.contains(P2WSH_BADGE),
        "P2WSH badge is absent: {markup}"
    );
    assert_eq!(
        markup.matches(P2WSH_BADGE).count(),
        1,
        "P2WSH badge count: {markup}"
    );
    assert!(
        !markup.contains(">P2WPKH<"),
        "the longer bc1q shape painted P2WPKH: {markup}"
    );
    assert!(
        !markup.contains("V0_P2WSH"),
        "address badge was uppercased into the output type: {markup}"
    );
    assert!(!markup.contains("title=\"P2WSH\""), "{markup}");
    assert!(!markup.contains("aria-label=\"P2WSH\""), "{markup}");
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
        "AddressPage does not paint the address badge from the address string: {page}"
    );
}

#[test]
fn shorter_bc1q_stays_p2wpkh_and_does_not_show_p2wsh() {
    assert!(
        mainnet_bc1q_shape(P2WPKH_ADDR, 38),
        "fixture is not a mainnet bc1q address with 38 bech32 characters"
    );
    let markup = address_rows_markup(&address_body(P2WPKH_ADDR), "", "");
    assert!(
        markup.contains(P2WPKH_BADGE),
        "P2WPKH badge is absent: {markup}"
    );
    assert_eq!(
        markup.matches(P2WPKH_BADGE).count(),
        1,
        "P2WPKH badge count: {markup}"
    );
    assert!(
        !markup.contains(">P2WSH<"),
        "the shorter bc1q shape painted P2WSH: {markup}"
    );
    assert!(
        !markup.contains("V0_P2WSH"),
        "address badge was uppercased into the output type: {markup}"
    );
}
