//! Confirmation count on the transaction page.
//!
//! The count is the tip height minus `status.block_height`, plus 1.
//! A missing block height is unconfirmed and omits the count. It is not zero.
//! An empty or unparsed tip body omits the count and does not paint Unconfirmed.
//! When that tip is absent and `status.block_height` is set, the label is the
//! word Confirmed once. That word is not the substring inside Unconfirmed or
//! Confirmations.
//! A parsed tip with no block height paints the paragraph once and one line
//! whose entire text is Unconfirmed.
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
    // The paragraph and the whole line each contain the word.
    assert_eq!(
        unconfirmed.matches("Unconfirmed").count(),
        2,
        "expected Unconfirmed in the paragraph and on its own line in {unconfirmed}"
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
        2,
        "expected the word Confirmed in the paragraph and on its own line in {empty_tip}"
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
        2,
        "expected the word Confirmed in the paragraph and on its own line in {non_integer}"
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
    // The paragraph and the whole line each contain the word.
    assert_eq!(
        unconfirmed.matches("Unconfirmed").count(),
        2,
        "expected Unconfirmed in the paragraph and on its own line in {unconfirmed}"
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

/// Whole line `Unconfirmed` when the tip parses and `status.block_height` is absent.
///
/// GPUI `transaction_screen_paints_unconfirmed` requires one line whose entire
/// text equals Unconfirmed. The paragraph stays. A parsed tip with a height
/// paints the count and omits this line. An empty tip omits this line.
#[test]
fn parsed_tip_without_height_paints_unconfirmed_as_its_own_line() {
    const LINE: &str = "Unconfirmed";
    let markup = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "100");
    assert!(
        markup.contains("<p class=\"unconfirmed\">Unconfirmed</p>"),
        "the paragraph was dropped: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );

    let counted = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "100");
    assert!(
        !counted.lines().any(|line| line == LINE),
        "a parsed tip with a block height painted {LINE}: {counted}"
    );

    let empty_tip = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "");
    assert!(
        !empty_tip.lines().any(|line| line == LINE),
        "an empty tip painted {LINE}: {empty_tip}"
    );

    let confirmed = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":4}"#)), "");
    assert!(
        !confirmed.lines().any(|line| line == LINE),
        "Confirmed painted {LINE}: {confirmed}"
    );

    let neither = confirmation_markup(&tx(None), "");
    assert!(
        !neither.lines().any(|line| line == LINE),
        "no tip and no status painted {LINE}: {neither}"
    );
}

