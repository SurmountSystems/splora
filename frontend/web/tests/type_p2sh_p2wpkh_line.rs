//! Whole line `Type P2SH-P2WPKH` on the address page from `GET /address/:addr`.
//!
//! GPUI `address_screen_paints_p2sh_p2wpkh_badge` requires one line whose entire
//! text equals `Type P2SH-P2WPKH`. The address is `375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH`.
//! The address JSON has no type field. The badge is the `160014` spend.
//! The address-type paragraph stays. That paragraph is a longer line.
//! It is not this sentence. The words sit outside the paragraph tag.
//! The wrapped page has no line equal to `Type P2SH` and does not contain
//! `P2SH-P2WSH`. A `220020` spend, a no-spend address, an empty witness,
//! a scriptsig that does not start with `160014`, and a spend of another
//! address omit this sentence. The scriptsig and the witness stay off the page.

use splora_web::address_rows_markup;

const LINE: &str = "Type P2SH-P2WPKH";
const PLAIN: &str = "Type P2SH";
/// `transactions-random.json` prevout for the `160014` scriptsig.
const ADDR: &str = "375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH";
/// Same file, a different mainnet address whose first character is `3`.
const OTHER_P2SH: &str = "3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB";
/// Same file, the `220020` wrapped spend.
const P2SH_P2WSH_ADDR: &str = "3GcrZrbUuvE4UtUdSbKTXcRnTqmfMdyMAC";
const SCRIPTSIG_P2WPKH: &str = "1600149b27f072e4b972927c445d1946162a550b0914d8";
const SCRIPTSIG_P2WSH: &str =
    "220020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75";
