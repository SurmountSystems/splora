//! Whole line `Inputs & Outputs` on the transaction from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_v0_p2wsh_output_type` in
//! `frontend/native/tests/rest_of_indexer.rs` requires one line whose entire
//! text is `Inputs & Outputs`. The count assert is at lines 6048 through
//! 6055, from `paint_v0_p2wsh_tx` with txid `wshone` and a `v0_p2wsh` vout.
//! The ampersand is a real `&`, not the HTML escape. The h2 stays. An empty
//! body omits the h2 and omits this line.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "wshone";
const LINE: &str = "Inputs & Outputs";
const HEADING: &str = "<h2>Inputs & Outputs</h2>";
const WSH_HEX: &str = "wshpubhexdead";
const WSH_ASM: &str = "wshasmdead";
const WSH_SIG: &str = "wshsighexdead";
const WSH_SIG_ASM: &str = "wshsigasmdead";
const WSH_WIT: &str = "wshwitdead";
const WSH_INNER: &str = "wshinnerdead";
const WSH_LEAF: &str = "wshleafdead";

fn v0_p2wsh_vin() -> String {
    format!(
        r#"{{"txid":"prev","vout":0,"prevout":{{"scriptpubkey":"{WSH_HEX}","scriptpubkey_asm":"{WSH_ASM}","scriptpubkey_type":"v0_p2wpkh","value":5}},"scriptsig":"{WSH_SIG}","scriptsig_asm":"{WSH_SIG_ASM}","witness":["{WSH_WIT}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{WSH_INNER}","leaf_version":"{WSH_LEAF}"}}"#
    )
}

fn v0_p2wsh_vout(type_field: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{WSH_HEX}","scriptpubkey_asm":"{WSH_ASM}",{type_field}"value":1}}"#
    )
}

fn v0_p2wsh_tx(txid: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4}}}}"#,
        v0_p2wsh_vin()
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

fn line_count(markup: &str) -> usize {
    markup.lines().filter(|line| *line == LINE).count()
}

#[test]
fn v0_p2wsh_tx_paints_inputs_and_outputs_as_its_own_line() {
    let markup = paint(&v0_p2wsh_tx(
        TXID,
        &v0_p2wsh_vout(r#""scriptpubkey_type":"v0_p2wsh","#),
    ));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals Inputs & Outputs: {markup}"
    );
    assert_eq!(
        line_count(&markup),
        1,
        "no line equals Inputs & Outputs: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Inputs &amp; Outputs"),
        "the whole line used an HTML escape: {markup}"
    );
    assert_eq!(
        markup.matches(HEADING).count(),
        1,
        "the h2 was dropped: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<') && line.contains('&')),
        "Inputs & Outputs was inside a tag or used an escape: {markup}"
    );

    let empty = transaction_row_markup("");
    assert!(
        !empty.contains(HEADING),
        "an empty body painted the h2: {empty}"
    );
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty body painted Inputs & Outputs: {empty}"
    );
    assert_eq!(
        line_count(&empty),
        0,
        "an empty body painted Inputs & Outputs: {empty}"
    );
}
