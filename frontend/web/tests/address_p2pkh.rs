//! Address-page badge for a mainnet pay-to-pubkey-hash address.
//!
//! After a successful no-query `GET /address/:addr`, a mainnet address whose
//! first character is `1` paints the visible badge `P2PKH` beside the row
//! word `Type`. Angular has no dedicated `p2pkh` case. The default arm
//! uppercases the type, so the span text is `P2PKH`. The address JSON has no
//! type field. The badge is classified from the address string. A `P2WPKH`
//! address, a `P2WSH` address, a `P2TR` address, and a mainnet address that
//! starts with `3` do not paint `P2PKH`. A failed address load keeps
//! `Error loading address data.` and the indexer body `Invalid Bitcoin address`
//! and does not paint this badge.

use splora_web::{address_requests, address_rows_markup, indexer_failure_text};

/// Cypress fixture `details_rbf/tx02_api_tx.json`, `scriptpubkey_type` `p2pkh`.
const P2PKH_ADDR: &str = "1stAprEZapjCYGUACUcXohQqSHF9MU5Kj";
const P2WPKH_ADDR: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
const P2WSH_ADDR: &str = "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3";
const P2TR_ADDR: &str = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
/// `api-docs-data.ts` mainnet address. First character is `3`.
const P2SH_ADDR: &str = "3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB";
const P2PKH_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2PKH</span></span>";
const P2WPKH_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WPKH</span></span>";
const P2WSH_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2WSH</span></span>";
const P2TR_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2TR</span></span>";
const LOAD_ERROR: &str = "Error loading address data.";
const INDEXER_BODY: &str = "Invalid Bitcoin address";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn base58_char(ch: char) -> bool {
    matches!(
        ch,
        'a'..='k' | 'm'..='z' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | '1'..='9'
    )
}

/// `detectAddressType` on mainnet: first character `1`, then `base58Regex`.
fn mainnet_p2pkh_shape(address: &str) -> bool {
    let mut chars = address.chars();
    if chars.next() != Some('1') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    (26..=34).contains(&rest.len()) && rest.iter().copied().all(base58_char)
}

fn bech32_lw(ch: char) -> bool {
    matches!(ch, 'a' | 'c'..='h' | 'j'..='n' | 'p'..='z' | '0'..='9')
}

fn mainnet_bc1_shape(address: &str, second: char, data_len: usize) -> bool {
    let Some(suffix) = address.strip_prefix("bc1") else {
        return false;
    };
    let mut chars = suffix.chars();
    if chars.next() != Some(second) {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    rest.len() == data_len && rest.iter().copied().all(bech32_lw)
}

fn mainnet_p2sh_shape(address: &str) -> bool {
    let mut chars = address.chars();
    if chars.next() != Some('3') {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    (26..=34).contains(&rest.len()) && rest.iter().copied().all(base58_char)
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
fn address_page_paints_p2pkh_badge_after_no_query_get() {
    assert!(
        mainnet_p2pkh_shape(P2PKH_ADDR),
        "fixture is not a mainnet address that detectAddressType classifies as p2pkh"
    );
    let body = address_body(P2PKH_ADDR);
    assert!(body.contains("\"address\""));
    assert!(body.contains("\"chain_stats\""));
    assert!(body.contains("\"mempool_stats\""));
    assert!(
        !body.contains("\"type\""),
        "address JSON must not carry a type field: {body}"
    );
    let calls = address_requests(P2PKH_ADDR);
    assert!(
        calls.iter().any(|call| {
            call.method == "GET"
                && call.path == format!("/address/{P2PKH_ADDR}")
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
        markup.contains(P2PKH_BADGE),
        "P2PKH badge is absent: {markup}"
    );
    assert_eq!(
        markup.matches(P2PKH_BADGE).count(),
        1,
        "P2PKH badge count: {markup}"
    );
    assert_eq!(
        markup.matches(">P2PKH<").count(),
        1,
        "P2PKH badge text count: {markup}"
    );
    assert!(!markup.contains(">P2WPKH<"), "{markup}");
    assert!(!markup.contains(">P2WSH<"), "{markup}");
    assert!(!markup.contains(">P2TR<"), "{markup}");
    assert!(!markup.contains(">P2SH<"), "{markup}");
    assert!(!markup.contains("title=\"P2PKH\""), "{markup}");
    assert!(!markup.contains("aria-label=\"P2PKH\""), "{markup}");
    assert!(
        !markup.contains("output-type"),
        "output type row changed: {markup}"
    );
    assert!(
        !markup.contains(LOAD_ERROR),
        "a successful address load painted the error span: {markup}"
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

    let path = format!("/address/{P2PKH_ADDR}");
    let failed = indexer_failure_text("GET", &path, INDEXER_BODY);
    assert!(failed.contains(INDEXER_BODY), "{failed}");
    assert!(
        !failed.contains(LOAD_ERROR),
        "the indexer body became the span: {failed}"
    );
    let error_markup = address_rows_markup(&failed, "", "");
    assert!(
        error_markup.contains(LOAD_ERROR),
        "Error loading address data. is absent: {error_markup}"
    );
    assert!(
        error_markup.contains(INDEXER_BODY),
        "Invalid Bitcoin address was replaced or dropped: {error_markup}"
    );
    assert!(
        !error_markup.contains(">P2PKH<"),
        "a failed address load painted P2PKH: {error_markup}"
    );
}

#[test]
fn other_address_shapes_do_not_show_p2pkh() {
    assert!(
        mainnet_bc1_shape(P2WPKH_ADDR, 'q', 38),
        "fixture is not a mainnet bc1q address with 38 bech32 characters"
    );
    assert!(
        mainnet_bc1_shape(P2WSH_ADDR, 'q', 58),
        "fixture is not a mainnet bc1q address with 58 bech32 characters"
    );
    assert!(
        mainnet_bc1_shape(P2TR_ADDR, 'p', 58),
        "fixture is not a mainnet bc1 address with p plus 58 bech32 characters"
    );
    assert!(
        mainnet_p2sh_shape(P2SH_ADDR),
        "fixture is not a mainnet address whose first character is 3"
    );

    let p2wpkh = address_rows_markup(&address_body(P2WPKH_ADDR), "", "");
    assert!(
        p2wpkh.contains(P2WPKH_BADGE),
        "P2WPKH badge is absent: {p2wpkh}"
    );
    assert!(
        !p2wpkh.contains(">P2PKH<"),
        "a P2WPKH address painted P2PKH: {p2wpkh}"
    );

    let p2wsh = address_rows_markup(&address_body(P2WSH_ADDR), "", "");
    assert!(
        p2wsh.contains(P2WSH_BADGE),
        "P2WSH badge is absent: {p2wsh}"
    );
    assert!(
        !p2wsh.contains(">P2PKH<"),
        "a P2WSH address painted P2PKH: {p2wsh}"
    );

    let p2tr = address_rows_markup(&address_body(P2TR_ADDR), "", "");
    assert!(p2tr.contains(P2TR_BADGE), "P2TR badge is absent: {p2tr}");
    assert!(
        !p2tr.contains(">P2PKH<"),
        "a P2TR address painted P2PKH: {p2tr}"
    );

    let p2sh = address_rows_markup(&address_body(P2SH_ADDR), "", "");
    assert!(
        !p2sh.contains(">P2PKH<"),
        "a mainnet address that starts with 3 painted P2PKH: {p2sh}"
    );
}
