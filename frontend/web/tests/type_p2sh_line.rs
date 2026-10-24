//! Whole line `Type P2SH` on the address page from `GET /address/:addr`.
//!
//! GPUI `address_screen_paints_p2sh_badge` requires one line whose entire
//! text equals `Type P2SH`. The address is `3Bmb9Jig8A5kHdDSxvDZ6eryj3AXd3swuJ`.
//! Its first character is `3`. The remainder is 33 base58 characters.
//! The address JSON has no type field. The badge is the address string.
//! The address-type paragraph stays. That paragraph is a longer line.
//! It is not this sentence. The words sit outside the paragraph tag.
//! An empty transaction list keeps `spend_rewrote_p2sh` false.
//! A rewritten spend omits the line. The no-spend page does not contain
//! `P2SH-P2WPKH` or `P2SH-P2WSH`, and no line equals `Type P2SH-P2WPKH`.
//! A shorter `bc1q` stays `Type P2WPKH` and does not paint `Type P2SH`.
//! A longer `bc1q` stays `Type P2WSH`. A `bc1p` stays `Type P2TR`.
//! A leading `1` stays `Type P2PKH`. A failed address load keeps
//! `Error loading address data.` and does not show this badge.

use splora_web::{address_rows_markup, indexer_failure_text};

const LINE: &str = "Type P2SH";
const WRAPPED_LINE: &str = "Type P2SH-P2WPKH";
const ADDR: &str = "3Bmb9Jig8A5kHdDSxvDZ6eryj3AXd3swuJ";
const WPKH: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
const WSH: &str = "bc1qjasf9z3h7w3jspkhtgatgpyvvzgpa2wwd2lr0eh5tx44reyn2k7sfc27a4";
const TR: &str = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
const PKH: &str = "1stAprEZapjCYGUACUcXohQqSHF9MU5Kj";
const PARAGRAPH: &str = "<p class=\"address-type\"><span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2SH</span></span></p>";
const LOAD_ERROR: &str = "Error loading address data.";
const INDEXER_BODY: &str = "Invalid Bitcoin address";

