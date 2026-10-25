//! Three GPUI whole lines that Leptos painters did not emit.
//!
//! `1 of 12` is `asset_markup` on `GET /asset/:id` when one transaction is
//! loaded and chain `tx_count` plus mempool `tx_count` is 12. The h2 stays.
//! `Type V0_P2WPKH` is `output_party_at` through `transaction_row_markup`
//! when the output script type is `v0_p2wpkh`. The Type spans stay.
//! `input address lq1qqvin value 10` is `input_party` through
//! `transaction_row_markup` when that prevout publishes value 10. One space
//! sits between the address and the word value. The value span stays.

use splora_web::{asset_markup, transaction_requests, transaction_row_markup};

const ONE_OF_TWELVE: &str = "1 of 12";
const TYPE_V0_P2WPKH: &str = "Type V0_P2WPKH";
const INPUT_LQ1QQVIN: &str = "input address lq1qqvin value 10";

fn assert_one_whole_line(markup: &str, line: &str) {
    assert!(
        markup.lines().any(|row| row == line),
        "no line equals {line}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|row| *row == line).count(),
        1,
        "{line} was dropped or repeated: {markup}"
    );
    assert!(
        markup.lines().any(|row| row == line && !row.contains('<')),
        "the sentence was inside a tag: {line}: {markup}"
    );
}

fn issued_asset_with_twelve() -> String {
    r#"{"asset_id":"ac3e0ff248c5051ffd61e00155b7122e5ebc04fd397a0ecbdd4f4e4a56232926","name":"Test Asset","ticker":"TEST","precision":2,"chain_stats":{"tx_count":3,"issuance_count":1,"issued_amount":5000,"burned_amount":200},"mempool_stats":{"tx_count":9,"issuance_count":4,"issued_amount":999999,"burned_amount":888888}}"#
        .to_string()
}

fn one_asset_tx() -> String {
    r#"[{"txid":"assetchaintx","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{"confirmed":true}}]"#
        .to_string()
}

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"nextthree","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4}}}}"#
    )
}

fn empty_input() -> String {
    r#"{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}"#
        .to_string()
}

fn v0_p2wpkh_output() -> String {
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#
        .to_string()
}

fn clear_output(address: &str, value: u64) -> String {
    format!(
        r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{address}","value":{value}}}"#
    )
}

fn input_with(prevout: &str) -> String {
    format!(
        r#"{{"txid":"prevouttx","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295}}"#
    )
}

fn paint_tx(body: &str) -> String {
    let requests = transaction_requests("nextthree");
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/tx/nextthree");
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(body)
}

#[test]
fn next_three_lines_one_of_twelve() {
    let markup = asset_markup(&issued_asset_with_twelve(), &one_asset_tx(), true);
    assert_one_whole_line(&markup, ONE_OF_TWELVE);
    assert!(
        markup.contains("<h2>1 of 12&nbsp;Issuance and Burn Transactions</h2>"),
        "the asset h2 was dropped: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|row| row == ONE_OF_TWELVE && !row.contains('\u{00a0}')),
        "the whole line painted a nbsp: {markup}"
    );
}

#[test]
fn next_three_lines_type_v0_p2wpkh() {
    let markup = paint_tx(&tx(&empty_input(), &v0_p2wpkh_output()));
    assert_one_whole_line(&markup, TYPE_V0_P2WPKH);
    assert!(
        markup.contains("<span class=\"output-type-label\">Type</span>"),
        "the Type label span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"output-type\">V0_P2WPKH</span>"),
        "the uppercase type span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">v0_p2wpkh</span>"),
        "the scriptpubkey-type span was dropped: {markup}"
    );
    assert!(
        markup.lines().any(|row| row == "Type"),
        "the Type word line was dropped: {markup}"
    );
    for bare in [
        "V0_P2WPKH",
        "V0_P2WSH",
        "v0_p2wsh",
        "MULTISIG",
        "multisig",
        "EMPTY",
    ] {
        assert!(
            !markup.lines().any(|row| row == bare),
            "a bare token was painted: {bare}: {markup}"
        );
    }
}

#[test]
fn next_three_lines_input_address_lq1qqvin_value_10() {
    let body = tx(
        &input_with(&clear_output("lq1qqvin", 10)),
        &clear_output("lq1qqout1", 9),
    );
    let markup = paint_tx(&body);
    assert_one_whole_line(&markup, INPUT_LQ1QQVIN);
    assert!(
        markup.contains("<span class=\"value\">10</span>"),
        "the value span was dropped: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|row| row == "output address lq1qqout1 value 9"),
        "the output address line was dropped: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|row| row == "input address  lq1qqvin value 10"),
        "the input line used two spaces before the address: {markup}"
    );
}
