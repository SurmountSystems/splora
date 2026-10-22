//! SegWit badge on the transaction page from a no-query `GET /tx/:txid`.
//!
//! `tx-features.component.html` line 2 paints the visible word SegWit when
//! segwit is enabled and the realized gain is nonzero and the potential gain
//! is not. The tooltip stays off. The RBF badge stays off. `P2WPKH` is the
//! address badge, not this one.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BADGE: &str = "<span class=\"badge bg-success me-1\">SegWit</span>";
const WITNESS: &str = "304402";
const LEGACY_SIG: &str = "473044";

fn input(kind: &str, script_sig: &str, witness: &str) -> String {
    let witness_field = if witness.is_empty() {
        String::new()
    } else {
        format!(r#","witness":["{witness}"]"#)
    };
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"{kind}","value":1000}},"scriptsig":"{script_sig}","scriptsig_asm":"","is_coinbase":false,"sequence":4294967293{witness_field}}}"#
    )
}

fn tx(vin: &str, status: &str) -> String {
    let status_field = if status.is_empty() {
        String::new()
    } else {
        format!(r#","status":{status}"#)
    };
    format!(
        r#"{{"txid":"{TXID}","version":2,"locktime":0,"vin":[{vin}],"vout":[{{"scriptpubkey":"76a914bb","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","value":900}}],"size":110,"weight":400,"sigops":1,"fee":100{status_field}}}"#
    )
}

fn native_segwit(status: &str) -> String {
    tx(&input("v0_p2wpkh", "", WITNESS), status)
}

fn paint(body: &str) -> String {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(body)
}

fn assert_badge(markup: &str) {
    assert!(markup.contains("SegWit"), "SegWit is absent: {markup}");
    assert_eq!(
        markup.matches(BADGE).count(),
        1,
        "expected one SegWit badge in {markup}"
    );
    assert_stays_off(markup);
}

fn assert_no_badge(markup: &str) {
    assert!(!markup.contains("SegWit"), "SegWit was painted: {markup}");
    assert_stays_off(markup);
}

fn assert_stays_off(markup: &str) {
    assert!(!markup.contains("RBF"), "RBF badge was painted: {markup}");
    assert!(
        !markup.contains(">P2WPKH<"),
        "address badge was painted: {markup}"
    );
    assert!(!markup.contains("ngbTooltip"), "{markup}");
    assert!(!markup.contains("on fees"), "{markup}");
    assert!(!markup.contains("saved"), "{markup}");
    assert!(!markup.contains("<del"), "{markup}");
}

fn transaction_page_source() -> &'static str {
    let src = include_str!("../src/views.rs");
    let marker = "fn TransactionPage(";
    let start = src
        .find(marker)
        .expect("TransactionPage is missing from views.rs");
    let after_name = &src[start..];
    let brace_rel = after_name.find('{').expect("TransactionPage has no body");
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
    panic!("TransactionPage body is not closed");
}

#[test]
fn unconfirmed_native_segwit_paints_the_badge() {
    let markup = paint(&native_segwit(r#"{"confirmed":false}"#));
    assert_badge(&markup);
    let page = transaction_page_source();
    assert!(
        page.contains("transaction_row_markup("),
        "TransactionPage does not paint the transaction row"
    );
    assert!(
        page.contains("segwit_badge_markup("),
        "TransactionPage does not pass the network into the SegWit badge: {page}"
    );
}

#[test]
fn confirmed_at_mainnet_activation_paints_the_badge() {
    let markup = paint(&native_segwit(
        r#"{"confirmed":true,"block_height":477120}"#,
    ));
    assert_badge(&markup);
}

#[test]
fn gain_zero_omits_the_badge() {
    let legacy = tx(&input("p2pkh", LEGACY_SIG, ""), r#"{"confirmed":false}"#);
    assert_no_badge(&paint(&legacy));
    let empty_witness = tx(&input("v0_p2wpkh", "", ""), r#"{"confirmed":false}"#);
    assert_no_badge(&paint(&empty_witness));
}

#[test]
fn segwit_off_omits_the_badge() {
    let markup = paint(&native_segwit(
        r#"{"confirmed":true,"block_height":477119}"#,
    ));
    assert_no_badge(&markup);
    let no_height = paint(&native_segwit(r#"{"confirmed":true}"#));
    assert_no_badge(&no_height);
    assert_no_badge(&paint(&native_segwit("")));
    assert_no_badge(&paint("not-json"));
    assert_no_badge(&paint(""));
}
