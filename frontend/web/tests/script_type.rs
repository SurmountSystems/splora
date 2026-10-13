//! `scriptpubkey_type` on the transaction row from `GET /tx/:txid`.
//!
//! `transaction_row_markup` paints the indexer spelling. Script hex and asm stay off the page.

use splora_web::transaction_row_markup;

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn input(prevout: &str) -> String {
    format!(
        r#"{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":{prevout},"scriptsig":"SCRIPTSIGHEXDEAD","scriptsig_asm":"SCRIPTSIGASMDEAD","is_coinbase":false,"sequence":1}}"#
    )
}

fn prevout(script: &str, asm: &str, kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{script}","scriptpubkey_asm":"{asm}","scriptpubkey_type":"{kind}"}}"#
    )
}

fn output(script: &str, asm: &str, kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{script}","scriptpubkey_asm":"{asm}","scriptpubkey_type":"{kind}"}}"#
    )
}

fn assert_absent(markup: &str, needle: &str) {
    assert!(
        !markup.contains(needle),
        "{needle} leaked into the markup: {markup}"
    );
}

#[test]
fn output_paints_its_script_type() {
    let body = tx(
        &input("null"),
        &output("PUBKEYHEXDEAD", "OP_DUP_ASM", "p2pkh"),
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">p2pkh</span>"),
        "output script type is missing: {markup}"
    );
    assert_absent(&markup, "PUBKEYHEXDEAD");
    assert_absent(&markup, "OP_DUP_ASM");
}

#[test]
fn input_paints_the_previous_output_script_type() {
    let body = tx(
        &input(&prevout("PREVOUTHEXDEAD", "PREVOUTASMDEAD", "v0_p2wpkh")),
        &output("OUTPUBKEYHEXDEAD", "OUTASMDEAD", "p2pkh"),
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">input script type v0_p2wpkh</span>"),
        "input script type is missing: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">p2pkh</span>"),
        "output script type is missing: {markup}"
    );
    assert_absent(&markup, "PREVOUTHEXDEAD");
    assert_absent(&markup, "PREVOUTASMDEAD");
    assert_absent(&markup, "OUTPUBKEYHEXDEAD");
    assert_absent(&markup, "OUTASMDEAD");
}

#[test]
fn empty_script_type_stays_off_the_page() {
    let body = tx(
        &input(&prevout("EMPTYPREVHEXDEAD", "EMPTYPREVASMDEAD", "")),
        &output("EMPTYOUTHEXDEAD", "EMPTYOUTASMDEAD", ""),
    );
    let markup = transaction_row_markup(&body);
    assert_absent(&markup, "scriptpubkey-type");
    assert_absent(&markup, "EMPTYPREVHEXDEAD");
    assert_absent(&markup, "EMPTYPREVASMDEAD");
    assert_absent(&markup, "EMPTYOUTHEXDEAD");
    assert_absent(&markup, "EMPTYOUTASMDEAD");
}

#[test]
fn missing_prevout_does_not_invent_an_input_script_type() {
    let body = tx(
        &input("null"),
        &output("MISSINGPREVHEXDEAD", "MISSINGPREVASMDEAD", "p2pkh"),
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">p2pkh</span>"),
        "output script type is missing: {markup}"
    );
    assert_absent(&markup, "input script type");
}

#[test]
fn op_return_keeps_the_existing_span() {
    let body = tx(
        &input("null"),
        &output("OPRETHEXDEAD", "OPRETASMDEAD", "op_return"),
    );
    let markup = transaction_row_markup(&body);
    assert_eq!(
        markup
            .matches("<span class=\"scriptpubkey-type\">op_return</span>")
            .count(),
        1,
        "op_return type span is not once: {markup}"
    );
    assert_eq!(
        markup
            .matches("<span class=\"op-return\">OP_RETURN</span>")
            .count(),
        1,
        "OP_RETURN span is not once: {markup}"
    );
    assert_absent(&markup, "OPRETHEXDEAD");
    assert_absent(&markup, "OPRETASMDEAD");
}
