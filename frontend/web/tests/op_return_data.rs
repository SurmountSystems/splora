//! OP_RETURN data label on a transaction output from `GET /tx/:txid`.
//!
//! Angular `transactions-list.component.html` paints the word `data` beside
//! `OP_RETURN` when `vout.scriptpubkey_type` is `op_return`. The load is
//! `GET /tx/:txid` with no query and no fragment. A non-op_return output
//! does not show the word. An input prevout does not show it. The script
//! hex and the script asm stay off the page.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SCRIPT: &str = "ZZPAYLOADHEXZZ";
const ASM: &str = "ZZPAYLOADASMZZ";
const LABEL: &str = "<span class=\"op-return-data-label\">data</span>";

fn input_json(prevout: &str) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"SIGHEXZZ","scriptsig_asm":"SIGASMZZ","is_coinbase":false,"sequence":1}}"#
    )
}

fn output(script: &str, asm: &str, kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{script}","scriptpubkey_asm":"{asm}","scriptpubkey_type":"{kind}","value":0}}"#
    )
}

fn tx(prevout: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1}}"#,
        input_json(prevout)
    )
}

fn paint(prevout: &str, vout: &str) -> String {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(&tx(prevout, vout))
}

fn assert_one(markup: &str, expected: &str) {
    assert_eq!(
        markup.matches(expected).count(),
        1,
        "expected one {expected} in {markup}"
    );
}

fn assert_absent(markup: &str, needle: &str) {
    assert!(
        !markup.contains(needle),
        "{needle} leaked into the markup: {markup}"
    );
}

#[test]
fn op_return_output_paints_the_data_label_once() {
    let markup = paint("null", &output(SCRIPT, ASM, "op_return"));
    assert!(markup.contains(LABEL), "data label is absent: {markup}");
    assert_one(&markup, LABEL);
    assert_one(&markup, "<span class=\"op-return\">OP_RETURN</span>");
    assert_absent(&markup, SCRIPT);
    assert_absent(&markup, ASM);
    assert_absent(&markup, "SIGHEXZZ");
    assert_absent(&markup, "SIGASMZZ");
}

#[test]
fn runestone_keeps_the_button_and_paints_data_once() {
    let markup = paint("null", &output("6a5d0614c0a2331441ee", ASM, "op_return"));
    assert!(markup.contains(LABEL), "data label is absent: {markup}");
    assert_one(&markup, LABEL);
    assert_one(
        &markup,
        "<span class=\"op-return\">OP_RETURN</span><button type=\"button\" class=\"btn btn-sm badge badge-ord\">Runestone</button>",
    );
    assert_absent(&markup, "6a5d0614c0a2331441ee");
    assert_absent(&markup, ASM);
}

#[test]
fn non_op_return_output_omits_the_data_label() {
    let markup = paint("null", &output(SCRIPT, ASM, "p2pkh"));
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "op-return-data-label");
    assert_one(&markup, "<span class=\"output-type\">P2PKH</span>");
    assert_absent(&markup, SCRIPT);
    assert_absent(&markup, ASM);
}

#[test]
/// Whole line `data` from an `op_return` output on no-query `GET /tx/:txid`.
///
/// GPUI `transaction_screen_paints_op_return_data` requires one line equal
/// to `data`. Angular `transactions-list.component.html` paints that word
/// beside `OP_RETURN`. The span stays. A runestone keeps its button. Script
/// hex and script asm stay off. A non-`op_return` output and an input
/// prevout omit the line.
#[test]
fn op_return_output_paints_data_as_its_own_line() {
    const LINE: &str = "data";
    let markup = paint("null", &output(SCRIPT, ASM, "op_return"));
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
        markup.contains(LABEL),
        "the data span was dropped: {markup}"
    );
    assert!(
        markup.contains("<span class=\"op-return\">OP_RETURN</span>"),
        "OP_RETURN was dropped: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line.starts_with("data ")),
        "{LINE} was painted as a longer sentence: {markup}"
    );
    assert_absent(&markup, SCRIPT);
    assert_absent(&markup, ASM);

    let stone = paint("null", &output("6a5d0614c0a2331441ee", ASM, "op_return"));
    assert_eq!(
        stone.lines().filter(|line| *line == LINE).count(),
        1,
        "a runestone output did not paint {LINE} once: {stone}"
    );
    assert!(
        stone.contains("Runestone"),
        "Runestone was dropped: {stone}"
    );
    assert!(
        stone.contains(
            "<span class=\"op-return\">OP_RETURN</span><button type=\"button\" class=\"btn btn-sm badge badge-ord\">Runestone</button>"
        ),
        "the runestone button left the OP_RETURN span: {stone}"
    );
    assert_absent(&stone, "6a5d0614c0a2331441ee");
    assert_absent(&stone, ASM);

    let other = paint("null", &output(SCRIPT, ASM, "p2pkh"));
    assert!(
        !other.lines().any(|line| line == LINE),
        "a non-op_return output painted {LINE}: {other}"
    );

    let prevout = output(SCRIPT, ASM, "op_return");
    let input = paint(&prevout, &output("0014aa", "OUTASMZZ", "p2pkh"));
    assert!(
        !input.lines().any(|line| line == LINE),
        "an input prevout painted {LINE}: {input}"
    );
    assert_absent(&input, SCRIPT);
    assert_absent(&input, ASM);
}

#[test]
fn input_prevout_op_return_omits_the_data_label() {
    let prevout = output(SCRIPT, ASM, "op_return");
    let markup = paint(&prevout, &output("0014aa", "OUTASMZZ", "p2pkh"));
    assert_absent(&markup, LABEL);
    assert_absent(&markup, "op-return-data-label");
    assert_one(
        &markup,
        "<span class=\"previous-output-type\">OP_RETURN</span>",
    );
    assert_one(&markup, "<span class=\"output-type\">P2PKH</span>");
    assert_absent(&markup, "<span class=\"op-return\">OP_RETURN</span>");
    assert_absent(&markup, SCRIPT);
    assert_absent(&markup, ASM);
    assert_absent(&markup, "OUTASMZZ");
}
