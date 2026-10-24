//! Whole line `nSequence 0x0100` on the transaction input from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_nsequence` requires one line whose entire
//! text is `nSequence 0x0100` when sequence is 256. The decimal span and the
//! hex span stay glued. Sequence 0 does not gain `nSequence 0x00` or
//! `nSequence 0x0100` as its own line. An empty body omits the line.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "NSEQPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "NSEQSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "NSEQSIGASMDEAD";
const SCRIPT_ASM: &str = "NSEQASMDEAD";
const WITNESS: &str = "NSEQWITDEAD";
const LINE: &str = "nSequence 0x0100";

fn tx(sequence: Option<u32>) -> String {
    let sequence_field = match sequence {
        Some(value) => format!(r#","sequence":{value}"#),
        None => String::new(),
    };
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false{sequence_field}}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
    )
}

fn sequence_spans(sequence: u32, hex: &str) -> String {
    format!(
        "<span class=\"sequence\">sequence {sequence}</span><span class=\"nsequence\">nSequence {hex}</span>"
    )
}

#[test]
fn sequence_256_paints_nsequence_0100_as_its_own_line() {
    let markup = transaction_row_markup(&tx(Some(256)));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert_eq!(
        markup.matches(&sequence_spans(256, "0x0100")).count(),
        1,
        "the glued sequence span was not one occurrence: {markup}"
    );

    let zero = transaction_row_markup(&tx(Some(0)));
    assert!(
        !zero.lines().any(|line| line == "nSequence 0x00"),
        "nSequence 0x00 was painted as its own line: {zero}"
    );
    assert!(
        !zero.lines().any(|line| line == LINE),
        "sequence 0 painted {LINE}: {zero}"
    );
    assert_eq!(
        zero.matches(&sequence_spans(0, "0x00")).count(),
        1,
        "sequence 0 dropped the hex span: {zero}"
    );

    let empty = transaction_row_markup("");
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty body painted {LINE}: {empty}"
    );
    assert!(
        !empty.lines().any(|line| line == "nSequence 0x00"),
        "an empty body painted nSequence 0x00: {empty}"
    );
    assert!(
        !empty.contains(LINE),
        "an empty body contains {LINE}: {empty}"
    );
}
