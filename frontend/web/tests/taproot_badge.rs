//! Taproot badge on the transaction page from a no-query `GET /tx/:txid`.
//!
//! `tx-features.component.html` lines 11-25 paint the visible word Taproot
//! when taproot is enabled. The tooltip stays off. SegWit is a different
//! badge. The RBF badge stays off. `P2WPKH` is the address badge. Runestone
//! stays. Error loading address data and Error loading block data stay.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BADGE: &str = "<span class=\"badge bg-success me-1\">Taproot</span>";
const SCRIPT: &str = "00112233445566778899aabbccddeeff";
const SCRIPT_ASM: &str = "OP_DUP_TAPROOT_FIXTURE";
const SCRIPTSIG: &str = "deadbeefscriptsig";
const SCRIPTSIG_ASM: &str = "OP_PUSHBYTES_TAPROOT_FIXTURE";
const WITNESS: &str = "cafebabewitness";

fn tx(status: &str) -> String {
    let status_field = if status.is_empty() {
        String::new()
    } else {
        format!(r#","status":{status}"#)
    };
    format!(
        r#"{{"txid":"{TXID}","version":2,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"{SCRIPT}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1000}},"scriptsig":"{SCRIPTSIG}","scriptsig_asm":"{SCRIPTSIG_ASM}","is_coinbase":false,"sequence":4294967293,"witness":["{WITNESS}"]}}],"vout":[{{"scriptpubkey":"{SCRIPT}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":900}}],"size":110,"weight":400,"sigops":1,"fee":100{status_field}}}"#
    )
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
    assert!(markup.contains("Taproot"), "Taproot is absent: {markup}");
    assert_eq!(
        markup.matches(BADGE).count(),
        1,
        "expected one Taproot badge in {markup}"
    );
    assert!(
        !markup.contains("SegWit"),
        "SegWit is a different badge: {markup}"
    );
    assert_stays_off(markup);
}

fn assert_no_badge(markup: &str) {
    assert!(!markup.contains("Taproot"), "Taproot was painted: {markup}");
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
    assert!(!markup.contains(SCRIPT), "script hex was painted: {markup}");
    assert!(
        !markup.contains(SCRIPT_ASM),
        "script asm was painted: {markup}"
    );
    assert!(
        !markup.contains(SCRIPTSIG),
        "scriptsig was painted: {markup}"
    );
    assert!(
        !markup.contains(SCRIPTSIG_ASM),
        "scriptsig asm was painted: {markup}"
    );
    assert!(!markup.contains(WITNESS), "witness was painted: {markup}");
    assert!(
        !markup.contains("Runestone"),
        "Runestone was painted: {markup}"
    );
    let src = include_str!("../src/rows.rs");
    assert!(
        src.contains("Error loading address data."),
        "address load error was removed"
    );
    assert!(
        src.contains("Error loading block data."),
        "block load error was removed"
    );
    assert!(
        src.contains(">Runestone</button>"),
        "Runestone button was removed"
    );
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
fn unconfirmed_paints_the_badge() {
    let markup = paint(&tx(r#"{"confirmed":false}"#));
    assert_badge(&markup);
    let page = transaction_page_source();
    assert!(
        page.contains("transaction_row_markup("),
        "TransactionPage does not paint the transaction row"
    );
    assert!(
        page.contains("taproot_badge_markup("),
        "TransactionPage does not pass the network into the Taproot badge: {page}"
    );
}

#[test]
fn confirmed_at_mainnet_activation_paints_the_badge() {
    let markup = paint(&tx(r#"{"confirmed":true,"block_height":709632}"#));
    assert_badge(&markup);
}

#[test]
fn taproot_off_omits_the_badge() {
    let markup = paint(&tx(r#"{"confirmed":true,"block_height":709631}"#));
    assert_no_badge(&markup);
    let no_height = paint(&tx(r#"{"confirmed":true}"#));
    assert_no_badge(&no_height);
    assert_no_badge(&paint(&tx("")));
    assert_no_badge(&paint("not-json"));
    assert_no_badge(&paint(""));
}
