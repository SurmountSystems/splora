//! Output script type `provably_unspendable` from `GET /tx/:txid`.
//!
//! The address-type caption is the words `provably unspendable`.
//! That string is `vout.scriptpubkey_type`. The Type cell is those words,
//! not `PROVABLY_UNSPENDABLE`. The `scriptpubkey-type` span stays the
//! indexer spelling. A different type does not paint the words.

use splora_web::transaction_row_markup;

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SCRIPT_HEX: &str = "UNSPENDPUBHEXDEAD";
const SCRIPT_ASM: &str = "UNSPENDASMDEAD";
const SCRIPT_SIG: &str = "UNSPENDSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "UNSPENDSIGASMDEAD";
const WITNESS: &str = "UNSPENDWITDEAD";
const LABEL: &str = "<span class=\"output-type\">provably unspendable</span>";

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
fn provably_unspendable() {
    let markup = transaction_row_markup(&tx(&output("provably_unspendable")));
    assert_one(&markup, LABEL);
    assert_absent(&markup, "PROVABLY_UNSPENDABLE");
    assert_one(
        &markup,
        "<span class=\"scriptpubkey-type\">provably_unspendable</span>",
    );
    assert_one(&markup, "<span class=\"output-type-label\">Type</span>");
    assert_absent(&markup, SCRIPT_HEX);
    assert_absent(&markup, SCRIPT_ASM);
    assert_absent(&markup, SCRIPT_SIG);
    assert_absent(&markup, SCRIPT_SIG_ASM);
    assert_absent(&markup, WITNESS);

    let other = transaction_row_markup(&tx(&output("p2pkh")));
    assert_absent(&other, "provably unspendable");
    assert_one(&other, "<span class=\"output-type\">P2PKH</span>");
}
