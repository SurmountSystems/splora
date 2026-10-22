//! Whole line `Locktime` from no-query `GET /tx/:txid`.
//!
//! Angular shows the label Locktime beside the raw `tx.locktime` number.
//! The label cell stays. This line is the label text alone. The sentence
//! `Locktime 0` stays off. Script hex, asm, scriptsig, and witness stay off.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "LOCKOWNPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "LOCKOWNSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "LOCKOWNSIGASMDEAD";
const SCRIPT_ASM: &str = "LOCKOWNASMDEAD";
const WITNESS: &str = "LOCKOWNWITDEAD";

fn tx(locktime: u32) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":{locktime},"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
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
    assert_absent(markup, SCRIPT_SIG_HEX);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, SCRIPT_ASM);
    assert_absent(markup, WITNESS);
}

/// The label cell stays. This line is the label text alone.
#[test]
fn locktime_paints_locktime_as_its_own_line() {
    const LINE: &str = "Locktime";
    let markup = transaction_row_markup(&tx(0));
    assert!(
        markup.lines().any(|line| line == LINE),
        "Locktime is absent: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "Locktime was not painted once: {markup}"
    );
    assert_eq!(
        markup
            .matches("<td class=\"locktime-label\">Locktime</td>")
            .count(),
        1,
        "expected one locktime label cell in {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Locktime 0"),
        "Locktime 0 was painted: {markup}"
    );
    assert_scripts_stay_off(&markup);
}
