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
