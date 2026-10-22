//! Whole line `Type` from no-query `GET /tx/:txid`.
//!
//! Angular shows the label Type beside `vout.scriptpubkey_type` uppercased.
//! The label span stays. This line is the label text alone. An empty type
//! omits it. Script hex, asm, scriptsig, and witness stay off.

use splora_web::transaction_row_markup;

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SCRIPT_HEX: &str = "TYPEOWNPUBHEXDEAD";
const SCRIPT_ASM: &str = "TYPEOWNASMDEAD";
const SCRIPT_SIG: &str = "TYPEOWNSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "TYPEOWNSIGASMDEAD";
const WITNESS: &str = "TYPEOWNWITDEAD";

fn tx(vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn output(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"{kind}","value":1}}"#
    )
}

fn assert_absent(markup: &str, needle: &str) {
    assert!(
        !markup.contains(needle),
        "{needle} leaked into the markup: {markup}"
    );
}

fn assert_scripts_stay_off(markup: &str) {
    assert_absent(markup, SCRIPT_HEX);
    assert_absent(markup, SCRIPT_ASM);
    assert_absent(markup, SCRIPT_SIG);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, WITNESS);
}

/// A nonempty `scriptpubkey_type` paints `Type` as its own line.
#[test]
fn type_paints_type_as_its_own_line() {
    const LINE: &str = "Type";
    let markup = transaction_row_markup(&tx(&output("v0_p2wpkh")));
    assert!(
        markup.lines().any(|line| line == LINE),
        "Type is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "Type was not painted once: {markup}"
    );
    assert_eq!(
        markup
            .matches("<span class=\"output-type-label\">Type</span>")
            .count(),
        1,
        "expected one Type label span in {markup}"
    );
    let blank = transaction_row_markup(&tx(&output("")));
    assert!(
        !blank.lines().any(|line| line == LINE),
        "an empty type painted {LINE}: {blank}"
    );
    assert_scripts_stay_off(&markup);
}
