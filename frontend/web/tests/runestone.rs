//! Runestone on a non-liquid transaction output from `GET /tx/:txid`.
//!
//! Angular paints a button whose visible text is Runestone beside the
//! existing OP_RETURN text when `scriptpubkey_type` is `op_return` and
//! `scriptpubkey` hex starts with `6a5d`. The load is `GET /tx/:txid` with
//! no query and no fragment. An OP_RETURN whose script does not start with
//! that prefix does not show the word. A non-OP_RETURN output does not show
//! it. An input prevout does not show it. The script hex is not a label.
//! Script asm, scriptsig, and witness stay off the page.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SCRIPT_SIG: &str = "RUNESIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "RUNESIGASMDEAD";
const WITNESS: &str = "RUNEWITDEAD";
const RUNESTONE_SCRIPT: &str = "6a5d0614c0a2331441ee";
const RUNESTONE_ASM: &str = "RUNEASMDEAD";
const PLAIN_SCRIPT: &str = "6a0b68656c6c6f";
const PLAIN_ASM: &str = "PLAINASMDEAD";
const EMBEDDED_SCRIPT: &str = "aa6a5ddeadbeef";
const EMBEDDED_ASM: &str = "EMBEDDEDASMDEAD";
const UPPER_SCRIPT: &str = "6A5Ddeadbeef";
const UPPER_ASM: &str = "UPPERASMDEAD";
const OTHER_SCRIPT: &str = "6a5d0014aabbccdd";
const OTHER_ASM: &str = "OTHERASMDEAD";
const EXACT_SCRIPT: &str = "6a5d";
const EXACT_ASM: &str = "EXACTASMDEAD";
const WORD: &str = "Runestone";
const OP_RETURN_SPAN: &str = "<span class=\"op-return\">OP_RETURN</span>";
const BUTTON: &str =
    "<button type=\"button\" class=\"btn btn-sm badge badge-ord\">Runestone</button>";
const BESIDE: &str = "<span class=\"op-return\">OP_RETURN</span><button type=\"button\" class=\"btn btn-sm badge badge-ord\">Runestone</button>";

fn input_json(prevout: &str) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}"#
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

fn assert_scripts_stay_off(markup: &str) {
    for needle in [
        RUNESTONE_SCRIPT,
        RUNESTONE_ASM,
        PLAIN_SCRIPT,
        PLAIN_ASM,
        EMBEDDED_SCRIPT,
        EMBEDDED_ASM,
        UPPER_SCRIPT,
        UPPER_ASM,
        OTHER_SCRIPT,
        OTHER_ASM,
        EXACT_SCRIPT,
        EXACT_ASM,
        SCRIPT_SIG,
        SCRIPT_SIG_ASM,
        WITNESS,
    ] {
        assert_absent(markup, needle);
    }
}

fn assert_op_return_stays(markup: &str) {
    assert_one(markup, OP_RETURN_SPAN);
    assert_one(markup, "<span class=\"scriptpubkey-type\">op_return</span>");
    assert_one(markup, "<span class=\"output-type-label\">Type</span>");
    assert_one(markup, "<span class=\"output-type\">OP_RETURN</span>");
}

#[test]
fn prefixed_op_return_paints_runestone_beside_op_return() {
    let markup = paint(
        "null",
        &output(RUNESTONE_SCRIPT, RUNESTONE_ASM, "op_return"),
    );
    assert_op_return_stays(&markup);
    assert_eq!(
        markup.matches(WORD).count(),
        1,
        "{WORD} was not the visible word on the no-query transaction: {markup}"
    );
    assert_one(&markup, BUTTON);
    assert_one(&markup, BESIDE);
    assert_one(&markup, "<h2>Inputs & Outputs</h2>");
    assert_scripts_stay_off(&markup);
}

#[test]
fn exact_prefix_paints_runestone_once() {
    let markup = paint("null", &output(EXACT_SCRIPT, EXACT_ASM, "op_return"));
    assert_op_return_stays(&markup);
    assert_eq!(
        markup.matches(WORD).count(),
        1,
        "{WORD} was not the visible word for scriptpubkey 6a5d: {markup}"
    );
    assert_one(&markup, BESIDE);
    assert_scripts_stay_off(&markup);
}

#[test]
fn op_return_without_the_prefix_omits_runestone() {
    for (script, asm) in [
        (PLAIN_SCRIPT, PLAIN_ASM),
        (EMBEDDED_SCRIPT, EMBEDDED_ASM),
        (UPPER_SCRIPT, UPPER_ASM),
    ] {
        let markup = paint("null", &output(script, asm, "op_return"));
        assert_op_return_stays(&markup);
        assert_absent(&markup, WORD);
        assert_absent(&markup, "badge-ord");
        assert_scripts_stay_off(&markup);
    }
}

#[test]
fn non_op_return_with_the_prefix_omits_runestone() {
    let markup = paint("null", &output(OTHER_SCRIPT, OTHER_ASM, "p2pkh"));
    assert_one(&markup, "<span class=\"scriptpubkey-type\">p2pkh</span>");
    assert_one(&markup, "<span class=\"output-type\">P2PKH</span>");
    assert_absent(&markup, OP_RETURN_SPAN);
    assert_absent(&markup, WORD);
    assert_absent(&markup, "badge-ord");
    assert_scripts_stay_off(&markup);
}

#[test]
fn input_prevout_with_the_prefix_omits_runestone() {
    let prevout = output(RUNESTONE_SCRIPT, RUNESTONE_ASM, "op_return");
    let markup = paint(&prevout, &output("0014aa", "OUTASMDEAD", "p2pkh"));
    assert_one(
        &markup,
        "<span class=\"previous-output-type\">OP_RETURN</span>",
    );
    assert_one(&markup, "<span class=\"output-type\">P2PKH</span>");
    assert_absent(&markup, OP_RETURN_SPAN);
    assert_absent(&markup, WORD);
    assert_absent(&markup, "badge-ord");
    assert_absent(&markup, "OUTASMDEAD");
    assert_scripts_stay_off(&markup);
}
