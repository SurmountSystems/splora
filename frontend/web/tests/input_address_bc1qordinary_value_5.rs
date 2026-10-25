//! Whole line `input address bc1qordinary value 5` on an ordinary prevout.
//!
//! GPUI `transaction_screen_paints_issuance_label` already requires this
//! exact line. It is not `output address bc1qordinary value 9`. One space
//! sits between the address and the word value. The address anchor and the
//! value span stay. This calls `transaction_row_markup`.

use splora_web::{transaction_requests, transaction_row_markup};

const LINE: &str = "input address bc1qordinary value 5";

fn assert_one_whole_line(markup: &str, line: &str) {
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {line}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|row| *row == line).count(),
        1,
        "{line} was dropped or repeated: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "the sentence was inside a tag: {line}: {markup}"
    );
}

fn ordinary_vin() -> String {
    r#"{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":5},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}"#.to_string()
}

fn tx(txid: &str) -> String {
    let vin = ordinary_vin();
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{{"scriptpubkey":"6a","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":1}}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
    )
}

fn paint(txid: &str) -> String {
    let requests = transaction_requests(txid);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{txid}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(&tx(txid))
}

#[test]
fn input_address_bc1qordinary_value_5() {
    let markup = paint("ordinary5");
    assert_one_whole_line(&markup, LINE);
    assert!(
        markup.contains("<a class=\"address\" href=\"/address/bc1qordinary\">bc1qordinary</a>"),
        "the address anchor was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"value\">5</span>"),
        "the value span was dropped: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "input address  bc1qordinary value 5"),
        "the prevout line used two spaces before the address: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "output address bc1qordinary value 9"),
        "the input line was painted as the output sentence: {markup}"
    );
}