fn stats(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

/// Empty transaction list. `spend_rewrote_p2sh` stays false.
fn paint(body: &str) -> String {
    address_rows_markup(body, "", "")
}

/// A vin spending this address has a nonempty witness and a scriptsig
/// that starts with `160014`, so the plain badge is rewritten away.
fn rewritten_spend(address: &str) -> String {
    format!(
        r#"[{{"txid":"cb9b47ac04023b29fb633a8ef04af351ac9fd74c57c9a2163f683516274767e3","version":1,"locktime":0,"vin":[{{"txid":"45e1cb33599acb071810ccc801b71bd7610865f5b899492946ab1bfbcb61cad6","vout":2,"prevout":{{"scriptpubkey":"a9143b13a1f71c20c799d86bb624b3898c826d6c82da87","scriptpubkey_asm":"OP_HASH160 OP_PUSHBYTES_20 3b13a1f71c20c799d86bb624b3898c826d6c82da OP_EQUAL","scriptpubkey_type":"p2sh","scriptpubkey_address":"{address}","value":5139}},"scriptsig":"1600149b27f072e4b972927c445d1946162a550b0914d8","scriptsig_asm":"OP_PUSHBYTES_22 00149b27f072e4b972927c445d1946162a550b0914d8","is_coinbase":false,"sequence":141,"witness":["3040021c23902a01d4c5cff2c33c8bdb778a5aadea78a9a0d6d4db60aaa0fba1022069237d9dbf2db8cff9c260ba71250493682d01a746f4a45c5c7ea386e56d2bc902","0240187acd3e2fd3d8e1acffefa85907b6550730c24f78dfd3301c829fc4daf3cc"]}}],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1}}]"#
    )
}

#[test]
fn address_page_paints_type_p2sh_as_its_own_line() {
    let rest = ADDR
        .strip_prefix('3')
        .expect("mainnet p2sh address starts with 3");
    assert_eq!(rest.len(), 33, "{ADDR}");
    let body = stats(ADDR);
    assert!(
        !body.contains("\"type\""),
        "address JSON must not carry a type field: {body}"
    );
    let markup = paint(&body);
    assert!(
        markup.contains(PARAGRAPH),
        "the address-type paragraph was dropped: {markup}"
    );
    assert!(
        markup
            .lines()
            .filter(|line| line.contains(PARAGRAPH))
            .all(|line| line != LINE),
        "the address-type paragraph line is the whole sentence: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains("<p")),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == WRAPPED_LINE),
        "a line equal to {WRAPPED_LINE} appeared on the no-spend page: {markup}"
    );
    assert!(
        !markup.contains("P2SH-P2WPKH"),
        "the no-spend page painted P2SH-P2WPKH: {markup}"
    );
    assert!(
        !markup.contains("P2SH-P2WSH"),
        "the no-spend page painted P2SH-P2WSH: {markup}"
    );
    assert!(
        !markup.contains(LOAD_ERROR),
        "a parsed address that returns stats showed the load error: {markup}"
    );
    for absent in ["script hex", "script asm", "witness", "tooltip"] {
        assert!(
            !markup.contains(absent),
            "{absent} was painted on the p2sh address: {markup}"
        );
    }
    assert!(
        !markup.contains("output-type"),
        "the address badge was the output-type span: {markup}"
    );

    let short = paint(&stats(WPKH));
    assert!(
        short.lines().any(|line| line == "Type P2WPKH"),
        "the shorter bc1q address dropped Type P2WPKH: {short}"
    );
    assert!(
        !short.lines().any(|line| line == LINE),
        "the shorter bc1q address painted {LINE}: {short}"
    );

    let script = paint(&stats(WSH));
    assert!(
        script.lines().any(|line| line == "Type P2WSH"),
        "the longer bc1q address dropped Type P2WSH: {script}"
    );
    assert!(
        !script.lines().any(|line| line == LINE),
        "the longer bc1q address painted {LINE}: {script}"
    );

    let taproot = paint(&stats(TR));
    assert!(
        taproot.lines().any(|line| line == "Type P2TR"),
        "the longer bc1p address dropped Type P2TR: {taproot}"
    );
    assert!(
        !taproot.lines().any(|line| line == LINE),
        "the longer bc1p address painted {LINE}: {taproot}"
    );

    let pubkey = paint(&stats(PKH));
    assert!(
        pubkey.lines().any(|line| line == "Type P2PKH"),
        "the p2pkh address dropped Type P2PKH: {pubkey}"
    );
    assert!(
        !pubkey.lines().any(|line| line == LINE),
        "the p2pkh address painted {LINE}: {pubkey}"
    );

    let rewritten = address_rows_markup(&body, &rewritten_spend(ADDR), "");
    assert!(
        rewritten.contains("P2SH-P2WPKH"),
        "the rewritten spend did not paint the wrapped badge: {rewritten}"
    );
    assert!(
        !rewritten.lines().any(|line| line == LINE),
        "a rewritten spend painted {LINE}: {rewritten}"
    );
    assert!(
        !rewritten.contains(PARAGRAPH),
        "a rewritten spend kept the plain address-type paragraph: {rewritten}"
    );

    let empty = paint("");
    assert!(
        !empty.lines().any(|line| line == LINE),
        "an empty body painted {LINE}: {empty}"
    );
    assert!(
        !empty.contains(">P2SH<"),
        "an empty body painted P2SH: {empty}"
    );

    let missing_address = paint(
        r#"{"chain_stats":{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":10,"spent_txo_sum":0},"mempool_stats":{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}"#,
    );
    assert!(
        !missing_address.lines().any(|line| line == LINE),
        "a missing address painted {LINE}: {missing_address}"
    );
    assert!(
        !missing_address.contains(">P2SH<"),
        "a missing address painted P2SH: {missing_address}"
    );

    let failed_body = indexer_failure_text("GET", &format!("/address/{ADDR}"), INDEXER_BODY);
    let failed = paint(&failed_body);
    assert!(
        failed.contains(LOAD_ERROR),
        "Error loading address data. is absent: {failed}"
    );
    assert!(
        failed.contains(INDEXER_BODY),
        "Invalid Bitcoin address was replaced or dropped: {failed}"
    );
    assert!(
        !failed.lines().any(|line| line == LINE),
        "a failed address load showed the P2SH badge: {failed}"
    );
    assert!(
        !failed.lines().any(|line| line.starts_with("Type ")),
        "a failed address load painted a type badge: {failed}"
    );
}
