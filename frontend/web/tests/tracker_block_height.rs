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
        2,
        "expected Block height in the label div and on its own line in {markup}"
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

/// Whole line `Block height` from no-query `GET /tx/:txid`.
///
/// A confirmed status with `block_height` set, including explicit 0, paints
/// one markup line whose entire text is `Block height`. The label div stays.
/// The height number does not become a second caption. A missing height key,
/// JSON null, or a height on an unconfirmed status omits the line.
#[test]
fn confirmed_block_height_paints_block_height_as_its_own_line() {
    const LINE: &str = "Block height";

    let markup = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":90}"#)), "100");
    assert!(
        markup.contains(LABEL),
        "Block height label div is absent: {markup}"
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
    assert!(
        !markup
            .lines()
            .any(|line| line == "Block height 90" || line.starts_with("Block height ")),
        "the height was painted as a second caption: {markup}"
    );

    let zero = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":0}"#)), "100");
    assert!(
        zero.lines().any(|line| line == LINE),
        "explicit zero omitted {LINE}: {zero}"
    );
    assert_eq!(
        zero.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once for zero: {zero}"
    );
    assert!(
        !zero
            .lines()
            .any(|line| line == "Block height 0" || line.starts_with("Block height ")),
        "zero height was painted as a second caption: {zero}"
    );

    let no_tip = confirmation_markup(&tx(Some(r#"{"confirmed":true,"block_height":4}"#)), "");
    assert!(
        no_tip.lines().any(|line| line == LINE),
        "Block height is absent beside Confirmed: {no_tip}"
    );
    assert_eq!(
        no_tip.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once beside Confirmed: {no_tip}"
    );

    let missing = confirmation_markup(&tx(Some(r#"{"confirmed":true}"#)), "100");
    assert!(
        !missing.lines().any(|line| line == LINE),
        "a missing height painted {LINE}: {missing}"
    );
    assert!(
        !missing.contains(LINE),
        "a missing height leaked {LINE}: {missing}"
    );

    let null_height = confirmation_markup(
        &tx(Some(r#"{"confirmed":true,"block_height":null}"#)),
        "100",
    );
    assert!(
        !null_height.lines().any(|line| line == LINE),
        "JSON null painted {LINE}: {null_height}"
    );
    assert!(
        !null_height.contains(LINE),
        "JSON null leaked {LINE}: {null_height}"
    );

    let height_unconfirmed =
        confirmation_markup(&tx(Some(r#"{"confirmed":false,"block_height":4}"#)), "4");
    assert!(
        !height_unconfirmed.lines().any(|line| line == LINE),
        "a height on an unconfirmed status painted {LINE}: {height_unconfirmed}"
    );
    assert!(
        !height_unconfirmed.contains(LINE),
        "a height on an unconfirmed status leaked {LINE}: {height_unconfirmed}"
    );
}
