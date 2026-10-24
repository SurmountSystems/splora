//! Whole line `Previous output script` on the transaction input from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_previous_output_script` requires one line
//! whose entire text is `Previous output script` when `vin.prevout` is present.
//! The span stays. The null-prevout `empty` output that
//! `empty_output_type.rs` treats as absent does not gain that line.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "Previous output script";
const LABEL_SPAN: &str =
    "<span class=\"previous-output-script-label\">Previous output script</span>";
const SCRIPT_HEX: &str = "PREVOUTHEXDEAD";
const SCRIPT_ASM: &str = "PREVOUTASMDEAD";
const SCRIPT_SIG: &str = "PREVSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "PREVSIGASMDEAD";
const WITNESS: &str = "PREVWITDEAD";
const LEAF_VERSION: &str = "NILLEAFDEAD";
const INNER_TAPSCRIPT: &str = "NILTAPSCRIPTDEAD";

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn tx_confirmed(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
    )
}

fn input(prevout: &str, extra: &str) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1{extra}}}"#
    )
}

fn absent_input() -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}","{LEAF_VERSION}"],"is_coinbase":false,"sequence":1,"inner_witnessscript_asm":"{INNER_TAPSCRIPT}"}}"#
    )
}

fn prevout(kind: &str, asm: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{asm}","scriptpubkey_type":"{kind}"}}"#
    )
}

fn output(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"{kind}","value":1}}"#
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
fn prevout_paints_previous_output_script_as_its_own_line() {
    let markup = paint(&tx(
        &input(&prevout("v0_p2wpkh", SCRIPT_ASM), ""),
        &output("p2pkh"),
    ));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(line_count(&markup), 1, "{LINE} was not one line: {markup}");
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "{LINE} was inside a span: {markup}"
    );
    assert_eq!(markup.matches(LABEL_SPAN).count(), 1, "{markup}");

    // `empty_output_shows_uppercase_type_once` treats the sentence as absent.
    let absent = paint(&tx_confirmed(&absent_input(), &output("empty")));
    assert!(
        !absent.lines().any(|line| line == LINE),
        "a null prevout on an empty output painted {LINE}: {absent}"
    );
    assert_eq!(
        line_count(&absent),
        0,
        "a null prevout on an empty output painted {LINE}: {absent}"
    );
}
