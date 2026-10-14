//! Confirmation count on the transaction page.
//!
//! The count is the tip height minus `status.block_height`, plus 1.
//! A missing block height is unconfirmed and omits the count. It is not zero.
//! An empty or unparsed tip body omits the count and does not paint Unconfirmed.
//! When that tip is absent and `status.block_height` is set, the label is the
//! word Confirmed once. That word is not the substring inside Unconfirmed or
//! Confirmations.
//! A parsed tip with no block height paints the word Unconfirmed once.
//! Script hex stays off the page.

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

/// Whole-word count. `Unconfirmed` is one token, not the word Confirmed.
/// `Confirmations` is one token, not the word Confirmed.
fn whole_word_count(markup: &str, word: &str) -> usize {
    markup
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| *token == word)
        .count()
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
fn parsed_tip_without_a_count_paints_unconfirmed_once() {
    let unconfirmed = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "100");
    let expected = "<p class=\"unconfirmed\">Unconfirmed</p>";
    assert_eq!(
        unconfirmed.matches(expected).count(),
        1,
        "expected one {expected} in {unconfirmed}"
    );
    assert_eq!(
        unconfirmed.matches("Unconfirmed").count(),
        1,
        "expected Unconfirmed once in {unconfirmed}"
    );
    assert!(
        !unconfirmed.contains("class=\"confirmations\""),
        "{unconfirmed}"
    );
    assert!(!unconfirmed.contains("Confirmations 0"), "{unconfirmed}");
    assert_scripts_stay_off(&unconfirmed);

    let counted = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "100");
    assert!(
        !counted.contains("Unconfirmed"),
        "a known count must not paint Unconfirmed: {counted}"
    );
    assert_scripts_stay_off(&counted);

    let empty_tip = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "");
    assert!(
        !empty_tip.contains("Unconfirmed"),
        "an empty tip must not paint Unconfirmed: {empty_tip}"
    );
    assert_scripts_stay_off(&empty_tip);
}

#[test]
fn missing_tip_with_block_height_paints_confirmed_once() {
    let confirmed = "<p class=\"confirmed\">Confirmed</p>";
    assert_eq!(
        whole_word_count("Unconfirmed", "Confirmed"),
        0,
        "Unconfirmed is not the word Confirmed"
    );
    assert_eq!(
        whole_word_count("Confirmations 11", "Confirmed"),
        0,
        "Confirmations is not the word Confirmed"
    );
    assert_eq!(whole_word_count("Confirmed", "Confirmed"), 1);

    let empty_tip = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "");
    assert_eq!(
        empty_tip.matches(confirmed).count(),
        1,
        "expected one {confirmed} in {empty_tip}"
    );
    assert_eq!(
        whole_word_count(&empty_tip, "Confirmed"),
        1,
        "expected the word Confirmed once in {empty_tip}"
    );
    assert!(!empty_tip.contains("Unconfirmed"), "{empty_tip}");
    assert!(
        !empty_tip.contains("class=\"confirmations\""),
        "{empty_tip}"
    );
    assert!(!empty_tip.contains("Confirmations 0"), "{empty_tip}");
    assert_scripts_stay_off(&empty_tip);

    let non_integer = confirmation_markup(
        &tx(Some(r#"{"confirmed":true,"block_height":90}"#)),
        "not-a-height",
    );
    assert_eq!(
        non_integer.matches(confirmed).count(),
        1,
        "expected one {confirmed} in {non_integer}"
    );
    assert_eq!(
        whole_word_count(&non_integer, "Confirmed"),
        1,
        "expected the word Confirmed once in {non_integer}"
    );
    assert!(!non_integer.contains("Unconfirmed"), "{non_integer}");
    assert!(
        !non_integer.contains("class=\"confirmations\""),
        "{non_integer}"
    );
    assert!(!non_integer.contains("Confirmations 0"), "{non_integer}");
    assert_scripts_stay_off(&non_integer);

    let counted = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "100");
    assert_one(&counted, 11);
    assert_eq!(
        whole_word_count(&counted, "Confirmed"),
        0,
        "a parsed tip with a block height must not paint the word Confirmed: {counted}"
    );
    assert!(!counted.contains("Unconfirmed"), "{counted}");
    assert_scripts_stay_off(&counted);

    let unconfirmed = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "100");
    let unconfirmed_markup = "<p class=\"unconfirmed\">Unconfirmed</p>";
    assert_eq!(
        unconfirmed.matches(unconfirmed_markup).count(),
        1,
        "expected one {unconfirmed_markup} in {unconfirmed}"
    );
    assert_eq!(
        unconfirmed.matches("Unconfirmed").count(),
        1,
        "expected Unconfirmed once in {unconfirmed}"
    );
    assert_eq!(
        whole_word_count(&unconfirmed, "Confirmed"),
        0,
        "Unconfirmed is not the word Confirmed: {unconfirmed}"
    );
    assert!(!unconfirmed.contains("Confirmations 0"), "{unconfirmed}");
    assert_scripts_stay_off(&unconfirmed);

    let empty_without = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "");
    assert!(!empty_without.contains("Unconfirmed"), "{empty_without}");
    assert!(
        !empty_without.contains("Confirmations 0"),
        "{empty_without}"
    );
    assert_eq!(
        whole_word_count(&empty_without, "Confirmed"),
        0,
        "an empty tip without a block height must not paint Confirmed: {empty_without}"
    );
    assert_scripts_stay_off(&empty_without);

    let omitted_empty = confirmation_markup(&tx(None), "");
    assert!(!omitted_empty.contains("Unconfirmed"), "{omitted_empty}");
    assert!(
        !omitted_empty.contains("Confirmations 0"),
        "{omitted_empty}"
    );
    assert_eq!(
        whole_word_count(&omitted_empty, "Confirmed"),
        0,
        "an empty tip with no status must not paint Confirmed: {omitted_empty}"
    );
    assert_scripts_stay_off(&omitted_empty);

    let bad_without = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "not-a-height");
    assert!(!bad_without.contains("Unconfirmed"), "{bad_without}");
    assert!(!bad_without.contains("Confirmations 0"), "{bad_without}");
    assert_eq!(
        whole_word_count(&bad_without, "Confirmed"),
        0,
        "a non-integer tip without a block height must not paint Confirmed: {bad_without}"
    );
    assert_scripts_stay_off(&bad_without);

    let omitted_bad = confirmation_markup(&tx(None), "not-a-height");
    assert!(!omitted_bad.contains("Unconfirmed"), "{omitted_bad}");
    assert!(!omitted_bad.contains("Confirmations 0"), "{omitted_bad}");
    assert_eq!(
        whole_word_count(&omitted_bad, "Confirmed"),
        0,
        "a non-integer tip with no status must not paint Confirmed: {omitted_bad}"
    );
    assert_scripts_stay_off(&omitted_bad);
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
