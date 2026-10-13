//! Confirmation count on the transaction page.
//!
//! The count is the tip height minus `status.block_height`, plus 1.
//! A missing block height is unconfirmed and omits the count. It is not zero.
//! An empty or unparsed tip body omits the count. Script hex stays off the page.

use splora_web::confirmation_markup;

const SCRIPT_HEX: &str = "CONFHEXDEADBEEF";

fn tx(status: Option<&str>) -> String {
    let status_field = match status {
        Some(status) => format!(r#","status":{status}"#),
        None => String::new(),
    };
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"CONFSIGHEXDEAD","scriptsig_asm":"CONFSIGASMDEAD","witness":["CONFWITNESSDEAD"],"is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"CONFASMDEAD","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":4,"sigops":0,"fee":1{status_field}}}"#
    )
}

fn paragraph(count: i128) -> String {
    format!("<p class=\"confirmations\">Confirmations {count}</p>")
}

fn assert_one(markup: &str, count: i128) {
    let expected = paragraph(count);
    assert_eq!(
        markup.matches(&expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

fn assert_scripts_stay_off(markup: &str) {
    for needle in [
        SCRIPT_HEX,
        "CONFSIGHEXDEAD",
        "CONFSIGASMDEAD",
        "CONFASMDEAD",
        "CONFWITNESSDEAD",
    ] {
        assert!(
            !markup.contains(needle),
            "{needle} leaked into the markup: {markup}"
        );
    }
}

fn transaction_page() -> &'static str {
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
fn confirmations_are_tip_minus_block_plus_one() {
    let markup = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "100");
    assert_one(&markup, 11);
    assert_scripts_stay_off(&markup);
}

#[test]
fn transaction_in_the_tip_has_one_confirmation() {
    let markup = confirmation_markup(
        &tx(Some(r#"{"confirmed":true,"block_height":800000}"#)),
        "800000",
    );
    assert_one(&markup, 1);
    assert_scripts_stay_off(&markup);
}

#[test]
fn missing_block_height_omits_the_count() {
    let unconfirmed = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "100");
    assert!(
        !unconfirmed.contains("class=\"confirmations\""),
        "{unconfirmed}"
    );
    assert!(!unconfirmed.contains("Confirmations 101"), "{unconfirmed}");
    assert!(!unconfirmed.contains("Confirmations 0"), "{unconfirmed}");
    assert_scripts_stay_off(&unconfirmed);

    let omitted = confirmation_markup(&tx(None), "100");
    assert!(!omitted.contains("class=\"confirmations\""), "{omitted}");
    assert!(!omitted.contains("Confirmations 101"), "{omitted}");
    assert!(!omitted.contains("Confirmations 0"), "{omitted}");
    assert_scripts_stay_off(&omitted);
}

#[test]
fn missing_tip_omits_the_count() {
    let markup = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "");
    assert!(!markup.contains("class=\"confirmations\""), "{markup}");
    assert!(!markup.contains("Confirmations 0"), "{markup}");
    assert_scripts_stay_off(&markup);
}

#[test]
fn unparsed_tip_omits_the_count() {
    let markup = confirmation_markup(
        &tx(Some(r#"{"confirmed":true,"block_height":90}"#)),
        "not-a-height",
    );
    assert!(!markup.contains("class=\"confirmations\""), "{markup}");
    assert_scripts_stay_off(&markup);
}

#[test]
fn transaction_page_uses_the_existing_tip_height() {
    let page = transaction_page();
    assert!(
        page.contains("blocks_tip_height_path"),
        "TransactionPage does not schedule the existing tip height: {page}"
    );
    assert!(
        page.contains("confirmation_markup"),
        "TransactionPage does not paint the confirmation count: {page}"
    );
    assert!(
        !page.contains("800000"),
        "TransactionPage hardcodes a tip height: {page}"
    );
}
