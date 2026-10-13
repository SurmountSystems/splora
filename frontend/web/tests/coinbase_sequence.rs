//! Coinbase and sequence on the transaction input from `GET /tx/:txid`.
//!
//! `transaction_row_markup` paints a coinbase span only when `is_coinbase` is
//! true, and prints `sequence` including zero. A missing sequence key fails
//! input deserialization, so the label stays off the page. Script hex and asm
//! stay off the page.

use splora_web::transaction_row_markup;

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn input(
    is_coinbase: bool,
    sequence: Option<u32>,
    prevout: &str,
    script_hex: &str,
    script_asm: &str,
) -> String {
    let sequence_field = match sequence {
        Some(value) => format!(r#","sequence":{value}"#),
        None => String::new(),
    };
    format!(
        r#"{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":{prevout},"scriptsig":"{script_hex}","scriptsig_asm":"{script_asm}","is_coinbase":{is_coinbase}{sequence_field}}}"#
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

fn visible_text(markup: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for ch in markup.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    text
}

fn assert_scripts_stay_off(markup: &str, needles: &[&str]) {
    for needle in needles {
        assert_absent(markup, needle);
    }
}

#[test]
fn coinbase_input_says_it_is_a_coinbase() {
    let body = tx(
        &input(
            true,
            Some(u32::MAX),
            "null",
            "COINSIGHEXDEAD",
            "COINSIGASMDEAD",
        ),
        &output("COINOUTHEXDEAD", "COINOUTASMDEAD", "p2pkh"),
    );
    let markup = transaction_row_markup(&body);
    assert_eq!(
        markup
            .matches("<span class=\"coinbase\">coinbase</span>")
            .count(),
        1,
        "coinbase span is not once: {markup}"
    );
    assert!(
        markup.contains("<span class=\"sequence\">sequence 4294967295</span>"),
        "sequence is missing: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">p2pkh</span>"),
        "output script type is missing: {markup}"
    );
    assert_scripts_stay_off(
        &markup,
        &[
            "COINSIGHEXDEAD",
            "COINSIGASMDEAD",
            "COINOUTHEXDEAD",
            "COINOUTASMDEAD",
        ],
    );
}

#[test]
fn non_coinbase_input_does_not_say_coinbase() {
    let body = tx(
        &input(
            false,
            Some(1),
            &prevout("NCPREVHEXDEAD", "NCPREVASMDEAD", "v0_p2wpkh"),
            "NCSIGHEXDEAD",
            "NCSIGASMDEAD",
        ),
        &output("NCOUTHEXDEAD", "NCOUTASMDEAD", "p2pkh"),
    );
    let markup = transaction_row_markup(&body);
    assert_absent(&markup, "class=\"coinbase\"");
    assert!(
        !visible_text(&markup).contains("coinbase"),
        "visible word coinbase leaked: {markup}"
    );
    assert!(
        markup.contains("<span class=\"sequence\">sequence 1</span>"),
        "sequence is missing: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">input script type v0_p2wpkh</span>"),
        "input script type is missing: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">p2pkh</span>"),
        "output script type is missing: {markup}"
    );
    assert_scripts_stay_off(
        &markup,
        &[
            "NCPREVHEXDEAD",
            "NCPREVASMDEAD",
            "NCSIGHEXDEAD",
            "NCSIGASMDEAD",
            "NCOUTHEXDEAD",
            "NCOUTASMDEAD",
        ],
    );
}

#[test]
fn sequence_zero_prints_the_integer() {
    let body = tx(
        &input(
            false,
            Some(0),
            &prevout("ZPREVHEXDEAD", "ZPREVASMDEAD", "v0_p2wpkh"),
            "ZSIGHEXDEAD",
            "ZSIGASMDEAD",
        ),
        &output("ZOUTHEXDEAD", "ZOUTASMDEAD", "p2pkh"),
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.contains("<span class=\"sequence\">sequence 0</span>"),
        "sequence 0 was omitted: {markup}"
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">input script type v0_p2wpkh</span>"),
        "input script type is missing: {markup}"
    );
    assert_scripts_stay_off(
        &markup,
        &[
            "ZPREVHEXDEAD",
            "ZPREVASMDEAD",
            "ZSIGHEXDEAD",
            "ZSIGASMDEAD",
            "ZOUTHEXDEAD",
            "ZOUTASMDEAD",
        ],
    );
}

#[test]
fn missing_sequence_omits_the_label() {
    let body = tx(
        &input(
            false,
            None,
            &prevout("MPREVHEXDEAD", "MPREVASMDEAD", "v0_p2wpkh"),
            "MSIGHEXDEAD",
            "MSIGASMDEAD",
        ),
        &output("MOUTHEXDEAD", "MOUTASMDEAD", "p2pkh"),
    );
    let markup = transaction_row_markup(&body);
    assert_absent(&markup, "class=\"sequence\"");
    assert_absent(&markup, "sequence 0");
    assert_scripts_stay_off(
        &markup,
        &[
            "MPREVHEXDEAD",
            "MPREVASMDEAD",
            "MSIGHEXDEAD",
            "MSIGASMDEAD",
            "MOUTHEXDEAD",
            "MOUTASMDEAD",
        ],
    );
}
