//! Whole lines GPUI already requires.
//!
//! `transaction_screen_paints_p2pk_label` at
//! `frontend/native/tests/rest_of_indexer.rs:8713` requires
//! `output script type p2pkh`.
//! `block_screen_paints_nonce_hex` at
//! `frontend/native/tests/rest_of_indexer.rs:10389` requires
//! `nonce 0 0x00000000`.
//! `transaction_screen_paints_p2pk_label` at
//! `frontend/native/tests/rest_of_indexer.rs:8601` requires
//! `output address bc1qp2pk value 3` with one space before `value`.

use splora_web::{
    block_details_markup, block_load_plan, transaction_requests, transaction_row_markup,
};

const P2PKH_LINE: &str = "output script type p2pkh";
const NONCE_LINE: &str = "nonce 0 0x00000000";
const ADDRESS_LINE: &str = "output address bc1qp2pk value 3";

fn ordinary_vin() -> String {
    r#"{"txid":"prev","vout":0,"prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qin","value":8},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}"#.to_string()
}

fn tx(txid: &str, vout: &str) -> String {
    let vin = ordinary_vin();
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
    )
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

fn assert_whole_line(markup: &str, line: &str) {
    assert!(
        markup.lines().any(|row| row == line),
        "no line equals {line}: {markup}"
    );
}

#[test]
fn p2pkh_output_paints_output_script_type_p2pkh() {
    let vout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pkh","value":6}"#;
    let markup = paint_tx("p2pkother", &tx("p2pkother", vout));
    assert_whole_line(&markup, P2PKH_LINE);
}

#[test]
fn open_block_details_paint_nonce_0_hex() {
    let id = "noncehex0";
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
    let body = format!(
        r#"{{"id":"{id}","height":2,"version":1,"timestamp":1700000000,"tx_count":1,"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":0,"bits":386547056,"difficulty":1}}"#
    );
    let open = block_details_markup(&body, true);
    assert_whole_line(&open, NONCE_LINE);
}

#[test]
fn addressed_p2pk_output_paints_output_address_bc1qp2pk_value_3() {
    let vout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","scriptpubkey_address":"bc1qp2pk","value":3}"#;
    let markup = paint_tx("p2pkaddr", &tx("p2pkaddr", vout));
    assert_ne!(ADDRESS_LINE, "output address bc1qp2pk  value 3");
    assert_whole_line(&markup, ADDRESS_LINE);
}
