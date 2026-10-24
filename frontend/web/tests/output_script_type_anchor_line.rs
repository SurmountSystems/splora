//! Whole line `output script type anchor` on a transaction output from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_anchor_output_type` in
//! `frontend/native/tests/rest_of_indexer.rs` paints one line whose entire
//! text is `output script type anchor` when `vout.scriptpubkey_type` is
//! `anchor`. The count assert is at lines 5824 through 5830. The fixture is
//! `paint_anchor_tx` at line 5676 with `anchor_vout` at line 5663. An empty
//! type omits any `output script type` line at lines 5887 through 5889. A
//! missing key and JSON null omit the word `anchor` at lines 5909 through
//! 5911. GPUI `render_address_txs` at `frontend/native/src/screens.rs:4497`
//! calls `render_tx_parties`, which calls `push_script_type` at line 3475,
//! so an address transaction with an anchor output does not omit this line.
//! This test does not assert an address omit. A non-anchor output omits this
//! exact line: `push_script_type` at lines 3491 through 3495 pushes the type
//! that is present. The scriptpubkey-type span and the `Type ANCHOR` line stay.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "atypeone";
const LINE: &str = "output script type anchor";
const ANC_HEX: &str = "ancpubhexdead";
const ANC_ASM: &str = "ancasmdead";
const ANC_SIG: &str = "ancsighexdead";
const ANC_SIG_ASM: &str = "ancsigasmdead";
const ANC_WIT: &str = "ancwitdead";
const ANC_INNER: &str = "ancinnerdead";
const ANC_LEAF: &str = "ancleafdead";

fn anchor_vin() -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"{ANC_HEX}","scriptpubkey_asm":"{ANC_ASM}","scriptpubkey_type":"v0_p2wpkh","value":5}},"scriptsig":"{ANC_SIG}","scriptsig_asm":"{ANC_SIG_ASM}","witness":["{ANC_WIT}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{ANC_INNER}","leaf_version":"{ANC_LEAF}"}}"#
    )
}

fn anchor_vout(type_field: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{ANC_HEX}","scriptpubkey_asm":"{ANC_ASM}",{type_field}"value":1}}"#
    )
}

fn anchor_tx(txid: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4}}}}"#,
        anchor_vin()
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
fn anchor_output_paints_output_script_type_anchor_as_its_own_line() {
    let markup = paint(&anchor_tx(
        TXID,
        &anchor_vout(r#""scriptpubkey_type":"anchor","#),
    ));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals output script type anchor: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "no line equals output script type anchor: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">anchor</span>"),
        "the scriptpubkey-type span was dropped: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == "Type ANCHOR").count(),
        1,
        "Type ANCHOR was dropped: {markup}"
    );

    let ordinary = paint(&anchor_tx(
        "atypewpkh",
        &anchor_vout(r#""scriptpubkey_type":"v0_p2wpkh","#),
    ));
    assert_eq!(
        ordinary.lines().filter(|line| *line == LINE).count(),
        0,
        "a non-anchor output painted {LINE}: {ordinary}"
    );

    let empty = paint(&anchor_tx(
        "atypeempty",
        &anchor_vout(r#""scriptpubkey_type":"","#),
    ));
    assert_eq!(
        empty.lines().filter(|line| *line == LINE).count(),
        0,
        "an empty scriptpubkey_type painted {LINE}: {empty}"
    );
    assert!(
        !empty.contains("output script type"),
        "an empty type painted a script type line: {empty}"
    );

    let missing = paint(&anchor_tx("atypemiss", &anchor_vout("")));
    assert_eq!(
        missing.lines().filter(|line| *line == LINE).count(),
        0,
        "a missing scriptpubkey_type painted {LINE}: {missing}"
    );

    let null_type = paint(&anchor_tx(
        "atypenull",
        &anchor_vout(r#""scriptpubkey_type":null,"#),
    ));
    assert_eq!(
        null_type.lines().filter(|line| *line == LINE).count(),
        0,
        "JSON null scriptpubkey_type painted {LINE}: {null_type}"
    );
}
