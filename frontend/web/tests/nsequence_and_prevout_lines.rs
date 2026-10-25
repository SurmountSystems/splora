//! Whole lines from `GET /tx/:txid`.
//!
//! GPUI requires `nSequence 0xffffffff` when sequence is 4294967295,
//! `Previous output type V0_P2WPKH` when the prevout type is `v0_p2wpkh`,
//! and `Previous output type P2PK` when the prevout type is `p2pk`.
//! Sequence 256 keeps `nSequence 0x0100`. Sequence 0 does not gain
//! `nSequence 0x00`. The hex span and the previous-output-type span stay.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SCRIPT_HEX: &str = "NPVOUTHEXDEAD";
const SCRIPT_ASM: &str = "NPVOUTASMDEAD";
const SCRIPT_SIG: &str = "NPVSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "NPVSIGASMDEAD";
const WITNESS: &str = "NPVWITDEAD";
const NSEQUENCE_LINE: &str = "nSequence 0xffffffff";
const V0_LINE: &str = "Previous output type V0_P2WPKH";
const P2PK_LINE: &str = "Previous output type P2PK";

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true}}}}"#
    )
}

fn input(prevout: &str, sequence: u32) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":{sequence}}}"#
    )
}

fn prevout(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"{kind}","value":5}}"#
    )
}

fn output(kind: &str) -> String {
    format!(
        r#"{{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"{kind}","scriptpubkey_address":"bc1qordinary","value":9}}"#
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

fn assert_one_line(markup: &str, line: &str) {
    assert!(
        markup.lines().any(|row| row == line),
        "no line equals {line}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|row| *row == line).count(),
        1,
        "{line} was not one line: {markup}"
    );
}

#[test]
fn nsequence_and_prevout_ffffffff_line() {
    let markup = paint(&tx(&input("null", 4294967295), &output("p2pkh")));
    assert_one_line(&markup, NSEQUENCE_LINE);
    assert_eq!(
        markup
            .matches("<span class=\"nsequence\">nSequence 0xffffffff</span>")
            .count(),
        1,
        "the hex span was not one occurrence: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == "sequence 4294967295"),
        "sequence 4294967295 decimal line was dropped: {markup}"
    );

    let padded = paint(&tx(&input("null", 256), &output("p2pkh")));
    assert!(
        padded.lines().any(|line| line == "nSequence 0x0100"),
        "sequence 256 dropped nSequence 0x0100: {padded}"
    );
    assert!(
        !padded.lines().any(|line| line == NSEQUENCE_LINE),
        "sequence 256 painted {NSEQUENCE_LINE}: {padded}"
    );

    let zero = paint(&tx(&input("null", 0), &output("p2pkh")));
    assert!(
        !zero.lines().any(|line| line == "nSequence 0x00"),
        "nSequence 0x00 was painted as its own line: {zero}"
    );
    assert!(
        !zero.lines().any(|line| line == NSEQUENCE_LINE),
        "sequence 0 painted {NSEQUENCE_LINE}: {zero}"
    );
    assert_eq!(
        zero.matches("<span class=\"nsequence\">nSequence 0x00</span>")
            .count(),
        1,
        "sequence 0 dropped the hex span: {zero}"
    );

    let empty = paint("");
    assert!(
        !empty.lines().any(|line| line == NSEQUENCE_LINE),
        "an empty body painted {NSEQUENCE_LINE}: {empty}"
    );
}

#[test]
fn nsequence_and_prevout_v0_p2wpkh_line() {
    let markup = paint(&tx(&input(&prevout("v0_p2wpkh"), 1), &output("p2pkh")));
    assert_one_line(&markup, V0_LINE);
    assert_eq!(
        markup
            .matches("<span class=\"previous-output-type\">V0_P2WPKH</span>")
            .count(),
        1,
        "the previous-output-type span was not one occurrence: {markup}"
    );
    assert!(
        markup.contains("<span class=\"previous-output-type-label\">Previous output type</span>"),
        "the previous output type label span was dropped: {markup}"
    );

    let other = paint(&tx(&input(&prevout("p2sh"), 1), &output("p2pkh")));
    assert!(
        !other.lines().any(|line| line == V0_LINE),
        "a p2sh prevout painted {V0_LINE}: {other}"
    );

    let empty = paint("");
    assert!(
        !empty.lines().any(|line| line == V0_LINE),
        "an empty body painted {V0_LINE}: {empty}"
    );
}

#[test]
fn nsequence_and_prevout_p2pk_line() {
    let markup = paint(&tx(&input(&prevout("p2pk"), 1), &output("v0_p2wpkh")));
    assert_one_line(&markup, P2PK_LINE);
    assert_eq!(
        markup
            .matches("<span class=\"previous-output-type\">P2PK</span>")
            .count(),
        1,
        "the previous-output-type span was not one occurrence: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Type P2PK"),
        "Type P2PK was painted on the input path: {markup}"
    );

    let other = paint(&tx(&input(&prevout("v0_p2wpkh"), 1), &output("v0_p2wpkh")));
    assert!(
        !other.lines().any(|line| line == P2PK_LINE),
        "a v0_p2wpkh prevout painted {P2PK_LINE}: {other}"
    );

    let empty = paint("");
    assert!(
        !empty.lines().any(|line| line == P2PK_LINE),
        "an empty body painted {P2PK_LINE}: {empty}"
    );
}
