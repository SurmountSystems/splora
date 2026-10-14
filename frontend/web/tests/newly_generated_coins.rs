//! `(Newly Generated Coins)` after the coinbase span.
//!
//! Angular paints that parenthetical in the coinbase switch case, and only
//! when the network is not liquid and not liquidtestnet. This markup takes no
//! network. Coinbase true paints the parenthetical. A false coinbase does not.
//! A missing `is_coinbase` key and JSON null fail parse, so the row stays
//! empty. Script hex, scriptsig asm, and witness stay off the page.

use splora_web::transaction_row_markup;

const SCRIPT_SIG: &str = "NEWLYSIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "NEWLYSIGASMDEAD";
const WITNESS: &str = "NEWLYWITDEAD";
const OUT_SCRIPT: &str = "NEWLYOUTHEXDEAD";
const OUT_ASM: &str = "NEWLYOUTASMDEAD";
const PREV_SCRIPT: &str = "NEWLYPREVHEXDEAD";
const PREV_ASM: &str = "NEWLYPREVASMDEAD";

const COINBASE_SPAN: &str = "<span class=\"coinbase\">coinbase</span>";
const NEWLY_SPAN: &str = "<span class=\"newly-generated-coins\">(Newly Generated Coins)</span>";
const ADJACENT: &str = "<span class=\"coinbase\">coinbase</span><span class=\"newly-generated-coins\">(Newly Generated Coins)</span>";

fn tx(vin: &str) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{vin}],"vout":[{{"scriptpubkey":"{OUT_SCRIPT}","scriptpubkey_asm":"{OUT_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":7,"sigops":0,"fee":1}}"#
    )
}

fn input(prevout_field: &str, coinbase_field: &str) -> String {
    format!(
        r#"{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,{prevout_field}"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"]{coinbase_field},"sequence":1}}"#
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

fn assert_sentinels_stay_off(markup: &str) {
    assert_absent(markup, SCRIPT_SIG);
    assert_absent(markup, SCRIPT_SIG_ASM);
    assert_absent(markup, WITNESS);
    assert_absent(markup, OUT_SCRIPT);
    assert_absent(markup, OUT_ASM);
    assert_absent(markup, PREV_SCRIPT);
    assert_absent(markup, PREV_ASM);
}

#[test]
fn coinbase_true_paints_newly_generated_coins_once() {
    let markup =
        transaction_row_markup(&tx(&input(r#""prevout":null,"#, r#","is_coinbase":true"#)));
    assert_one(&markup, COINBASE_SPAN);
    assert_one(&markup, NEWLY_SPAN);
    assert_one(&markup, ADJACENT);
    assert_absent(&markup, "UNKNOWN");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn coinbase_false_with_prevout_omits_the_parenthetical() {
    let prevout = format!(
        r#""prevout":{{"scriptpubkey":"{PREV_SCRIPT}","scriptpubkey_asm":"{PREV_ASM}","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qnewly"}},"#
    );
    let markup = transaction_row_markup(&tx(&input(&prevout, r#","is_coinbase":false"#)));
    assert_absent(&markup, NEWLY_SPAN);
    assert_absent(&markup, "(Newly Generated Coins)");
    assert_absent(&markup, COINBASE_SPAN);
    assert_absent(&markup, "UNKNOWN");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn omitted_is_coinbase_key_omits_the_parenthetical() {
    let markup = transaction_row_markup(&tx(&input(r#""prevout":null,"#, "")));
    assert_absent(&markup, NEWLY_SPAN);
    assert_absent(&markup, "(Newly Generated Coins)");
    assert_absent(&markup, "<tr");
}

#[test]
fn null_is_coinbase_omits_the_parenthetical() {
    let markup =
        transaction_row_markup(&tx(&input(r#""prevout":null,"#, r#","is_coinbase":null"#)));
    assert_absent(&markup, NEWLY_SPAN);
    assert_absent(&markup, "(Newly Generated Coins)");
    assert_absent(&markup, "<tr");
}