/// Whole line `Confirmed` when the tip is absent and `status.block_height` is set.
///
/// GPUI `assert_one_fee_and_confirmed` requires one line whose entire text
/// equals `Confirmed`. The paragraph stays. A parsed tip paints the count
/// instead. A missing height omits the line. `Unconfirmed` is not this line.
#[test]
fn missing_tip_paints_confirmed_as_its_own_line() {
    const LINE: &str = "Confirmed";
    let markup = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":4}"#)), "");
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert_eq!(
        markup
            .matches("<p class=\"confirmed\">Confirmed</p>")
            .count(),
        1,
        "the Confirmed paragraph was dropped: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "confirmed" || line == "Unconfirmed"),
        "confirmation letter case changed: {markup}"
    );

    let non_integer = confirmation_markup(
        &tx(Some(r#"{"confirmed":true,"block_height":4}"#)),
        "not-a-height",
    );
    assert!(
        non_integer.lines().any(|line| line == LINE),
        "no line equals {LINE}: {non_integer}"
    );
    assert_eq!(
        non_integer.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {non_integer}"
    );

    let counted = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":4}"#)), "100");
    assert!(
        !counted.lines().any(|line| line == LINE),
        "a parsed tip painted {LINE}: {counted}"
    );

    let missing_height = confirmation_markup(&tx(Some(r#"{"confirmed":true}"#)), "");
    assert!(
        !missing_height.lines().any(|line| line == LINE),
        "a missing height painted {LINE}: {missing_height}"
    );

    let unconfirmed = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "100");
    assert!(
        !unconfirmed.lines().any(|line| line == LINE),
        "Unconfirmed painted {LINE}: {unconfirmed}"
    );
}

/// Whole line `confirmations 1` when the tip and the block height agree.
///
/// GPUI `assert_badge_neighbors` requires one line whose entire text equals
/// `confirmations 1`. The capital paragraph stays. A count other than 1 omits
/// this line. An omitted count omits this line. `Confirmations 1` is not this line.
#[test]
fn transaction_in_the_tip_paints_confirmations_1_as_its_own_line() {
    const LINE: &str = "confirmations 1";
    let markup = confirmation_markup(
        &tx(Some(r#"{"confirmed":true,"block_height":800000}"#)),
        "800000",
    );
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert_one(&markup, 1);
    assert!(
        !markup.lines().any(|line| line == "Confirmations 1"),
        "a capital whole line was painted: {markup}"
    );

    let other = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "100");
    assert!(
        !other.lines().any(|line| line == LINE),
        "a count other than 1 painted {LINE}: {other}"
    );
    assert_one(&other, 11);

    let omitted = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "100");
    assert!(
        !omitted.lines().any(|line| line == LINE),
        "an omitted count painted {LINE}: {omitted}"
    );

    let empty_tip =
        confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":800000}"#)), "");
    assert!(
        !empty_tip.lines().any(|line| line == LINE),
        "an empty tip painted {LINE}: {empty_tip}"
    );
}

/// Whole line `confirmations 11` when the tip is 100 and the block height is 90.
///
/// The count is the tip minus the block height, plus one. The capital paragraph
/// stays. A count of 1 still paints `confirmations 1` and does not paint this
/// line. Unconfirmed and a missing tip omit this line. `Confirmations 11` is
/// not this line.
#[test]
fn transaction_eleven_confirmations_paints_confirmations_11_as_its_own_line() {
    const LINE: &str = "confirmations 11";
    let markup = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "100");
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert_one(&markup, 11);
    assert!(
        !markup.lines().any(|line| line == "Confirmations 11"),
        "a capital whole line was painted: {markup}"
    );

    let one = confirmation_markup(
        &tx(Some(r#"{"confirmed":true,"block_height":800000}"#)),
        "800000",
    );
    assert!(
        one.lines().any(|line| line == "confirmations 1"),
        "a count of 1 did not paint confirmations 1: {one}"
    );
    assert_eq!(
        one.lines()
            .filter(|line| *line == "confirmations 1")
            .count(),
        1,
        "confirmations 1 was not painted once: {one}"
    );
    assert!(
        !one.lines().any(|line| line == LINE),
        "a count of 1 painted {LINE}: {one}"
    );
    assert_one(&one, 1);

    let unconfirmed = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "100");
    assert!(
        !unconfirmed.lines().any(|line| line == LINE),
        "unconfirmed painted {LINE}: {unconfirmed}"
    );

    let missing_tip = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "");
    assert!(
        !missing_tip.lines().any(|line| line == LINE),
        "a missing tip painted {LINE}: {missing_tip}"
    );
}

/// Whole line `confirmations 3` when the tip is 800000 and the block height is 799998.
///
/// GPUI `transaction_screen_paints_confirmation_count` requires one line whose
/// entire text equals `confirmations 3`. The count is the tip minus the block
/// height, plus one. The capital paragraph stays. A count of 1 still paints
/// `confirmations 1` and does not paint this line. A count of 11 still paints
/// `confirmations 11` and does not paint this line. `Confirmations 3` is not
/// this line.
#[test]
fn transaction_three_confirmations_paints_confirmations_3_as_its_own_line() {
    const LINE: &str = "confirmations 3";
    let markup = confirmation_markup(
        &tx(Some(r#"{"confirmed":true,"block_height":799998}"#)),
        "800000",
    );
    assert!(
        markup.lines().any(|line| line == LINE),
        "height 799998 did not paint {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert_one(&markup, 3);
    assert!(
        !markup.lines().any(|line| line == "Confirmations 3"),
        "a capital whole line was painted: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "confirmations 1"),
        "height 799998 painted confirmations 1: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "confirmations 11"),
        "height 799998 painted confirmations 11: {markup}"
    );

    let one = confirmation_markup(
        &tx(Some(r#"{"confirmed":true,"block_height":800000}"#)),
        "800000",
    );
    assert!(
        one.lines().any(|line| line == "confirmations 1"),
        "a count of 1 did not paint confirmations 1: {one}"
    );
    assert_eq!(
        one.lines()
            .filter(|line| *line == "confirmations 1")
            .count(),
        1,
        "confirmations 1 was not painted once: {one}"
    );
    assert!(
        !one.lines().any(|line| line == LINE),
        "a count of 1 painted {LINE}: {one}"
    );
    assert_one(&one, 1);

    let eleven = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "100");
    assert!(
        eleven.lines().any(|line| line == "confirmations 11"),
        "a count of 11 did not paint confirmations 11: {eleven}"
    );
    assert_eq!(
        eleven
            .lines()
            .filter(|line| *line == "confirmations 11")
            .count(),
        1,
        "confirmations 11 was not painted once: {eleven}"
    );
    assert!(
        !eleven.lines().any(|line| line == LINE),
        "a count of 11 painted {LINE}: {eleven}"
    );
    assert_one(&eleven, 11);
}
