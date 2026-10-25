//! `scriptpubkey_type` on the transaction row from `GET /tx/:txid`.
//!
//! `transaction_row_markup` paints the indexer spelling. Script hex and asm stay off the page.
//! A `v0_p2wpkh` output also paints one whole line `output script type v0_p2wpkh`.
//! That line is not `input script type v0_p2wsh` and not `Previous output type V0_P2WSH`.
//! A `p2pk` output also paints one whole line `output script type p2pk`.
//! That line is not `input script type p2pk`, not `output script type op_return`,
//! not `output script type empty`, and not `output script type v0_p2wpkh`.
//! The bare token `p2pk` is not its own line. The scriptpubkey-type span stays.

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
    assert!(
        markup
            .lines()
            .any(|line| line == "output script type op_return")
    );
    assert_absent(&markup, "OPRETHEXDEAD");
    assert_absent(&markup, "OPRETASMDEAD");
}

#[test]
fn v0_p2wpkh_output_paints_output_script_type_v0_p2wpkh_as_its_own_line() {
    let body = tx(
        &format!(
            "{},{},{}",
            input(&prevout("P2PKPREVHEXDEAD", "P2PKPREVASMDEAD", "p2pk")),
            input(&prevout(
                "OPRETURNPREVHEXDEAD",
                "OPRETURNPREVASMDEAD",
                "op_return",
            )),
            input(&prevout(
                "V0P2WSHPREVHEXDEAD",
                "V0P2WSHPREVASMDEAD",
                "v0_p2wsh",
            )),
        ),
        &output("V0P2WPKHPUBHEXDEAD", "V0P2WPKHASMDEAD", "v0_p2wpkh"),
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup
            .lines()
            .any(|line| line == "output script type v0_p2wpkh"),
        "no line equals output script type v0_p2wpkh: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">v0_p2wpkh</span>"),
        "the scriptpubkey-type span was dropped: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "input script type p2pk"),
        "input script type p2pk was dropped: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == "input script type op_return"),
        "input script type op_return was dropped: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == "Previous output type V0_P2WSH"),
        "Previous output type V0_P2WSH was dropped: {markup}"
    );
    for bare in [
        "p2pk",
        "op_return",
        "v0_p2wsh",
        "V0_P2WSH",
        "MULTISIG",
        "multisig",
        "EMPTY",
    ] {
        assert!(
            !markup.lines().any(|line| line == bare),
            "a bare token was painted as a substitute: {bare}: {markup}"
        );
    }
    assert_absent(&markup, "V0P2WPKHPUBHEXDEAD");
    assert_absent(&markup, "V0P2WPKHASMDEAD");
    assert_absent(&markup, "P2PKPREVHEXDEAD");
    assert_absent(&markup, "P2PKPREVASMDEAD");
    assert_absent(&markup, "OPRETURNPREVHEXDEAD");
    assert_absent(&markup, "OPRETURNPREVASMDEAD");
    assert_absent(&markup, "V0P2WSHPREVHEXDEAD");
    assert_absent(&markup, "V0P2WSHPREVASMDEAD");
    assert_absent(&markup, "SCRIPTSIGHEXDEAD");
    assert_absent(&markup, "SCRIPTSIGASMDEAD");
}

#[test]
fn p2pk_output_paints_output_script_type_p2pk_as_its_own_line() {
    let body = tx(
        &input(&prevout(
            "V0P2WPKHPREVHEXDEAD",
            "V0P2WPKHPREVASMDEAD",
            "v0_p2wpkh",
        )),
        &output("P2PKPUBHEXDEAD", "P2PKASMDEAD", "p2pk"),
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.lines().any(|line| line == "output script type p2pk"),
        "no line equals output script type p2pk: {markup}"
    );
    assert_eq!(
        markup
            .lines()
            .filter(|line| *line == "output script type p2pk")
            .count(),
        1,
        "output script type p2pk was dropped or repeated: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == "output script type p2pk" && !line.contains('<')),
        "output script type p2pk was inside a span: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">p2pk</span>"),
        "the scriptpubkey-type span was dropped: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "Type P2PK"),
        "Type P2PK was dropped: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "p2pk"),
        "a bare token was painted as a substitute: p2pk: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "input script type p2pk"),
        "an output-only p2pk fixture painted input script type p2pk: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "output script type op_return"),
        "output script type op_return was painted on a p2pk output: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "output script type empty"),
        "output script type empty was painted on a p2pk output: {markup}"
    );
    assert!(
        !markup
            .lines()
            .any(|line| line == "output script type v0_p2wpkh"),
        "output script type v0_p2wpkh was painted on a p2pk output: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == "Previous output type V0_P2WPKH"),
        "Previous output type V0_P2WPKH was dropped: {markup}"
    );
    assert_absent(&markup, "P2PKPUBHEXDEAD");
    assert_absent(&markup, "P2PKASMDEAD");
    assert_absent(&markup, "V0P2WPKHPREVHEXDEAD");
    assert_absent(&markup, "V0P2WPKHPREVASMDEAD");
    assert_absent(&markup, "SCRIPTSIGHEXDEAD");
    assert_absent(&markup, "SCRIPTSIGASMDEAD");
}
