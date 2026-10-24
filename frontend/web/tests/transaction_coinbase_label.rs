//! Visible word Coinbase on the transaction page from `GET /tx/:txid`.
//!
//! Angular paints `<span i18n="transactions-list.coinbase">Coinbase</span>`
//! on a coinbase input. The lowercase coinbase span stays the data row, and
//! `(Newly Generated Coins)` stays beside it. A non-coinbase input does not
//! gain the word. The load is `GET /tx/:txid` with no query and no fragment.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const WORD: &str = "Coinbase";
const DATA_SPAN: &str = "<span class=\"coinbase\">coinbase</span>";
const NEWLY_SPAN: &str = "<span class=\"newly-generated-coins\">(Newly Generated Coins)</span>";
const ADJACENT: &str = "<span class=\"coinbase\">coinbase</span><span class=\"newly-generated-coins\">(Newly Generated Coins)</span>";
const SCRIPT_SIG: &str = "COINLABELSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "COINLABELSIGASMDEAD";
const OUT_SCRIPT: &str = "COINLABELOUTHEXDEAD";
const OUT_ASM: &str = "COINLABELOUTASMDEAD";

fn views_source() -> &'static str {
    include_str!("../src/views.rs")
}

/// Body inside the function braces, not the next function.
fn component_body<'a>(src: &'a str, name: &str) -> &'a str {
    let marker = format!("fn {name}(");
    let start = src
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is missing from views.rs"));
    let after_name = &src[start..];
    let brace_rel = after_name
        .find('{')
        .unwrap_or_else(|| panic!("{name} has no body"));
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
    panic!("{name} body is not closed");
}

