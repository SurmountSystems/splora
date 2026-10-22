//! Address-page badge for a mainnet pay-to-taproot address.
//!
//! After a successful no-query `GET /address/:addr`, a mainnet address that
//! starts with `bc1` and then `p` plus 58 bech32 characters paints the visible
//! badge `P2TR` beside the row word `Type`. The address JSON has no type
//! field. The badge is classified from the address string. A `P2WPKH` address
//! and a `P2WSH` address do not paint `P2TR`. The badge is not `V1_P2TR` and
//! it is not the input label `P2TR tapscript`.

use splora_web::{address_requests, address_rows_markup};

const P2TR_ADDR: &str = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
const P2WPKH_ADDR: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
const P2WSH_ADDR: &str = "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3";
const P2TR_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2TR</span></span>";
const P2WPKH_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WPKH</span></span>";
const P2WSH_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WSH</span></span>";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn bech32_lw(ch: char) -> bool {
    matches!(ch, 'a' | 'c'..='h' | 'j'..='n' | 'p'..='z' | '0'..='9')
}

fn mainnet_bc1p_shape(address: &str) -> bool {
    let Some(suffix) = address.strip_prefix("bc1") else {
        return false;
    };
    let mut chars = suffix.chars();
    if chars.next() != Some('p') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    rest.len() == 58 && rest.iter().copied().all(bech32_lw)
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
fn address_page_paints_p2tr_badge_after_no_query_get() {
    assert!(
        mainnet_bc1p_shape(P2TR_ADDR),
        "fixture is not a mainnet bc1 address with p plus 58 bech32 characters"
    );
    let body = address_body(P2TR_ADDR);
    assert!(body.contains("\"address\""));
    assert!(body.contains("\"chain_stats\""));
    assert!(body.contains("\"mempool_stats\""));
    assert!(
        !body.contains("\"type\""),
        "address JSON must not carry a type field: {body}"
    );
    let calls = address_requests(P2TR_ADDR);
    assert!(
        calls.iter().any(|call| {
            call.method == "GET"
                && call.path == format!("/address/{P2TR_ADDR}")
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
        markup.contains(P2TR_BADGE),
        "P2TR badge is absent: {markup}"
    );
    assert_eq!(
        markup.matches(P2TR_BADGE).count(),
        1,
        "P2TR badge count: {markup}"
    );
    assert_eq!(
        markup.matches(">P2TR<").count(),
        1,
        "P2TR badge text count: {markup}"
    );
    assert!(!markup.contains(">P2WPKH<"), "{markup}");
    assert!(!markup.contains(">P2WSH<"), "{markup}");
    assert!(!markup.contains(">P2PKH<"), "{markup}");
    assert!(!markup.contains(">P2SH<"), "{markup}");
    assert!(
        !markup.contains("P2TR tapscript"),
        "the address badge became the tapscript label: {markup}"
    );
    assert!(
        !markup.contains("V1_P2TR"),
        "address badge was uppercased into the output type: {markup}"
    );
    assert!(!markup.contains("title=\"P2TR\""), "{markup}");
    assert!(!markup.contains("aria-label=\"P2TR\""), "{markup}");
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
fn p2wpkh_and_p2wsh_do_not_show_p2tr() {
    assert!(
        mainnet_bc1q_shape(P2WPKH_ADDR, 38),
        "fixture is not a mainnet bc1q address with 38 bech32 characters"
    );
    assert!(
        mainnet_bc1q_shape(P2WSH_ADDR, 58),
        "fixture is not a mainnet bc1q address with 58 bech32 characters"
    );
    let p2wpkh = address_rows_markup(&address_body(P2WPKH_ADDR), "", "");
    assert!(
        p2wpkh.contains(P2WPKH_BADGE),
        "P2WPKH badge is absent: {p2wpkh}"
    );
    assert_eq!(
        p2wpkh.matches(P2WPKH_BADGE).count(),
        1,
        "P2WPKH badge count: {p2wpkh}"
    );
    assert!(
        !p2wpkh.contains(">P2TR<"),
        "a P2WPKH address painted P2TR: {p2wpkh}"
    );
    let p2wsh = address_rows_markup(&address_body(P2WSH_ADDR), "", "");
    assert!(
        p2wsh.contains(P2WSH_BADGE),
        "P2WSH badge is absent: {p2wsh}"
    );
    assert_eq!(
        p2wsh.matches(P2WSH_BADGE).count(),
        1,
        "P2WSH badge count: {p2wsh}"
    );
    assert!(
        !p2wsh.contains(">P2TR<"),
        "a P2WSH address painted P2TR: {p2wsh}"
    );
}
