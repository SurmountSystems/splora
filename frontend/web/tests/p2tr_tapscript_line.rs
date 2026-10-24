//! Whole line `P2TR tapscript` on the transaction input from `GET /tx/:txid`.
//!
//! GPUI requires one line whose entire text is `P2TR tapscript` when
//! `vin.prevout.scriptpubkey_type` is `v1_p2tr`. The span stays. Any other
//! prevout type, a missing prevout, and a P2SH redeem input omit that line.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "P2TR tapscript";
const LABEL_SPAN: &str = "<span class=\"p2tr-tapscript-label\">P2TR tapscript</span>";
const SCRIPT_SIG: &str = "P2TRLINEHEXDEAD";
const SCRIPT_SIG_ASM: &str = "P2TRLINEASMDEAD";
const WITNESS: &str = "P2TRLINEWITDEAD";
const SCRIPT_HEX: &str = "P2TRLINEPUBDEAD";
const SCRIPT_ASM: &str = "P2TRLINEPUBASMDEAD";
const OUT_SCRIPT: &str = "P2TRLINEOUTDEAD";
const OUT_ASM: &str = "P2TRLINEOUTASMDEAD";

fn tx(prevout: &str, out_kind: &str, redeem_field: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1{redeem_field}}}],"vout":[{{"scriptpubkey":"{OUT_SCRIPT}","scriptpubkey_asm":"{OUT_ASM}","scriptpubkey_type":"{out_kind}","value":1}}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn prevout(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"{kind}"}}"#
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
fn v1_p2tr_input_paints_p2tr_tapscript_as_its_own_line() {
    let markup = paint(&tx(&prevout("v1_p2tr"), "p2pkh", ""));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "{LINE} was inside a span: {markup}"
    );
    assert_eq!(markup.matches(LABEL_SPAN).count(), 1, "{markup}");

    let other = paint(&tx(&prevout("v0_p2wsh"), "p2pkh", ""));
    assert!(
        !other.lines().any(|line| line == LINE),
        "a v0_p2wsh prevout painted {LINE}: {other}"
    );

    let redeem = format!(r#","inner_redeemscript_asm":"OP_DUP OP_HASH160""#);
    let p2sh = paint(&tx(&prevout("p2sh"), "p2pkh", &redeem));
    assert!(
        !p2sh.lines().any(|line| line == LINE),
        "a p2sh redeem input painted {LINE}: {p2sh}"
    );

    let missing = paint(&tx("null", "p2pkh", ""));
    assert!(
        !missing.lines().any(|line| line == LINE),
        "a null prevout painted {LINE}: {missing}"
    );

    let output_only = paint(&tx(&prevout("p2pkh"), "v1_p2tr", ""));
    assert!(
        !output_only.lines().any(|line| line == LINE),
        "a v1_p2tr output painted {LINE}: {output_only}"
    );
}
