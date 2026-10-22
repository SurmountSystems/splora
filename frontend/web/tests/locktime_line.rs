//! Whole line `Locktime 0` is not painted from no-query `GET /tx/:txid`.
//!
//! Angular paints the label Locktime and the en-US number pipe of `tx.locktime`
//! as separate cells. That is not the sentence `Locktime 0`. The label
//! cell and the raw integer cell stay. A large locktime stays in those cells.
//! `Locktime 500000000` and `Locktime 500,000,000` are not this line. Script
//! hex, asm, scriptsig, and witness stay off the page. No left-to-right mark.

use splora_web::transaction_row_markup;

const SCRIPT_HEX: &str = "LOCKLINEPUBHEXDEAD";
const SCRIPT_SIG_HEX: &str = "LOCKLINESIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "LOCKLINESIGASMDEAD";
const SCRIPT_ASM: &str = "LOCKLINEASMDEAD";
const WITNESS: &str = "LOCKLINEWITDEAD";

fn tx(locktime: u32) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":{locktime},"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG_HEX}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
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

fn assert_scripts_stay_off(markup: &str) {
    assert_absent(markup, SCRIPT_HEX);
    assert_absent(markup, SCRIPT_SIG_HEX);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, SCRIPT_ASM);
    assert_absent(markup, WITNESS);
    assert_absent(markup, "&lrm;");
    assert_absent(markup, "\u{200e}");
}

#[test]
fn locktime_zero_paints_locktime_0_as_its_own_line() {
    const LINE: &str = "Locktime 0";
    let markup = transaction_row_markup(&tx(0));
    assert!(
        !markup.lines().any(|line| line == LINE),
        "{LINE} is still a whole line: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        0,
        "{LINE} was a whole line: {markup}"
    );
    assert_one(&markup, "<td class=\"locktime-label\">Locktime</td>");
    assert_one(&markup, "<td class=\"locktime\">0</td>");
    assert!(
        !markup.contains("Version"),
        "locktime 0 painted the word Version: {markup}"
    );
    assert_scripts_stay_off(&markup);

    let raw = transaction_row_markup(&tx(500_000_000));
    assert!(
        !raw.lines().any(|line| line == "Locktime 500000000"),
        "locktime 500000000 painted Locktime 500000000: {raw}"
    );
    assert!(
        !raw.lines().any(|line| line == "Locktime 500,000,000"),
        "locktime 500000000 painted Locktime 500,000,000: {raw}"
    );
    assert!(
        !raw.lines().any(|line| line == LINE),
        "locktime 500000000 painted Locktime 0: {raw}"
    );
    assert_one(&raw, "<td class=\"locktime\">500000000</td>");
    assert_absent(&raw, "500,000,000");
    assert_scripts_stay_off(&raw);
}
