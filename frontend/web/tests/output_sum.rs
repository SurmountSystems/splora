//! Output total on the transaction page from `GET /tx/:txid`.
//!
//! `transaction_row_markup` paints the sum of output values after the
//! transaction table. A missing value omits the total. A commitment is not
//! an amount. Script hex and asm stay off the page.

use splora_web::transaction_row_markup;

fn tx(vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn input(script_hex: &str, script_asm: &str) -> String {
    format!(
        r#"{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"{script_hex}","scriptsig_asm":"{script_asm}","is_coinbase":false,"sequence":1}}"#
    )
}

fn output(script: &str, asm: &str, value: Option<u64>, commitment: Option<&str>) -> String {
    let mut fields = format!(
        r#""scriptpubkey":"{script}","scriptpubkey_asm":"{asm}","scriptpubkey_type":"p2pkh""#
    );
    if let Some(value) = value {
        fields.push_str(&format!(r#","value":{value}"#));
    }
    if let Some(commitment) = commitment {
        fields.push_str(&format!(r#","valuecommitment":"{commitment}""#));
    }
    format!("{{{fields}}}")
}

fn assert_absent(markup: &str, needle: &str) {
    assert!(
        !markup.contains(needle),
        "{needle} leaked into the markup: {markup}"
    );
}

fn assert_scripts_stay_off(markup: &str, needles: &[&str]) {
    for needle in needles {
        assert_absent(markup, needle);
    }
}

fn assert_output_total(markup: &str, total: u128) {
    let paragraph = format!("<p class=\"output-total\">Output total {total}</p>");
    assert_eq!(
        markup.matches(&paragraph).count(),
        1,
        "expected one {paragraph} in {markup}"
    );
    let table_end = markup.find("</table>").expect("transaction table");
    let at = markup.find(&paragraph).expect("output total paragraph");
    assert!(
        table_end < at,
        "output total is not after the transaction table: {markup}"
    );
}

#[test]
fn output_total_is_the_sum_of_output_values() {
    let body = tx(
        &input("SUMSIGHEXDEAD", "SUMSIGASMDEAD"),
        &format!(
            "{},{}",
            output("SUMOUTHEXA", "SUMOUTASMA", Some(100), None),
            output("SUMOUTHEXB", "SUMOUTASMB", Some(50), None)
        ),
    );
    let markup = transaction_row_markup(&body);
    assert_output_total(&markup, 150);
    assert_scripts_stay_off(
        &markup,
        &[
            "SUMSIGHEXDEAD",
            "SUMSIGASMDEAD",
            "SUMOUTHEXA",
            "SUMOUTASMA",
            "SUMOUTHEXB",
            "SUMOUTASMB",
        ],
    );
}

#[test]
fn explicit_zero_values_paint_output_total_zero() {
    let body = tx(
        &input("ZERO2SIGHEXDEAD", "ZERO2SIGASMDEAD"),
        &format!(
            "{},{}",
            output("ZERO2OUTHEXA", "ZERO2OUTASMA", Some(0), None),
            output("ZERO2OUTHEXB", "ZERO2OUTASMB", Some(0), None)
        ),
    );
    let markup = transaction_row_markup(&body);
    assert_output_total(&markup, 0);
    assert_scripts_stay_off(
        &markup,
        &[
            "ZERO2SIGHEXDEAD",
            "ZERO2SIGASMDEAD",
            "ZERO2OUTHEXA",
            "ZERO2OUTASMA",
            "ZERO2OUTHEXB",
            "ZERO2OUTASMB",
        ],
    );
}

#[test]
fn one_output_of_zero_paints_output_total_zero() {
    let body = tx(
        &input("ONEZSIGHEXDEAD", "ONEZSIGASMDEAD"),
        &output("ONEZOUTHEXDEAD", "ONEZOUTASMDEAD", Some(0), None),
    );
    let markup = transaction_row_markup(&body);
    assert_output_total(&markup, 0);
    assert_scripts_stay_off(
        &markup,
        &[
            "ONEZSIGHEXDEAD",
            "ONEZSIGASMDEAD",
            "ONEZOUTHEXDEAD",
            "ONEZOUTASMDEAD",
        ],
    );
}

#[test]
fn missing_output_value_omits_the_total() {
    let body = tx(
        &input("MISSSIGHEXDEAD", "MISSSIGASMDEAD"),
        &format!(
            "{},{}",
            output("MISSOUTHEXA", "MISSOUTASMA", Some(40), None),
            output("MISSOUTHEXB", "MISSOUTASMB", None, None)
        ),
    );
    let markup = transaction_row_markup(&body);
    assert_absent(&markup, "class=\"output-total\"");
    assert_absent(&markup, "Output total 40");
    assert_absent(&markup, "Output total 0");
    assert_scripts_stay_off(
        &markup,
        &[
            "MISSSIGHEXDEAD",
            "MISSSIGASMDEAD",
            "MISSOUTHEXA",
            "MISSOUTASMA",
            "MISSOUTHEXB",
            "MISSOUTASMB",
        ],
    );
}

#[test]
fn valuecommitment_is_not_an_amount() {
    let body = tx(
        &input("COMMITSIGHEXDEAD", "COMMITSIGASMDEAD"),
        &output(
            "COMMITOUTHEXDEAD",
            "COMMITOUTASMDEAD",
            None,
            Some("COMMITHEXDEAD"),
        ),
    );
    let markup = transaction_row_markup(&body);
    assert_absent(&markup, "output-total");
    assert_absent(&markup, "Output total 0");
    assert_absent(&markup, "COMMITHEXDEAD");
    assert_scripts_stay_off(
        &markup,
        &[
            "COMMITSIGHEXDEAD",
            "COMMITSIGASMDEAD",
            "COMMITOUTHEXDEAD",
            "COMMITOUTASMDEAD",
        ],
    );
}

#[test]
fn empty_vout_paints_output_total_zero() {
    let body = tx(&input("EMPTYSIGHEXDEAD", "EMPTYSIGASMDEAD"), "");
    let markup = transaction_row_markup(&body);
    assert_output_total(&markup, 0);
    assert_scripts_stay_off(&markup, &["EMPTYSIGHEXDEAD", "EMPTYSIGASMDEAD"]);
}

#[test]
fn output_total_add_does_not_wrap() {
    let body = tx(
        &input("WRAPSIGHEXDEAD", "WRAPSIGASMDEAD"),
        &format!(
            "{},{}",
            output("WRAPOUTHEXA", "WRAPOUTASMA", Some(u64::MAX), None),
            output("WRAPOUTHEXB", "WRAPOUTASMB", Some(u64::MAX), None)
        ),
    );
    let markup = transaction_row_markup(&body);
    let sum = u64::MAX as u128 + u64::MAX as u128;
    assert_output_total(&markup, sum);
    assert_absent(
        &markup,
        &format!("<p class=\"output-total\">Output total {}</p>", u64::MAX),
    );
    assert_absent(&markup, "<p class=\"output-total\">Output total 0</p>");
    assert_scripts_stay_off(
        &markup,
        &[
            "WRAPSIGHEXDEAD",
            "WRAPSIGASMDEAD",
            "WRAPOUTHEXA",
            "WRAPOUTASMA",
            "WRAPOUTHEXB",
            "WRAPOUTASMB",
        ],
    );
}

/// Whole line `Output total 150` from no-query `GET /tx/:txid`.
///
/// GPUI `transaction_screen_paints_output_total_sentence` requires one line
/// equal to `Output total 150` for output values 100 and 50. The paragraph
/// stays after the table. A missing value omits the line.
#[test]
fn output_values_paint_output_total_as_its_own_line() {
    const LINE: &str = "Output total 150";
    let body = tx(
        &input("SUMSIGHEXDEAD", "SUMSIGASMDEAD"),
        &format!(
            "{},{}",
            output("SUMOUTHEXA", "SUMOUTASMA", Some(100), None),
            output("SUMOUTHEXB", "SUMOUTASMB", Some(50), None)
        ),
    );
    let markup = transaction_row_markup(&body);
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert_output_total(&markup, 150);
    assert_scripts_stay_off(
        &markup,
        &["SUMSIGHEXDEAD", "SUMSIGASMDEAD", "SUMOUTHEXA", "SUMOUTASMA"],
    );

    let missing = tx(
        &input("MISSSIGHEXDEAD", "MISSSIGASMDEAD"),
        &output("MISSOUTHEXA", "MISSOUTASMA", None, None),
    );
    assert!(
        !transaction_row_markup(&missing)
            .lines()
            .any(|line| line.starts_with("Output total ")),
        "a missing value painted an output total line"
    );
}
