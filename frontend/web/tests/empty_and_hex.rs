//! Whitespace inside `[]` is still an empty list.
//! A failed transaction hex body is a notice, not `<pre class="tx-hex">`.

use splora_web::{
    address_rows_markup, merkle_proof_markup, outspends_markup, recent_rows_markup,
    transaction_hex_markup, transaction_row_markup,
};

const TX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

const NO_TXS: &str = "<p class=\"empty\">There are no transactions.</p>";
const NO_UTXOS: &str = "<p class=\"empty\">There are no unspent outputs.</p>";

fn assert_not_an_empty_sentence(body: &str) {
    let txs = address_rows_markup("", body, "");
    assert!(
        !txs.contains("There are no"),
        "address transactions invented a sentence for {body:?}: {txs}"
    );
    let utxos = address_rows_markup("", "", body);
    assert!(
        !utxos.contains("There are no"),
        "address utxos invented a sentence for {body:?}: {utxos}"
    );
    let recent = recent_rows_markup(body);
    assert!(
        !recent.contains("There are no"),
        "mempool recent invented a sentence for {body:?}: {recent}"
    );
}

#[test]
fn whitespace_inside_empty_brackets_is_the_empty_list_sentence() {
    for body in ["[ ]", "[\n]", "[ \n\t ]", " [\n] "] {
        assert_eq!(
            address_rows_markup("", body, ""),
            NO_TXS,
            "address transactions {body:?}"
        );
        assert_eq!(
            address_rows_markup("", "", body),
            NO_UTXOS,
            "address utxos {body:?}"
        );
        assert_eq!(recent_rows_markup(body), NO_TXS, "mempool recent {body:?}");
    }

    assert_eq!(address_rows_markup("", "[]", ""), NO_TXS);
    assert_eq!(address_rows_markup("", "", "[]"), NO_UTXOS);
    assert_eq!(
        address_rows_markup("", "[]", "[]"),
        format!("{NO_TXS}{NO_UTXOS}")
    );
    assert_eq!(recent_rows_markup("[]"), NO_TXS);
    assert_eq!(address_rows_markup("", "", ""), "");
    assert_eq!(recent_rows_markup(""), "");

    for body in ["", "not-json", "[1]", "{}", "[1,2]", "[{}]", "[[]]"] {
        assert_not_an_empty_sentence(body);
    }

    let txs = format!(
        r#"[{{"txid":"{TX}","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}]"#
    );
    let rendered_txs = address_rows_markup("", &txs, "");
    assert!(rendered_txs.contains(TX), "{rendered_txs}");
    assert!(!rendered_txs.contains("There are no"), "{rendered_txs}");

    let utxos = format!(r#"[{{"txid":"{TX}","vout":0,"value":1,"status":{{"confirmed":false}}}}]"#);
    let rendered_utxos = address_rows_markup("", "", &utxos);
    assert!(rendered_utxos.contains(TX), "{rendered_utxos}");
    assert!(!rendered_utxos.contains("There are no"), "{rendered_utxos}");

    let recent = format!(r#"[{{"txid":"{TX}","fee":1,"vsize":1,"value":1}}]"#);
    let rendered_recent = recent_rows_markup(&recent);
    assert!(rendered_recent.contains(TX), "{rendered_recent}");
    assert!(
        !rendered_recent.contains("There are no"),
        "{rendered_recent}"
    );
}

#[test]
fn failed_hex_is_a_notice_and_real_hex_stays_in_pre() {
    let failure = "Transaction not found\nindexer error GET /tx/abc/hex";
    let hex_html = transaction_hex_markup(failure);
    assert_eq!(hex_html, outspends_markup(failure));
    assert_eq!(hex_html, merkle_proof_markup(failure));
    assert_eq!(
        hex_html,
        "<p class=\"notice\">Transaction not found</p><p class=\"notice\">indexer error GET /tx/abc/hex</p>"
    );
    assert!(!hex_html.contains("tx-hex"), "{hex_html}");
    assert!(!hex_html.contains("<pre"), "{hex_html}");

    assert_eq!(
        transaction_hex_markup("  deadbeef\n"),
        "<pre class=\"tx-hex\">deadbeef</pre>"
    );
    assert_eq!(transaction_hex_markup(""), "");
    assert_eq!(transaction_hex_markup(" \n\t"), "");

    let row = transaction_row_markup(&format!(
        r#"{{"txid":"{TX}","version":1,"locktime":0,"vin":[],"vout":[],"size":1,"weight":4,"sigops":0,"fee":1,"status":{{"confirmed":false}}}}"#
    ));
    assert!(!row.contains("tx-hex"), "{row}");
    assert!(!row.contains("deadbeef"), "{row}");
    assert!(row.contains(TX), "{row}");
}
