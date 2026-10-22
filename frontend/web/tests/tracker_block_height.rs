//! Block height caption on the transaction page from `tracker.component.html`.
//!
//! The element body is `Block height`. It is shown when the transaction is
//! confirmed and `status.block_height` is set. An unconfirmed transaction,
//! or a confirmed status with no height, omits it. The outspend status does
//! not gain this caption. The route is `/tx/:txid` with no query string.

use splora_web::{confirmation_markup, outspends_markup};

const LABEL: &str = "<div class=\"label\">Block height</div>";
const SPEND: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

fn tx(status: Option<&str>) -> String {
    let status_field = match status {
        Some(status) => format!(r#","status":{status}"#),
        None => String::new(),
    };
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":4,"sigops":0,"fee":1{status_field}}}"#
    )
}

#[test]
fn confirmed_transaction_paints_block_height_label() {
    let markup = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "100");
    assert!(markup.contains(LABEL), "Block height is absent: {markup}");
    assert_eq!(
        markup.matches("Block height").count(),
        1,
        "expected Block height once in {markup}"
    );
    assert!(
        !markup.contains("title=\"Block height\""),
        "Block height is a tooltip: {markup}"
    );

    let unconfirmed = confirmation_markup(&tx(Some(r#"{"confirmed":false}"#)), "100");
    assert!(
        !unconfirmed.contains("Block height"),
        "unconfirmed painted Block height: {unconfirmed}"
    );

    let confirmed_without_height = confirmation_markup(&tx(Some(r#"{"confirmed":true}"#)), "100");
    assert!(
        !confirmed_without_height.contains("Block height"),
        "a confirmed status with no height painted Block height: {confirmed_without_height}"
    );

    let omitted = confirmation_markup(&tx(None), "100");
    assert!(
        !omitted.contains("Block height"),
        "a missing status painted Block height: {omitted}"
    );

    let spend = format!(
        r#"[{{"spent":true,"txid":"{SPEND}","vin":1,"status":{{"confirmed":true,"block_height":842001}}}}]"#
    );
    let spend_html = outspends_markup(&spend);
    assert!(
        !spend_html.contains("Block height"),
        "outspend painted Block height: {spend_html}"
    );

    let src = include_str!("../src/views.rs");
    assert!(
        src.contains("path!(\"tx/:txid\")"),
        "transaction route is missing: {src}"
    );
    assert!(
        !src.contains("tx/:txid?"),
        "transaction route has a query string"
    );
    let start = src
        .find("fn TransactionPage(")
        .expect("TransactionPage is missing");
    let page = &src[start..];
    assert!(
        page.contains("confirmation_markup"),
        "TransactionPage does not paint confirmation markup"
    );
}
