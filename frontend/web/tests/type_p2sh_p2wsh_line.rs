//! Whole line `Type P2SH-P2WSH` on the address page from `GET /address/:addr`.
//!
//! GPUI `address_screen_paints_p2sh_p2wsh_badge` requires one line whose entire
//! text equals `Type P2SH-P2WSH`. The address is `3GcrZrbUuvE4UtUdSbKTXcRnTqmfMdyMAC`.
//! The address JSON has no type field. The badge is the `220020` spend.
//! The address-type paragraph stays. That paragraph is a longer line.
//! It is not this sentence. The words sit outside the paragraph tag.
//! The wrapped page has no line equal to `Type P2SH` and does not contain
//! `P2SH-P2WPKH`. A `160014` spend stays `Type P2SH-P2WPKH` and does not paint
//! this sentence. A no-spend address, an empty witness, a scriptsig that does
//! not start with `220020`, and a spend of another address omit this sentence.
//! A `bc1q` pay-to-witness-pubkey-hash address stays `Type P2WPKH`. A longer
//! `bc1q` stays `Type P2WSH`. A `bc1p` stays `Type P2TR`. A leading `1` stays
//! `Type P2PKH`. None of those paint `P2SH-P2WSH`. A failed address load keeps
//! `Error loading address data.` and `Invalid Bitcoin address`. The scriptsig
//! and the witness stay off the page.

use splora_web::{address_rows_markup, indexer_failure_text};

const LINE: &str = "Type P2SH-P2WSH";
const PLAIN: &str = "Type P2SH";
const WRAPPED_PKH: &str = "Type P2SH-P2WPKH";
/// `transactions-random.json` prevout for the `220020` scriptsig.
const P2SH_P2WSH_ADDR: &str = "3GcrZrbUuvE4UtUdSbKTXcRnTqmfMdyMAC";
/// Same file, a different mainnet address whose first character is `3`.
const OTHER_P2SH: &str = "3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB";
/// Same file, the `160014` wrapped spend.
const P2SH_P2WPKH_ADDR: &str = "375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH";
const SCRIPTSIG_P2WSH: &str =
    "220020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75";
