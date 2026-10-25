//! Address word `Issuance` on one transaction input from `GET /tx/:txid`.
//!
//! Angular paints `vin.issuance ? 'Issuance' : 'UNKNOWN'` in the address cell
//! when that input has no prevout. JSON null and an omitted prevout key are
//! both no prevout. An ordinary prevout does not gain the word. A missing
//! issuance key and JSON null issuance do not gain it. An issuance object
//! that also has a prevout keeps the field lines and does not gain the word.
//! No amount is invented. The path has no query and no fragment.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";
const ASSET_ENTROPY: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const SCRIPT_SIG: &str = "ISSUESIGHEXDEAD";
const SCRIPT_SIG_ASM: &str = "ISSUESIGASMDEAD";
const WITNESS: &str = "ISSUEWITDEAD";
const OUT_SCRIPT: &str = "ISSUEOUTHEXDEAD";
const OUT_ASM: &str = "ISSUEOUTASMDEAD";
const PREV_SCRIPT: &str = "ISSUEPREVHEXDEAD";
const PREV_ASM: &str = "ISSUEPREVASMDEAD";
const LABEL: &str = "Issuance";

fn issuance_object() -> String {
    format!(
        r#"{{"asset_id":"{ASSET_ID}","is_reissuance":false,"asset_entropy":"{ASSET_ENTROPY}"}}"#
    )
}

fn input(prevout_field: &str, issuance_field: &str) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,{prevout_field}{issuance_field}"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1}}"#
    )
}

fn tx(vin: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin}],"vout":[{{"scriptpubkey":"{OUT_SCRIPT}","scriptpubkey_asm":"{OUT_ASM}","scriptpubkey_type":"op_return","value":1}}],"size":10,"weight":40,"sigops":0,"fee":1,"status":{{"confirmed":true,"block_height":4,"block_hash":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd","block_time":1600000000}}}}"#
    )
}

fn paint(vin: &str) -> String {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(&tx(vin))
}

fn label_count(markup: &str) -> usize {
    markup.matches(LABEL).count()
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

fn assert_no_amount(markup: &str) {
    assert_absent(markup, "issuance asset amount");
    assert_absent(markup, "issuance token amount");
    assert_absent(markup, "Confidential");
}

fn assert_field_lines(markup: &str) {
    let asset = format!("issuance asset {ASSET_ID}");
    assert_eq!(
        markup.matches(&asset).count(),
        2,
        "expected two {asset} in {markup}"
    );
    assert_one(markup, "issuance reissuance false");
    assert_one(markup, &format!("issuance entropy {ASSET_ENTROPY}"));
}

fn assert_address(markup: &str, address: &str) {
    assert_one(
        markup,
        &format!("<a class=\"address\" href=\"/address/{address}\">{address}</a>"),
    );
}

fn assert_timestamp_stays_raw(markup: &str) {
    assert_one(markup, "<td class=\"timestamp-label\">Timestamp</td>");
    assert_one(markup, "<td class=\"block-time\">1600000000</td>");
    assert_absent(markup, "2020-");
    assert_absent(markup, "1,600,000,000");
}

fn assert_word_in_input_row(markup: &str) {
    let parties = markup.find("<td class=\"parties\">").expect("parties cell");
    let word = markup.find(&format!(">{LABEL}<")).expect(LABEL);
    let close = markup[parties..]
        .find("</td>")
        .map(|end| parties + end)
        .expect("parties cell close");
    assert!(
        parties < word && word < close,
        "{LABEL} was not inside the input row: {markup}"
    );
}

#[test]
fn issuance_without_prevout_paints_the_word_once() {
    let issued = input(
        r#""prevout":null,"#,
        &format!(r#""issuance":{},"#, issuance_object()),
    );
    let ordinary = input(
        r#""prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":5},"#,
        "",
    );
    let markup = paint(&format!("{issued},{ordinary}"));
    assert_eq!(
        label_count(&markup),
        1,
        "{LABEL} was not one word on the issuance input with no prevout: {markup}"
    );
    assert_word_in_input_row(&markup);
    assert_field_lines(&markup);
    assert_address(&markup, "bc1qordinary");
    assert_absent(&markup, "UNKNOWN");
    assert_eq!(
        markup
            .matches("<span class=\"sequence\">sequence 1</span>")
            .count(),
        2,
        "both inputs keep sequence: {markup}"
    );
    assert_no_amount(&markup);
    assert_timestamp_stays_raw(&markup);
    assert_sentinels_stay_off(&markup);
    assert_absent(&markup, "<h1>Asset</h1>");
}

#[test]
fn omitted_prevout_key_with_issuance_paints_the_word_once() {
    let markup = paint(&input("", &format!(r#""issuance":{},"#, issuance_object())));
    assert_eq!(
        label_count(&markup),
        1,
        "an omitted prevout key did not paint {LABEL} once: {markup}"
    );
    assert_field_lines(&markup);
    assert_absent(&markup, "UNKNOWN");
    assert_no_amount(&markup);
    assert_sentinels_stay_off(&markup);
}

#[test]
fn ordinary_prevout_does_not_gain_the_word() {
    let markup = paint(&input(
        r#""prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":5},"#,
        "",
    ));
    assert_eq!(
        label_count(&markup),
        0,
        "an ordinary prevout painted {LABEL}: {markup}"
    );
    assert_address(&markup, "bc1qordinary");
    assert_absent(&markup, "UNKNOWN");
    assert_absent(&markup, "issuance asset");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn missing_issuance_key_does_not_gain_the_word() {
    let markup = paint(&input(r#""prevout":null,"#, ""));
    assert_eq!(
        label_count(&markup),
        0,
        "a missing issuance key painted {LABEL}: {markup}"
    );
    assert_one(&markup, "<span class=\"unknown-input\">UNKNOWN</span>");
    assert_absent(&markup, "issuance asset");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn null_issuance_does_not_gain_the_word() {
    let markup = paint(&input(r#""prevout":null,"#, r#""issuance":null,"#));
    assert_eq!(
        label_count(&markup),
        0,
        "JSON null issuance painted {LABEL}: {markup}"
    );
    assert_one(&markup, "<span class=\"unknown-input\">UNKNOWN</span>");
    assert_absent(&markup, "issuance asset");
    assert_sentinels_stay_off(&markup);
}

#[test]
fn issuance_with_prevout_keeps_field_lines_without_the_word() {
    let markup = paint(&input(
        r#""prevout":{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qwithprev","value":8},"#,
        &format!(r#""issuance":{},"#, issuance_object()),
    ));
    assert_eq!(
        label_count(&markup),
        0,
        "an issuance input that has a prevout painted {LABEL}: {markup}"
    );
    assert_field_lines(&markup);
    assert_address(&markup, "bc1qwithprev");
    assert_absent(&markup, "UNKNOWN");
    assert_no_amount(&markup);
    assert_sentinels_stay_off(&markup);
}
