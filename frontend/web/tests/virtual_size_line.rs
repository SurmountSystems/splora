//! Whole line `Virtual size 1` on the transaction row from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_virtual_size_sentence` requires one line
//! whose entire text equals `Virtual size 1` when weight is 4. Virtual size
//! is `weight / 4`, integer division. The virtual-size cell stays. That cell
//! is a longer line. It is not this sentence.

use splora_web::transaction_row_markup;

const LINE: &str = "Virtual size 1";

fn tx(weight: u32) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":{weight},"sigops":0,"fee":1}}"#
    )
}

#[test]
fn virtual_size_paints_virtual_size_1_as_its_own_line() {
    let markup = transaction_row_markup(&tx(4));
    let cell = "<td class=\"virtual-size\">Virtual size 1</td>";
    assert!(
        markup.contains(cell),
        "the virtual-size cell was dropped: {markup}"
    );
    assert!(
        markup
            .lines()
            .filter(|line| line.contains(cell))
            .all(|line| line != LINE),
        "the virtual-size cell line is the whole sentence: {markup}"
    );
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
}

/// Whole line `virtual size 0` from no-query `GET /tx/:txid`.
///
/// GPUI `transaction_screen_paints_virtual_size` requires one line whose
/// entire text equals `virtual size 0` when weight is present and
/// `weight / 4` is 0. Weight 0 is that quotient. Weight 3 is also 0 and
/// does not round up. The capital cell `Virtual size 0` stays and is not
/// this line. A missing key or JSON null omits the line and is not zero.
/// Weight 5 is virtual size 1 and does not paint `virtual size 0`, and it
/// does not round up to `virtual size 2`. Lowercase `virtual size 1` stays
/// off that row. Weight 0 and weight 3 still omit that line.
/// Zero virtual size does not gain the Details button.
fn tx_weight_json(weight_field: &str) -> String {
    let weight_line = if weight_field.is_empty() {
        String::new()
    } else {
        format!("{weight_field},")
    };
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"76a914bb88ac","scriptpubkey_asm":"OP_DUP","scriptpubkey_type":"p2pkh","value":1}}],"size":10,{weight_line}"sigops":0,"fee":1}}"#
    )
}

#[test]
fn weight_zero_paints_virtual_size_0_as_its_own_line() {
    const ZERO: &str = "virtual size 0";
    let markup = transaction_row_markup(&tx(0));
    assert!(
        markup.lines().any(|line| line == ZERO),
        "weight 0 omitted {ZERO}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == ZERO).count(),
        1,
        "{ZERO} was not painted once: {markup}"
    );
    assert!(
        markup.contains("<td class=\"virtual-size\">Virtual size 0</td>"),
        "the virtual-size cell was dropped: {markup}"
    );
    assert!(
        markup
            .lines()
            .filter(|line| line.contains("virtual-size"))
            .all(|line| line != ZERO),
        "the virtual-size cell line is {ZERO}: {markup}"
    );
    assert!(
        !markup.contains("btn-outline-info"),
        "weight 0 painted the Details button: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "Details"),
        "weight 0 painted Details: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "virtual size 1"),
        "weight 0 painted virtual size 1: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == "virtual size 2"),
        "weight 0 painted virtual size 2: {markup}"
    );
    assert!(
        !markup.contains("fee-rate"),
        "weight 0 painted a fee rate: {markup}"
    );

    let three = transaction_row_markup(&tx(3));
    assert!(
        three.lines().any(|line| line == ZERO),
        "weight 3 did not paint {ZERO}: {three}"
    );
    assert!(
        !three.lines().any(|line| line == "virtual size 1"),
        "weight 3 rounded up: {three}"
    );
    assert!(
        !three.contains("btn-outline-info"),
        "weight 3 painted the Details button: {three}"
    );

    let five = transaction_row_markup(&tx(5));
    assert!(
        !five.lines().any(|line| line == ZERO),
        "weight 5 painted {ZERO}: {five}"
    );
    assert!(
        !five.lines().any(|line| line == "virtual size 1"),
        "weight 5 painted virtual size 1: {five}"
    );
    assert_eq!(
        five.lines()
            .filter(|line| *line == "virtual size 1")
            .count(),
        0,
        "virtual size 1 was painted: {five}"
    );
    assert!(
        !five.lines().any(|line| line == "virtual size 2"),
        "weight 5 rounded up: {five}"
    );

    let absent = transaction_row_markup(&tx_weight_json(""));
    assert!(
        !absent.lines().any(|line| line == ZERO),
        "missing weight painted {ZERO}: {absent}"
    );
    assert!(
        !absent.contains("virtual size 0"),
        "missing weight was painted as 0: {absent}"
    );

    let null = transaction_row_markup(&tx_weight_json(r#""weight":null"#));
    assert!(
        !null.lines().any(|line| line == ZERO),
        "null weight painted {ZERO}: {null}"
    );
    assert!(
        !null.contains("virtual size 0"),
        "null weight was painted as 0: {null}"
    );
}

/// Whole lines `virtual size 10` and `virtual size 2` from no-query `GET /tx/:txid`.
///
/// Integer virtual size is `weight / 4`. Weight 40 is 10. Weight 8 is 2.
/// Each paints that lowercase sentence as one whole line. The capital cell
/// `Virtual size N` stays. The whole line `virtual size 1` stays off.
/// Weight 0 and weight 5 still forbid `virtual size 2` in the test above.
#[test]
fn weight_forty_and_eight_paint_lowercase_virtual_size_lines() {
    let ten = transaction_row_markup(&tx(40));
    assert!(
        ten.lines().any(|line| line == "virtual size 10"),
        "weight 40 omitted virtual size 10: {ten}"
    );
    assert_eq!(
        ten.lines()
            .filter(|line| *line == "virtual size 10")
            .count(),
        1,
        "virtual size 10 was not painted once: {ten}"
    );
    assert!(
        ten.contains("<td class=\"virtual-size\">Virtual size 10</td>"),
        "weight 40 dropped the Virtual size 10 cell: {ten}"
    );
    assert!(
        ten.lines()
            .filter(|line| line.contains("virtual-size"))
            .all(|line| line != "virtual size 10"),
        "the virtual-size cell line is virtual size 10: {ten}"
    );
    assert!(
        !ten.lines().any(|line| line == "virtual size 1"),
        "weight 40 painted virtual size 1: {ten}"
    );

    let two = transaction_row_markup(&tx(8));
    assert!(
        two.lines().any(|line| line == "virtual size 2"),
        "weight 8 omitted virtual size 2: {two}"
    );
    assert_eq!(
        two.lines().filter(|line| *line == "virtual size 2").count(),
        1,
        "virtual size 2 was not painted once: {two}"
    );
    assert!(
        two.contains("<td class=\"virtual-size\">Virtual size 2</td>"),
        "weight 8 dropped the Virtual size 2 cell: {two}"
    );
    assert!(
        !two.lines().any(|line| line == "virtual size 1"),
        "weight 8 painted virtual size 1: {two}"
    );
}