fn tx(is_coinbase: bool) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","is_coinbase":{is_coinbase},"sequence":4294967295}}],"vout":[{{"scriptpubkey":"{OUT_SCRIPT}","scriptpubkey_asm":"{OUT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn span_texts(markup: &str) -> Vec<String> {
    let mut texts = Vec::new();
    let mut rest = markup;
    while let Some(start) = rest.find("<span") {
        let after = &rest[start..];
        let Some(gt) = after.find('>') else {
            break;
        };
        let Some(end) = after.find("</span>") else {
            break;
        };
        let text = &after[gt + 1..end];
        if !text.contains('<') {
            texts.push(text.to_string());
        }
        rest = &after[end + "</span>".len()..];
    }
    texts
}

fn assert_scripts_stay_off(markup: &str) {
    for needle in [SCRIPT_SIG, SCRIPT_SIG_ASM, OUT_SCRIPT, OUT_ASM] {
        assert!(
            !markup.contains(needle),
            "{needle} leaked into the markup: {markup}"
        );
    }
}

fn assert_no_query_transaction_page() {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    let page = component_body(views_source(), "TransactionPage");
    assert!(
        page.contains("transaction_row_markup("),
        "TransactionPage does not paint the transaction row"
    );
}

#[test]
fn coinbase_input_paints_the_word_coinbase_once() {
    assert_no_query_transaction_page();
    let markup = transaction_row_markup(&tx(true));
    let texts = span_texts(&markup);
    assert_eq!(
        texts.iter().filter(|text| text.as_str() == WORD).count(),
        1,
        "Coinbase is absent: {markup}"
    );
    assert!(
        texts
            .iter()
            .all(|text| !text.contains(WORD) || text.as_str() == WORD),
        "Coinbase was painted inside a longer sentence: {texts:?}"
    );
    assert_eq!(
        markup.matches(WORD).count(),
        2,
        "Coinbase was not once in the span and once on its own line: {markup}"
    );
    assert!(
        markup.contains(DATA_SPAN),
        "the coinbase data row was dropped: {markup}"
    );
    assert!(
        markup.contains(NEWLY_SPAN),
        "(Newly Generated Coins) was dropped: {markup}"
    );
    assert!(
        markup.contains(ADJACENT),
        "the coinbase parenthetical was split from the data row: {markup}"
    );
    assert_scripts_stay_off(&markup);
}

#[test]
fn non_coinbase_input_does_not_paint_the_word_coinbase() {
    assert_no_query_transaction_page();
    let markup = transaction_row_markup(&tx(false));
    assert!(
        !markup.contains(WORD),
        "false coinbase painted Coinbase: {markup}"
    );
    assert!(
        span_texts(&markup).iter().all(|text| text.as_str() != WORD),
        "false coinbase painted Coinbase: {markup}"
    );
    assert!(
        !markup.contains(DATA_SPAN),
        "false coinbase painted the coinbase data row: {markup}"
    );
    assert!(
        !markup.contains(NEWLY_SPAN),
        "false coinbase painted the parenthetical: {markup}"
    );
    assert_scripts_stay_off(&markup);
}

/// Whole line `input coinbase` from no-query `GET /tx/:txid` when `is_coinbase` is true.
///
/// The Coinbase label, the lowercase coinbase span, and `(Newly Generated Coins)` stay.
/// A false coinbase omits the line.
#[test]
fn coinbase_input_paints_input_coinbase() {
    const LINE: &str = "input coinbase";
    assert_no_query_transaction_page();
    let markup = transaction_row_markup(&tx(true));
    assert!(
        markup.lines().any(|line| line == LINE),
        "{LINE} is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert!(
        markup.contains(DATA_SPAN),
        "the coinbase data row was dropped: {markup}"
    );
    assert!(
        markup.contains(NEWLY_SPAN),
        "(Newly Generated Coins) was dropped: {markup}"
    );
    assert!(
        markup.contains(ADJACENT),
        "the coinbase parenthetical was split from the data row: {markup}"
    );
    let ordinary = transaction_row_markup(&tx(false));
    assert!(
        !ordinary.lines().any(|line| line == LINE),
        "false coinbase painted {LINE}: {ordinary}"
    );
    assert_scripts_stay_off(&markup);
}

/// Whole line `Coinbase` from no-query `GET /tx/:txid` when `is_coinbase` is true.
///
/// Angular paints the word Coinbase. GPUI `screen_paints_coinbase` requires
/// one line equal to Coinbase. The label span stays. A false coinbase omits
/// the line. `input coinbase` is the data row, not this label.
#[test]
fn coinbase_input_paints_coinbase_as_its_own_line() {
    const LINE: &str = "Coinbase";
    assert_no_query_transaction_page();
    let markup = transaction_row_markup(&tx(true));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        markup.contains("<span class=\"tx-coinbase-label\">Coinbase</span>"),
        "the Coinbase span was dropped: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line.starts_with("Coinbase ")),
        "{LINE} was painted as a longer sentence: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "input coinbase"),
        "the input coinbase row was dropped: {markup}"
    );
    let label_at = markup.lines().position(|line| line == LINE);
    let data_at = markup.lines().position(|line| line == "input coinbase");
    assert!(
        label_at.is_some() && data_at.is_some() && label_at < data_at,
        "{LINE} was not before input coinbase: {markup}"
    );
    let ordinary = transaction_row_markup(&tx(false));
    assert!(
        !ordinary.contains(LINE),
        "false coinbase painted {LINE}: {ordinary}"
    );
    assert_scripts_stay_off(&markup);
}

/// Whole line `coinbase` from no-query `GET /tx/:txid` when `is_coinbase` is true.
///
/// GPUI `transaction_screen_paints_coinbase_word` requires one line equal to
/// `coinbase`. The lowercase span stays glued to `(Newly Generated Coins)`.
/// `Coinbase` and `input coinbase` stay. A false coinbase omits the line.
#[test]
fn coinbase_input_paints_lowercase_coinbase_as_its_own_line() {
    const LINE: &str = "coinbase";
    assert_no_query_transaction_page();
    let markup = transaction_row_markup(&tx(true));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        markup.contains(ADJACENT),
        "the coinbase parenthetical was split from the data row: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "Coinbase"),
        "Coinbase was dropped: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "input coinbase"),
        "input coinbase was dropped: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line.starts_with("coinbase ")),
        "{LINE} was painted as a longer sentence: {markup}"
    );
    let ordinary = transaction_row_markup(&tx(false));
    assert!(
        !ordinary.lines().any(|line| line == LINE),
        "false coinbase painted {LINE}: {ordinary}"
    );
    assert_scripts_stay_off(&markup);
}
