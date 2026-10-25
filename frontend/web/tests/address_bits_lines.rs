//! Whole lines GPUI already requires from the transaction and block painters.
//!
//! `output address bc1qpeg value 1` is an addressed peg-out output.
//! `input address bc1qwithprev value 8` is an issuance input that still
//! has a prevout. `bits 2 0x00000002` is open block details when bits is 2.
//! One space sits between a present address and the word value. A two-space
//! blank-address line is a different sentence. The address anchor, the value
//! span, and the Bits cells stay.

use splora_web::{
    block_details_markup, block_load_plan, transaction_requests, transaction_row_markup,
};

const PEG_LINE: &str = "output address bc1qpeg value 1";
const PREV_LINE: &str = "input address bc1qwithprev value 8";
const BITS_LINE: &str = "bits 2 0x00000002";
const PEGOUT: &str = r#"{"genesis_hash":"aa","scriptpubkey":"bb","scriptpubkey_asm":"OP_TRUE"}"#;
const ASSET: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";
const ENTROPY: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

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

fn paint_tx(txid: &str, body: &str) -> String {
    let requests = transaction_requests(txid);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{txid}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(body)
}

fn peg_vin() -> String {
    r#"{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qin","value":8},"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}"#.to_string()
}

fn peg_tx(txid: &str) -> String {
    let vin = peg_vin();
    let vout = format!(
        r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","value":1,"scriptpubkey_address":"bc1qpeg","pegout":{PEGOUT}}}"#
    );
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
    )
}

fn issuance_vin() -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qwithprev","value":8}},"issuance":{{"asset_id":"{ASSET}","is_reissuance":false,"asset_entropy":"{ENTROPY}"}},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}}"#
    )
}

fn prev_tx(txid: &str) -> String {
    let vin = issuance_vin();
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_RETURN","scriptpubkey_type":"op_return","value":9}}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
    )
}

fn bits_block(id: &str) -> String {
    format!(
        r#"{{"id":"{id}","height":2,"version":536870912,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","mediantime":1699990000,"bits":2,"nonce":9,"difficulty":1}}"#
    )
}

#[test]
fn output_address_bc1qpeg_value_1() {
    let txid = "pegoutaddr";
    let markup = paint_tx(txid, &peg_tx(txid));
    assert_one_whole_line(&markup, PEG_LINE);
    assert!(
        markup.contains("<a class=\"address\" href=\"/address/bc1qpeg\">bc1qpeg</a>"),
        "the address anchor was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"value\">1</span>"),
        "the value span was dropped: {markup}"
    );
    assert!(
        !markup.lines().any(|row| row == "output address  value 1"),
        "the addressed peg-out painted the two-space blank line: {markup}"
    );
    assert!(
        !markup.lines().any(|row| row == "bc1qpeg"),
        "bc1qpeg was its own line: {markup}"
    );
}

#[test]
fn input_address_bc1qwithprev_value_8() {
    let txid = "issueprev";
    let markup = paint_tx(txid, &prev_tx(txid));
    assert_one_whole_line(&markup, PREV_LINE);
    assert!(
        markup.contains("<a class=\"address\" href=\"/address/bc1qwithprev\">bc1qwithprev</a>"),
        "the address anchor was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"value\">8</span>"),
        "the value span was dropped: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|row| row == "input address  bc1qwithprev value 8"),
        "the prevout line used two spaces before the address: {markup}"
    );
    assert!(
        !markup.lines().any(|row| row == "bc1qwithprev"),
        "bc1qwithprev was its own line: {markup}"
    );
}

#[test]
fn bits_2_0x00000002() {
    let id = "verhex536";
    assert!(!id.bytes().all(|byte| byte.is_ascii_digit()), "{id}");
    assert!(!id.contains('?') && !id.contains('#'), "{id}");
    let plan = block_load_plan(id, None);
    assert!(
        plan.iter()
            .any(|req| req.method == "GET" && req.path == format!("/block/{id}")),
        "{plan:?}"
    );
    assert!(
        plan.iter()
            .all(|req| !req.path.contains('?') && !req.path.contains('#')),
        "{plan:?}"
    );

    let body = bits_block(id);
    let open = block_details_markup(&body, true);
    assert_one_whole_line(&open, BITS_LINE);
    assert!(
        open.contains("<tr><td>Bits</td><td>2</td><td class=\"bits-hex\">0x00000002</td></tr>"),
        "the Bits cells were dropped: {open}"
    );
    assert!(
        !open
            .lines()
            .any(|row| row == "bits 2" || row == "0x00000002"),
        "bits 2 or 0x00000002 was its own line: {open}"
    );

    let closed = block_details_markup(&body, false);
    assert!(
        !closed.lines().any(|row| row == BITS_LINE),
        "closed details painted {BITS_LINE}: {closed}"
    );
    assert!(
        closed.contains("<tr><td>Bits</td><td>2</td><td class=\"bits-hex\">0x00000002</td></tr>"),
        "closed details dropped the Bits cells: {closed}"
    );
}