const SCRIPTSIG_P2WPKH: &str = "1600149b27f072e4b972927c445d1946162a550b0914d8";
const WITNESS_0: &str = "303f021c65aee6696e80be6e14545cfd64b44f17b0514c150eefdb090c0f0bd9021f3fef4aa95c252a225622aba99e4d5af5a6fe40d177acd593e64cf2f8557ccc03";
const WITNESS_1: &str = "03b55c6f0749e0f3e2caeca05f68e3699f1b3c62a550730f704985a6a9aae437a1";
const WITNESS_2: &str = "76a914db865fd920959506111079995f1e4017b489bfe38763ac6721024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c7c820120876475527c2103443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca52ae67a91446c3747322b220fdb925c9802f0e949c1feab99988ac6868";
const PKH_WITNESS_0: &str = "3040021c23902a01d4c5cff2c33c8bdb778a5aadea78a9a0d6d4db60aaa0fba1022069237d9dbf2db8cff9c260ba71250493682d01a746f4a45c5c7ea386e56d2bc902";
const PKH_WITNESS_1: &str = "0240187acd3e2fd3d8e1acffefa85907b6550730c24f78dfd3301c829fc4daf3cc";
const P2PKH_ADDR: &str = "1stAprEZapjCYGUACUcXohQqSHF9MU5Kj";
const P2WPKH_ADDR: &str = "bc1q8yj0herd4r4yxszw3nkfvt53433thk0f5qst4g";
const P2WSH_ADDR: &str = "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3";
const P2TR_ADDR: &str = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";
const PARAGRAPH: &str = "<p class=\"address-type\"><span class=\"address-type-label\">Type</span><span class=\"badge bg-primary\"><span>P2SH-P2WSH</span></span></p>";
const LOAD_ERROR: &str = "Error loading address data.";
const INDEXER_BODY: &str = "Invalid Bitcoin address";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn witness_items() -> String {
    format!(r#"["{WITNESS_0}","{WITNESS_1}","{WITNESS_2}"]"#)
}

fn pkh_witness() -> String {
    format!(r#"["{PKH_WITNESS_0}","{PKH_WITNESS_1}"]"#)
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
        r#"{{"txid":"cb9b47ac04023b29fb633a8ef04af351ac9fd74c57c9a2163f683516274767e3","vout":3,"prevout":{{"scriptpubkey":"a914a3c0698f2300c7b2e8107d4c9c988e642110039087","scriptpubkey_asm":"OP_HASH160 OP_PUSHBYTES_20 a3c0698f2300c7b2e8107d4c9c988e6421100390 OP_EQUAL","scriptpubkey_type":"{script_type}","scriptpubkey_address":"{address}","value":3220}},"scriptsig":"{scriptsig}","scriptsig_asm":"{asm}","is_coinbase":false,"sequence":3735928559{witness_field}}}"#
    )
}

fn txs(vins: &[String]) -> String {
    let vins = vins.join(",");
    format!(
        r#"[{{"txid":"cb9b47ac04023b29fb633a8ef04af351ac9fd74c57c9a2163f683516274767e3","version":1,"locktime":0,"vin":[{vins}],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1}}]"#
    )
}

/// A vin that does not spend `P2SH_P2WSH_ADDR` comes first. The matching spend is second.
fn matching_txs() -> String {
    let other = vin_json(OTHER_P2SH, "p2sh", "", Some("[]"));
    let spend = vin_json(
        P2SH_P2WSH_ADDR,
        "p2sh",
        SCRIPTSIG_P2WSH,
        Some(&witness_items()),
    );
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
        !markup.contains("P2SH-P2WSH"),
        "{why} painted P2SH-P2WSH: {markup}"
    );
}

