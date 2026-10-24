//! Spender confirmation on an outspend the transaction page already loads.
//!
//! `SpendingValue.status.confirmed` prints when the status object is present,
//! including false. A missing status key or JSON null omits the label. It is
//! not false. Block height, block hash, and block time on that status stay off.

use splora_web::{
    batch_outspends_markup, one_outspend_markup, outspends_markup, transaction_detail_requests,
};

const TX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SPEND: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const BLOCK: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn tx_link(txid: &str) -> String {
    format!("<a href=\"/tx/{txid}\">{txid}</a>")
}

fn confirmed_span(word: &str) -> String {
    format!("<span class=\"spend-confirmed\">confirmed {word}</span>")
}

#[test]
fn outspend_confirmed_prints_and_a_missing_status_omits_it() {
    let calls = transaction_detail_requests(TX);
    assert!(
        calls
            .iter()
            .any(|call| call.path == format!("/tx/{TX}/outspends")),
        "the transaction page does not request outspends: {calls:?}"
    );
    let src = include_str!("../src/views.rs");
    assert!(
        src.contains("outspends_markup"),
        "TransactionPage does not paint outspends"
    );
    assert!(
        src.contains("one_outspend_markup"),
        "TransactionPage does not paint one outspend"
    );
    assert!(
        src.contains("batch_outspends_markup"),
        "BlockPage does not paint batch outspends"
    );

    let spent = format!(
        r#"[{{"spent":true,"txid":"{SPEND}","vin":1,"status":{{"confirmed":true,"block_height":842001,"block_hash":"{BLOCK}","block_time":1600000001}}}}]"#
    );
    let html = outspends_markup(&spent);
    assert!(
        html.contains(&confirmed_span("true")),
        "confirmed true is missing: {html}"
    );
    assert!(html.contains("spent true"), "{html}");
    assert!(html.contains(&tx_link(SPEND)), "{html}");
    assert!(html.contains("vin 1"), "{html}");
    assert!(
        !html.contains(BLOCK),
        "block hash stays off the spend: {html}"
    );
    assert!(
        !html.contains("1600000001"),
        "block time stays off the spend: {html}"
    );
    assert!(
        !html.contains("842001"),
        "block height stays off the spend: {html}"
    );

    let batch =
        format!(r#"[[{{"spent":true,"txid":"{SPEND}","vin":3,"status":{{"confirmed":true}}}}]]"#);
    let batch_html = batch_outspends_markup(&batch);
    assert!(
        batch_html.contains(&confirmed_span("true")),
        "batch spend omitted confirmed true: {batch_html}"
    );

    let unconfirmed =
        format!(r#"{{"spent":true,"txid":"{SPEND}","vin":0,"status":{{"confirmed":false}}}}"#);
    let one = one_outspend_markup(&unconfirmed);
    assert!(
        one.contains(&confirmed_span("false")),
        "confirmed false was omitted: {one}"
    );
    assert!(one.contains("vin 0"), "{one}");

    let bare = outspends_markup(r#"[{"spent":false}]"#);
    assert!(bare.contains("spent false"), "{bare}");
    assert!(
        !bare.contains("confirmed"),
        "a missing status painted confirmed: {bare}"
    );
    assert!(!bare.contains("spend-confirmed"), "{bare}");

    let null_status = format!(r#"{{"spent":true,"txid":"{SPEND}","vin":2,"status":null}}"#);
    let null_html = one_outspend_markup(&null_status);
    assert!(null_html.contains("spent true"), "{null_html}");
    assert!(null_html.contains("vin 2"), "{null_html}");
    assert!(
        !null_html.contains("confirmed"),
        "JSON null status painted confirmed: {null_html}"
    );
}

/// Whole line `spent true` from no-query `GET /tx/:txid/outspends`.
///
/// GPUI `transaction_screen_paints_spent_true` requires one line equal to
/// `spent true` when `spent` is true. The paragraph text and the spending
/// txid stay. An unspent output stays `spent false` and omits this line.
#[test]
fn spent_output_paints_spent_true_as_its_own_line() {
    const LINE: &str = "spent true";
    let spent =
        format!(r#"[{{"spent":true,"txid":"{SPEND}","vin":1,"status":{{"confirmed":true}}}}]"#);
    let html = outspends_markup(&spent);
    assert!(
        html.lines().any(|line| line == LINE),
        "no line equals {LINE}: {html}"
    );
    assert_eq!(
        html.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {html}"
    );
    assert!(
        html.contains(&tx_link(SPEND)),
        "the spending txid was dropped: {html}"
    );
    assert!(html.contains("vin 1"), "vin was dropped: {html}");
    assert!(
        !html.lines().any(|line| line.starts_with("spent true ")),
        "{LINE} was painted as a longer sentence: {html}"
    );

    let unspent = outspends_markup(r#"[{"spent":false}]"#);
    assert!(
        unspent.lines().any(|line| line == "spent false"),
        "spent false is absent: {unspent}"
    );
    assert!(
        !unspent.lines().any(|line| line == LINE),
        "an unspent output painted {LINE}: {unspent}"
    );
}

/// Whole line `vin 1` from no-query `GET /tx/:txid/outspends`.
///
/// GPUI `transaction_screen_paints_vin` requires one line equal to `vin 1`
/// when the spend includes `vin`. The paragraph keeps its existing index.
/// A spend with no `vin` does not invent `vin 0`. `spent true` stays.
#[test]
fn spent_output_paints_vin_as_its_own_line() {
    const LINE: &str = "vin 1";
    let spent =
        format!(r#"[{{"spent":true,"txid":"{SPEND}","vin":1,"status":{{"confirmed":true}}}}]"#);
    let html = outspends_markup(&spent);
    assert!(
        html.lines().any(|line| line == LINE),
        "no line equals {LINE}: {html}"
    );
    assert_eq!(
        html.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {html}"
    );
    assert!(
        html.lines().any(|line| line == "spent true"),
        "spent true was dropped: {html}"
    );
    assert!(
        html.contains(&tx_link(SPEND)),
        "the spending txid was dropped: {html}"
    );
    assert!(
        !html.lines().any(|line| line.starts_with("vin 1 ")),
        "{LINE} was painted as a longer sentence: {html}"
    );

    let missing = outspends_markup(r#"[{"spent":false}]"#);
    assert!(
        !missing.lines().any(|line| line.starts_with("vin ")),
        "a spend with no vin invented a vin line: {missing}"
    );
    assert!(
        missing.lines().any(|line| line == "spent false"),
        "spent false was dropped: {missing}"
    );
}

/// Whole line `confirmed true` from no-query `GET /tx/:txid/outspends`.
///
/// GPUI `transaction_screen_paints_outspend_confirmed` requires one line
/// equal to `confirmed true` when the spend status is confirmed. The span
/// stays. A missing status omits the line. Block height, block hash, and
/// block time stay off. `spent true` and `vin 1` stay.
#[test]
fn spent_output_paints_confirmed_true_as_its_own_line() {
    const LINE: &str = "confirmed true";
    let spent = format!(
        r#"[{{"spent":true,"txid":"{SPEND}","vin":1,"status":{{"confirmed":true,"block_height":842001,"block_hash":"{BLOCK}","block_time":1600000001}}}}]"#
    );
    let html = outspends_markup(&spent);
    assert!(
        html.lines().any(|line| line == LINE),
        "no line equals {LINE}: {html}"
    );
    assert_eq!(
        html.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {html}"
    );
    assert!(
        html.contains(&confirmed_span("true")),
        "the confirmed span was dropped: {html}"
    );
    assert!(
        html.lines().any(|line| line == "spent true"),
        "spent true was dropped: {html}"
    );
    assert!(
        html.lines().any(|line| line == "vin 1"),
        "vin 1 was dropped: {html}"
    );
    assert!(!html.contains(BLOCK), "block hash leaked: {html}");
    assert!(!html.contains("842001"), "block height leaked: {html}");
    assert!(!html.contains("1600000001"), "block time leaked: {html}");
    assert!(
        !html.lines().any(|line| line.starts_with("confirmed true ")),
        "{LINE} was painted as a longer sentence: {html}"
    );

    let bare = outspends_markup(r#"[{"spent":false}]"#);
    assert!(
        !bare.lines().any(|line| line == LINE),
        "a missing status painted {LINE}: {bare}"
    );
}
