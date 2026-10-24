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
