//! Whole line `P2TR tapscript` on the transaction input from `GET /tx/:txid`.
//!
//! GPUI requires one line whose entire text is `P2TR tapscript` when
//! `vin.prevout.scriptpubkey_type` is `v1_p2tr`. The same input also requires
//! one line whose entire text is `Previous output type V1_P2TR`. That line is
//! not `Previous output type V0_P2WSH`, and not a bare `V1_P2TR`, `p2tr`, or
//! `MULTISIG`. The spans stay. Any other prevout type, a missing prevout, and
//! a P2SH redeem input omit both lines.

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
    assert!(
        markup
            .lines()
            .any(|line| line == "Previous output type V1_P2TR"),
        "Previous output type was dropped: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "Previous output type V0_P2WSH"),
        "a v1_p2tr prevout painted Previous output type V0_P2WSH: {markup}"
    );
    for bare in ["V1_P2TR", "p2tr", "MULTISIG"] {
        assert!(
            !markup.lines().any(|line| line == bare),
            "a bare token was painted as a substitute: {bare}: {markup}"
        );
    }

    let other = paint(&tx(&prevout("v0_p2wsh"), "p2pkh", ""));
    assert!(
        !other.lines().any(|line| line == LINE),
        "a v0_p2wsh prevout painted {LINE}: {other}"
    );
    assert!(
        other
            .lines()
            .any(|line| line == "Previous output type V0_P2WSH"),
        "Previous output type V0_P2WSH was dropped: {other}"
    );
    assert!(
        !other
            .lines()
            .any(|line| line == "Previous output type V1_P2TR"),
        "a v0_p2wsh prevout painted Previous output type V1_P2TR: {other}"
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
    assert!(
        !output_only
            .lines()
            .any(|line| line == "Previous output type V1_P2TR"),
        "a p2pkh prevout painted Previous output type V1_P2TR: {output_only}"
    );
}

/// Whole line `Type V1_P2TR` when `vout.scriptpubkey_type` is `v1_p2tr`.
///
/// Native `paint_p2tr("p2trout", Some("v0_p2wpkh"), true, None, "v1_p2tr")`
/// requires one line equal to `Type V1_P2TR`. The prevout type `v1_p2tr`
/// is `Previous output type V1_P2TR` and `P2TR tapscript`, not this line.
/// The address badge `Type P2TR` is a `bc1p` string with no type field.
/// An output whose script type is the word `p2tr` does not get this line.
#[test]
fn v1_p2tr_output_paints_type_v1_p2tr_whole_line() {
    let markup = paint(&tx(&prevout("v0_p2wpkh"), "v1_p2tr", ""));
    assert!(
        markup.lines().any(|line| line == "Type V1_P2TR"),
        "the output type line was dropped: {markup}"
    );
    let p2tr_output = paint(&tx(&prevout("v0_p2wpkh"), "p2tr", ""));
    assert!(
        !p2tr_output.lines().any(|line| line == "Type V1_P2TR"),
        "a p2tr output painted Type V1_P2TR: {p2tr_output}"
    );
    let input_only = paint(&tx(&prevout("v1_p2tr"), "p2pkh", ""));
    assert!(
        !input_only.lines().any(|line| line == "Type V1_P2TR"),
        "a v1_p2tr input painted Type V1_P2TR: {input_only}"
    );
}