const WITNESS_0: &str = "3040021c23902a01d4c5cff2c33c8bdb778a5aadea78a9a0d6d4db60aaa0fba1022069237d9dbf2db8cff9c260ba71250493682d01a746f4a45c5c7ea386e56d2bc902";
const WITNESS_1: &str = "0240187acd3e2fd3d8e1acffefa85907b6550730c24f78dfd3301c829fc4daf3cc";
const PARAGRAPH: &str = "<p class=\"address-type\"><span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2SH-P2WPKH</span></span></p>";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn witness_pair() -> String {
    format!(r#"["{WITNESS_0}","{WITNESS_1}"]"#)
}

fn vin_json(address: &str, script_type: &str, scriptsig: &str, witness: Option<&str>) -> String {
    let asm = if scriptsig.starts_with("160014") {
        "OP_PUSHBYTES_22 00149b27f072e4b972927c445d1946162a550b0914d8"
    } else if scriptsig.starts_with("220020") {
        "OP_PUSHBYTES_34 0020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75"
    } else {
        ""
    };
    let witness_field = match witness {
        Some(items) => format!(r#","witness":{items}"#),
        None => String::new(),
    };
    format!(
        r#"{{"txid":"45e1cb33599acb071810ccc801b71bd7610865f5b899492946ab1bfbcb61cad6","vout":2,"prevout":{{"scriptpubkey":"a9143b13a1f71c20c799d86bb624b3898c826d6c82da87","scriptpubkey_asm":"OP_HASH160 OP_PUSHBYTES_20 3b13a1f71c20c799d86bb624b3898c826d6c82da OP_EQUAL","scriptpubkey_type":"{script_type}","scriptpubkey_address":"{address}","value":5139}},"scriptsig":"{scriptsig}","scriptsig_asm":"{asm}","is_coinbase":false,"sequence":141{witness_field}}}"#
    )
}

fn txs(vins: &[String]) -> String {
    let vins = vins.join(",");
    format!(
        r#"[{{"txid":"cb9b47ac04023b29fb633a8ef04af351ac9fd74c57c9a2163f683516274767e3","version":1,"locktime":0,"vin":[{vins}],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1}}]"#
    )
}

/// A vin that does not spend `ADDR` comes first. The matching spend is second.
fn matching_txs() -> String {
    let other = vin_json(OTHER_P2SH, "p2sh", "", Some("[]"));
    let spend = vin_json(ADDR, "p2sh", SCRIPTSIG_P2WPKH, Some(&witness_pair()));
    txs(&[other, spend])
}

fn page(address: &str, txs_json: &str) -> String {
    address_rows_markup(&address_body(address), txs_json, "")
}

fn omits_line(markup: &str, why: &str) {
    assert!(
        !markup.lines().any(|line| line == LINE),
        "{why} painted {LINE}: {markup}"
    );
    assert!(
        !markup.contains("P2SH-P2WPKH"),
        "{why} painted P2SH-P2WPKH: {markup}"
    );
}

#[test]
fn address_page_paints_type_p2sh_p2wpkh_as_its_own_line() {
    assert!(
        SCRIPTSIG_P2WPKH.starts_with("160014"),
        "fixture scriptsig does not start with 160014"
    );
    assert!(
        SCRIPTSIG_P2WSH.starts_with("220020"),
        "fixture scriptsig does not start with 220020"
    );
    let body = address_body(ADDR);
    assert!(
        !body.contains("\"type\""),
        "address JSON must not carry a type field: {body}"
    );

    let markup = page(ADDR, &matching_txs());
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
        "no line equals Type P2SH-P2WPKH: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not painted once: {markup}"
    );
    assert!(
        !markup.lines().any(|line| line == PLAIN),
        "a line equal to {PLAIN} was painted on the wrapped page: {markup}"
    );
    assert!(
        !markup.contains("P2SH-P2WSH"),
        "the wrapped pubkey spend painted P2SH-P2WSH: {markup}"
    );
    assert!(
        !markup.contains(SCRIPTSIG_P2WPKH),
        "the scriptsig was printed: {markup}"
    );
    assert!(
        !markup.contains(WITNESS_0) && !markup.contains(WITNESS_1),
        "the witness was printed: {markup}"
    );

    let wrapped = page(
        ADDR,
        &txs(&[vin_json(
            ADDR,
            "p2sh",
            SCRIPTSIG_P2WSH,
            Some(&witness_pair()),
        )]),
    );
    omits_line(&wrapped, "the 220020 spend");
    assert!(
        !SCRIPTSIG_P2WSH.starts_with("160014"),
        "the 220020 scriptsig must not start with 160014"
    );

    let idle = page(ADDR, "");
    omits_line(&idle, "the no-spend address");
    assert_eq!(
        idle.lines().filter(|line| *line == PLAIN).count(),
        1,
        "the no-spend address dropped {PLAIN}: {idle}"
    );

    let empty_list = page(ADDR, "[]");
    omits_line(&empty_list, "an empty transaction list");

    let empty_witness = page(
        ADDR,
        &txs(&[vin_json(ADDR, "p2sh", SCRIPTSIG_P2WPKH, Some("[]"))]),
    );
    omits_line(&empty_witness, "an empty witness");

    let omitted_witness = page(
        ADDR,
        &txs(&[vin_json(ADDR, "p2sh", SCRIPTSIG_P2WPKH, None)]),
    );
    omits_line(&omitted_witness, "an omitted witness");

    let other_scriptsig = page(
        ADDR,
        &txs(&[vin_json(ADDR, "p2sh", "", Some(&witness_pair()))]),
    );
    omits_line(
        &other_scriptsig,
        "a scriptsig that does not start with 160014",
    );

    let foreign = page(
        ADDR,
        &txs(&[vin_json(
            OTHER_P2SH,
            "p2sh",
            SCRIPTSIG_P2WPKH,
            Some(&witness_pair()),
        )]),
    );
    omits_line(&foreign, "a spend of another address");

    let other_page = page(
        P2SH_P2WSH_ADDR,
        &txs(&[vin_json(
            P2SH_P2WSH_ADDR,
            "p2sh",
            SCRIPTSIG_P2WSH,
            Some(&witness_pair()),
        )]),
    );
    omits_line(&other_page, "the 220020 address");
}
