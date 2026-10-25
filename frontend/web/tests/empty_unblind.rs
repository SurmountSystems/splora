//! Confidential address lines on a Liquid transaction from `GET /tx/:txid`.
//!
//! The GPUI screen paints the exact line
//! `input address lq1qqvin value Confidential` when the input prevout
//! omits `value` and still has a commitment. The same body paints
//! `output address lq1qqout1 value Confidential` for the first output.
//! The same body paints
//! `output address lq1qqout2 value Confidential` for the second output.
//! A published value and a missing value with no commitment omit those
//! lines. The commitment hex stays off the page. The path has no query string.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "f2f41c0850e8e7e3f1af233161fd596662e67c11ef10ed15943884186fbb7f46";
const LINE: &str = "input address lq1qqvin value Confidential";

fn confidential_output(address: &str, commitment: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{address}","valuecommitment":"{commitment}","assetcommitment":"{commitment}aa"}}"#
    )
}

fn clear_output(address: &str, value: u64) -> String {
    format!(
        r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{address}","value":{value}}}"#
    )
}

fn blank_output(address: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"{address}"}}"#
    )
}

fn input_with(prevout: &str) -> String {
    format!(
        r#"{{"txid":"prevouttx","vout":0,"prevout":{prevout},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295}}"#
    )
}

fn tx_body(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
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

#[test]
fn empty_unblinded_tx_paints_input_address_confidential_line() {
    let vin = input_with(&confidential_output("lq1qqvin", "aa11"));
    let vout = format!(
        "{},{}",
        confidential_output("lq1qqout1", "bb22"),
        confidential_output("lq1qqout2", "cc33")
    );
    let body = tx_body(&vin, &vout);
    assert!(
        !body.contains("\"value\""),
        "the fixture must omit value: {body}"
    );
    let markup = paint(&body);
    assert_eq!(markup.matches(LINE).count(), 1, "{markup}");
    assert!(!markup.contains("aa11"), "{markup}");
    assert!(!markup.contains("bb22"), "{markup}");
    assert!(!markup.contains("cc33"), "{markup}");

    let published = paint(&tx_body(
        &input_with(&clear_output("lq1qqvin", 10)),
        &clear_output("lq1qqout1", 9),
    ));
    assert!(!published.contains(LINE), "{published}");
    assert!(!published.contains("Confidential"), "{published}");

    let blank = paint(&tx_body(
        &input_with(&blank_output("lq1qqvin")),
        &blank_output("lq1qqout1"),
    ));
    assert!(!blank.contains(LINE), "{blank}");
    assert!(!blank.contains("Confidential"), "{blank}");
}

const OUTPUT_LINE: &str = "output address lq1qqout1 value Confidential";

#[test]
fn empty_unblinded_tx_paints_output_address_lq1qqout1_confidential_line() {
    let vin = input_with(&confidential_output("lq1qqvin", "aa11"));
    let vout = format!(
        "{},{}",
        confidential_output("lq1qqout1", "bb22"),
        confidential_output("lq1qqout2", "cc33")
    );
    let body = tx_body(&vin, &vout);
    assert!(
        !body.contains("\"value\""),
        "the fixture must omit value: {body}"
    );
    let markup = paint(&body);
    assert_eq!(markup.matches(OUTPUT_LINE).count(), 1, "{markup}");
    assert!(!markup.contains("bb22"), "{markup}");
    assert!(!markup.contains("cc33"), "{markup}");

    let published = paint(&tx_body(
        &input_with(&clear_output("lq1qqvin", 10)),
        &clear_output("lq1qqout1", 9),
    ));
    assert!(!published.contains(OUTPUT_LINE), "{published}");
    assert!(!published.contains("Confidential"), "{published}");

    let blank = paint(&tx_body(
        &input_with(&blank_output("lq1qqvin")),
        &blank_output("lq1qqout1"),
    ));
    assert!(!blank.contains(OUTPUT_LINE), "{blank}");
    assert!(!blank.contains("Confidential"), "{blank}");
}

const OUTPUT2_LINE: &str = "output address lq1qqout2 value Confidential";

#[test]
fn empty_unblinded_tx_paints_output_address_lq1qqout2_confidential_line() {
    let vin = input_with(&confidential_output("lq1qqvin", "aa11"));
    let vout = format!(
        "{},{}",
        confidential_output("lq1qqout1", "bb22"),
        confidential_output("lq1qqout2", "cc33")
    );
    let body = tx_body(&vin, &vout);
    assert!(
        !body.contains("\"value\""),
        "the fixture must omit value: {body}"
    );
    let markup = paint(&body);
    assert_eq!(markup.matches(OUTPUT2_LINE).count(), 1, "{markup}");
    assert!(!markup.contains("bb22"), "{markup}");
    assert!(!markup.contains("cc33"), "{markup}");

    let published = paint(&tx_body(
        &input_with(&clear_output("lq1qqvin", 10)),
        &clear_output("lq1qqout2", 9),
    ));
    assert!(!published.contains(OUTPUT2_LINE), "{published}");
    assert!(!published.contains("Confidential"), "{published}");

    let blank = paint(&tx_body(
        &input_with(&blank_output("lq1qqvin")),
        &blank_output("lq1qqout2"),
    ));
    assert!(!blank.contains(OUTPUT2_LINE), "{blank}");
    assert!(!blank.contains("Confidential"), "{blank}");
}

/// A missing value with no commitment stays blank. The address is the
/// published address. The amount after `value` stays empty.
#[test]
fn missing_value_without_commitment_stays_blank() {
    let markup = paint(&tx_body(
        &input_with(&blank_output("lq1qqvin")),
        &blank_output("lq1qqout1"),
    ));
    assert!(
        markup
            .lines()
            .any(|line| line == "input address lq1qqvin value "),
        "{markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == "output address lq1qqout1 value "),
        "{markup}"
    );
}

/// A published value stays that number. One space sits between the address
/// and the word value. The amount is 9. It is not Confidential.
#[test]
fn explicit_value_is_not_confidential() {
    let markup = paint(&tx_body(
        &input_with(&clear_output("lq1qqvin", 10)),
        &clear_output("lq1qqout1", 9),
    ));
    assert!(
        markup
            .lines()
            .any(|line| line == "output address lq1qqout1 value 9"),
        "{markup}"
    );
}
