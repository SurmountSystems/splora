//! Whole line `Type MULTISIG` on a transaction output from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_multisig_output_type` requires one line
//! whose entire text is `Type MULTISIG` when `vout.scriptpubkey_type` is
//! `multisig`. The Type span and the uppercase value span stay. A different
//! script type omits the line. The address warning page still has no line
//! equal to `Type MULTISIG`. That page is the warning `Multisig 2 of 3`,
//! not this output script type.

use splora_web::{address_rows_markup, transaction_row_markup};

const LINE: &str = "Type MULTISIG";
const TYPE_LABEL: &str = "<span class=\"output-type-label\">Type</span>";
const TYPE_VALUE: &str = "<span class=\"output-type\">MULTISIG</span>";

fn tx(kind: &str) -> String {
    format!(
        r#"{{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}}],"vout":[{{"scriptpubkey":"51","scriptpubkey_asm":"OP_1","scriptpubkey_type":"{kind}","value":1}}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn address_page_with_multisig_output() -> String {
    let stats = r#"{"address":"3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB","chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#;
    let txs = r#"[{"txid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":1,"locktime":0,"vin":[{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":0,"prevout":null,"scriptsig":"","scriptsig_asm":"","is_coinbase":false,"sequence":1}],"vout":[{"scriptpubkey":"51","scriptpubkey_asm":"OP_1","scriptpubkey_type":"multisig","value":1}],"size":10,"weight":40,"sigops":0,"fee":1}]"#;
    address_rows_markup(stats, txs, "")
}

#[test]
fn multisig_output_paints_type_multisig_as_its_own_line() {
    let markup = transaction_row_markup(&tx("multisig"));
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "scriptpubkey_type multisig did not paint the exact line Type MULTISIG: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "Type MULTISIG was inside a span: {markup}"
    );
    assert!(
        markup.contains(TYPE_LABEL),
        "the Type span was dropped: {markup}"
    );
    assert!(
        markup.contains(TYPE_VALUE),
        "the uppercase multisig span was dropped: {markup}"
    );

    let other = transaction_row_markup(&tx("p2pkh"));
    assert!(
        !other.lines().any(|line| line == LINE),
        "a non-multisig output painted {LINE}: {other}"
    );

    let address = address_page_with_multisig_output();
    assert!(
        !address.lines().any(|line| line == LINE),
        "the address warning page painted {LINE}: {address}"
    );
}