#[test]
fn address_page_paints_type_p2sh_p2wsh_as_its_own_line() {
    assert!(
        SCRIPTSIG_P2WSH.starts_with("220020"),
        "fixture scriptsig does not start with 220020"
    );
    assert!(
        SCRIPTSIG_P2WPKH.starts_with("160014"),
        "fixture scriptsig does not start with 160014"
    );
    let body = address_body(P2SH_P2WSH_ADDR);
    assert!(
        !body.contains("\"type\""),
        "address JSON must not carry a type field: {body}"
    );

    let markup = page(P2SH_P2WSH_ADDR, &matching_txs());
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
        "no line equals Type P2SH-P2WSH: {markup}"
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
        !markup.contains("P2SH-P2WPKH"),
        "the wrapped script spend painted P2SH-P2WPKH: {markup}"
    );
    assert!(
        !markup.contains(SCRIPTSIG_P2WSH),
        "the scriptsig was printed: {markup}"
    );
    assert!(
        !markup.contains(SCRIPTSIG_P2WPKH),
        "the other scriptsig was printed: {markup}"
    );
    assert!(
        !markup.contains(WITNESS_0) && !markup.contains(WITNESS_1) && !markup.contains(WITNESS_2),
        "the witness was printed: {markup}"
    );
    assert!(
        !markup.contains(PKH_WITNESS_0) && !markup.contains(PKH_WITNESS_1),
        "the witness was printed: {markup}"
    );
    for absent in ["script hex", "script asm", "witness", "tooltip"] {
        assert!(
            !markup.contains(absent),
            "{absent} was painted on the p2sh-p2wsh address: {markup}"
        );
    }

    let wrapped_pubkey = page(
        P2SH_P2WPKH_ADDR,
        &txs(&[vin_json(
            P2SH_P2WPKH_ADDR,
            "p2sh",
            SCRIPTSIG_P2WPKH,
            Some(&pkh_witness()),
        )]),
    );
    omits_line(&wrapped_pubkey, "the 160014 spend");
    assert!(
        wrapped_pubkey.lines().any(|line| line == WRAPPED_PKH),
        "the 160014 spend dropped {WRAPPED_PKH}: {wrapped_pubkey}"
    );
    assert!(
        !wrapped_pubkey.lines().any(|line| line == PLAIN),
        "the 160014 spend painted {PLAIN}: {wrapped_pubkey}"
    );

    let idle = page(P2SH_P2WSH_ADDR, "");
    omits_line(&idle, "the no-spend address");
    assert_eq!(
        idle.lines().filter(|line| *line == PLAIN).count(),
        1,
        "the no-spend address dropped {PLAIN}: {idle}"
    );

    let empty_list = page(P2SH_P2WSH_ADDR, "[]");
    omits_line(&empty_list, "an empty transaction list");

    let empty_witness = page(
        P2SH_P2WSH_ADDR,
        &txs(&[vin_json(
            P2SH_P2WSH_ADDR,
            "p2sh",
            SCRIPTSIG_P2WSH,
            Some("[]"),
        )]),
    );
    omits_line(&empty_witness, "an empty witness");
    assert_eq!(
        empty_witness.lines().filter(|line| *line == PLAIN).count(),
        1,
        "an empty witness dropped {PLAIN}: {empty_witness}"
    );

    let omitted_witness = page(
        P2SH_P2WSH_ADDR,
        &txs(&[vin_json(P2SH_P2WSH_ADDR, "p2sh", SCRIPTSIG_P2WSH, None)]),
    );
    omits_line(&omitted_witness, "an omitted witness");

    let other_scriptsig = page(
        P2SH_P2WSH_ADDR,
        &txs(&[vin_json(
            P2SH_P2WSH_ADDR,
            "p2sh",
            "",
            Some(&witness_items()),
        )]),
    );
    omits_line(
        &other_scriptsig,
        "a scriptsig that does not start with 220020",
    );

    let foreign = page(
        P2SH_P2WSH_ADDR,
        &txs(&[vin_json(
            OTHER_P2SH,
            "p2sh",
            SCRIPTSIG_P2WSH,
            Some(&witness_items()),
        )]),
    );
    omits_line(&foreign, "a spend of another address");
    assert!(
        !foreign.contains(SCRIPTSIG_P2WSH),
        "the scriptsig was painted: {foreign}"
    );

    let short = page(P2WPKH_ADDR, &matching_txs());
    assert!(
        short.lines().any(|line| line == "Type P2WPKH"),
        "the shorter bc1q address dropped Type P2WPKH: {short}"
    );
    omits_line(&short, "the shorter bc1q address");

    let script = page(P2WSH_ADDR, &matching_txs());
    assert!(
        script.lines().any(|line| line == "Type P2WSH"),
        "the longer bc1q address dropped Type P2WSH: {script}"
    );
    omits_line(&script, "the longer bc1q address");

    let taproot = page(P2TR_ADDR, &matching_txs());
    assert!(
        taproot.lines().any(|line| line == "Type P2TR"),
        "the longer bc1p address dropped Type P2TR: {taproot}"
    );
    omits_line(&taproot, "the longer bc1p address");

    let pubkey = page(P2PKH_ADDR, &matching_txs());
    assert!(
        pubkey.lines().any(|line| line == "Type P2PKH"),
        "the p2pkh address dropped Type P2PKH: {pubkey}"
    );
    omits_line(&pubkey, "the p2pkh address");

    let failed_body =
        indexer_failure_text("GET", &format!("/address/{P2SH_P2WSH_ADDR}"), INDEXER_BODY);
    let failed = address_rows_markup(&failed_body, &matching_txs(), "");
    assert!(
        failed.contains(LOAD_ERROR),
        "Error loading address data. is absent: {failed}"
    );
    assert!(
        failed.contains(INDEXER_BODY),
        "Invalid Bitcoin address was replaced or dropped: {failed}"
    );
    omits_line(&failed, "a failed address load");
}
