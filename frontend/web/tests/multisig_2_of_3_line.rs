//! Whole line `Multisig 2 of 3` on the address page.
//!
//! GPUI `address_screen_paints_multisig_badge` requires one line whose entire
//! text equals `Multisig 2 of 3`. The address is
//! `3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB`. The redeem script is the 2-of-3
//! sample. The warning badge paragraph stays. That paragraph is a longer
//! line. It is not this sentence. The words sit outside the paragraph tag.
//! The page has no line equal to `Type MULTISIG`, `Type P2SH-P2WPKH`, or
//! `Type P2SH-P2WSH`. A `160014` spend, a `220020` spend, and a tapscript
//! address omit this sentence.

use splora_web::address_rows_markup;

const LINE: &str = "Multisig 2 of 3";
const TYPE_MULTISIG: &str = "Type MULTISIG";
const TYPE_P2SH_P2WPKH: &str = "Type P2SH-P2WPKH";
const TYPE_P2SH_P2WSH: &str = "Type P2SH-P2WSH";

/// `api-docs-data.ts` mainnet p2sh address for the 2-of-3 redeem script.
const P2SH_ADDR: &str = "3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB";
/// Same file, the other 2-of-3 address. Used as a spend that is not this page.
const OTHER_P2SH: &str = "3GcWrnGFoNzbn1KaiP5czS5xPELdWcgDX2";
/// `transactions-random.json` prevout for the `160014` scriptsig.
const P2SH_P2WPKH_ADDR: &str = "375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH";
/// `transactions-random.json` prevout for the `220020` scriptsig.
const P2SH_P2WSH_ADDR: &str = "3GcrZrbUuvE4UtUdSbKTXcRnTqmfMdyMAC";
/// Same file as the `220020` whole-line fixture, a tapscript address.
const P2TR_ADDR: &str = "bc1pahzjr2vnfqnqwl0h8jps2aa4zqe29wqxn2nagvd5a2z982k7ttdq8ry370";

/// `api-docs-data.ts` `inner_redeemscript_asm` for address `3L4Y...`.
const REDEEM_2_OF_3: &str = "OP_PUSHNUM_2 OP_PUSHBYTES_33 027f2a0df8e86535d08ca3e766a178f90c813d2dd1d55b0166e82518efbffb18c0 OP_PUSHBYTES_33 03556a35844b517e2fc8216701e2e0a64dbcbe62ad420ac6dd73dc79e69efeb695 OP_PUSHBYTES_33 031ef21bd55171032b7aec21ec82932735fb986f1d4d8611feee62ab38acf4a6bb OP_PUSHNUM_3 OP_CHECKMULTISIG";
/// `api-docs-data.ts` scriptsig for the 2-of-3 spend. It does not start with `160014`.
const SCRIPTSIG_2_OF_3: &str = "0048304502210099219ee0cd5da341650078e3c63885b3cc2211069f2551cf436e0100f421e1760220349b4ec284255b458d6da539fa17314e8330459e0a653c254f775d4ec8f32b3f0147304402203a8353c5ee76a2e266432e5f993f882e05725297e64c0833cf44719f7dda8d3b022058a2f72e7739efd21657b4943cac60a0a3c749e712787f0e85726da4c3adcf8e014c695221027f2a0df8e86535d08ca3e766a178f90c813d2dd1d55b0166e82518efbffb18c02103556a35844b517e2fc8216701e2e0a64dbcbe62ad420ac6dd73dc79e69efeb69521031ef21bd55171032b7aec21ec82932735fb986f1d4d8611feee62ab38acf4a6bb53ae";
/// `api-docs-data.ts` prevout script hex for the 2-of-3 spend.
const SCRIPT_HEX: &str = "a914c9848245ae4f5d5934b5cbdfb79e04cdd337470b87";
const SCRIPTSIG_P2WPKH: &str = "1600149b27f072e4b972927c445d1946162a550b0914d8";
const SCRIPTSIG_P2WSH: &str =
    "220020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75";
const PKH_WITNESS_0: &str = "3040021c23902a01d4c5cff2c33c8bdb778a5aadea78a9a0d6d4db60aaa0fba1022069237d9dbf2db8cff9c260ba71250493682d01a746f4a45c5c7ea386e56d2bc902";
const PKH_WITNESS_1: &str = "0240187acd3e2fd3d8e1acffefa85907b6550730c24f78dfd3301c829fc4daf3cc";
const WSH_WITNESS_0: &str = "303f021c65aee6696e80be6e14545cfd64b44f17b0514c150eefdb090c0f0bd9021f3fef4aa95c252a225622aba99e4d5af5a6fe40d177acd593e64cf2f8557ccc03";
const WSH_WITNESS_1: &str = "03b55c6f0749e0f3e2caeca05f68e3699f1b3c62a550730f704985a6a9aae437a1";
const WSH_WITNESS_2: &str = "76a914db865fd920959506111079995f1e4017b489bfe38763ac6721024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c7c820120876475527c2103443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca52ae67a91446c3747322b220fdb925c9802f0e949c1feab99988ac6868";
const TXID: &str = "9d3ea0d131c45450c135d549b62032019bc47a80368e14edc72caf38f5a88033";
const PREV: &str = "69da555a9c69788a3a081958457894e56b1ee6766bc72cecf881b1b4f327f78b";
const PARAGRAPH: &str = "<p class=\"address-type\"><span class=\"address-type-label\">Type</span><span class=\"badge rounded-pill bg-warning\"><span>Multisig 2 of 3</span></span></p>";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn page(address: &str, txs_json: &str) -> String {
    address_rows_markup(&address_body(address), txs_json, "")
}

fn badge_vin(
    address: &str,
    script_type: &str,
    scriptsig: &str,
    witness: Option<&str>,
    redeem: Option<&str>,
    witness_asm: Option<&str>,
) -> String {
    let witness_field = match witness {
        Some(items) => format!(r#","witness":{items}"#),
        None => String::new(),
    };
    let redeem_field = match redeem {
        Some(asm) => format!(r#","inner_redeemscript_asm":"{asm}""#),
        None => String::new(),
    };
    let witness_asm_field = match witness_asm {
        Some(asm) => format!(r#","inner_witnessscript_asm":"{asm}""#),
        None => String::new(),
    };
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"OP_HASH160 OP_PUSHBYTES_20 c9848245ae4f5d5934b5cbdfb79e04cdd337470b OP_EQUAL","scriptpubkey_type":"{script_type}","scriptpubkey_address":"{address}","value":24962957}},"scriptsig":"{scriptsig}","scriptsig_asm":"","is_coinbase":false,"sequence":4294967295{witness_field}{redeem_field}{witness_asm_field}}}"#
    )
}

fn badge_txs(vins: &[String]) -> String {
    let vins = vins.join(",");
    format!(
        r#"[{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vins}],"vout":[{{"scriptpubkey":"5121028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae2851ae","scriptpubkey_asm":"OP_PUSHNUM_1 OP_PUSHBYTES_33 028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae28 OP_PUSHNUM_1 OP_CHECKMULTISIG","scriptpubkey_type":"multisig","value":582}}],"size":5514,"weight":22056,"sigops":208,"fee":44000}}]"#
    )
}

fn two_of_three_spend() -> String {
    badge_vin(
        P2SH_ADDR,
        "p2sh",
        SCRIPTSIG_2_OF_3,
        None,
        Some(REDEEM_2_OF_3),
        None,
    )
}

fn matching_badge_txs() -> String {
    let other = badge_vin(OTHER_P2SH, "p2sh", "", None, None, None);
    badge_txs(&[other, two_of_three_spend()])
}

fn pkh_witness() -> String {
    format!(r#"["{PKH_WITNESS_0}","{PKH_WITNESS_1}"]"#)
}

fn pkh_vin(address: &str, script_type: &str, scriptsig: &str, witness: Option<&str>) -> String {
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

fn pkh_txs(vins: &[String]) -> String {
    let vins = vins.join(",");
    format!(
        r#"[{{"txid":"cb9b47ac04023b29fb633a8ef04af351ac9fd74c57c9a2163f683516274767e3","version":1,"locktime":0,"vin":[{vins}],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1}}]"#
    )
}

fn matching_pkh_txs() -> String {
    let other = pkh_vin(OTHER_P2SH, "p2sh", "", Some("[]"));
    let spend = pkh_vin(
        P2SH_P2WPKH_ADDR,
        "p2sh",
        SCRIPTSIG_P2WPKH,
        Some(&pkh_witness()),
    );
    pkh_txs(&[other, spend])
}

fn wsh_witness() -> String {
    format!(r#"["{WSH_WITNESS_0}","{WSH_WITNESS_1}","{WSH_WITNESS_2}"]"#)
}

fn wsh_vin(address: &str, script_type: &str, scriptsig: &str, witness: Option<&str>) -> String {
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

fn wsh_txs(vins: &[String]) -> String {
    let vins = vins.join(",");
    format!(
        r#"[{{"txid":"cb9b47ac04023b29fb633a8ef04af351ac9fd74c57c9a2163f683516274767e3","version":1,"locktime":0,"vin":[{vins}],"vout":[],"size":10,"weight":40,"sigops":0,"fee":1}}]"#
    )
}

fn matching_wsh_txs() -> String {
    let other = wsh_vin(P2SH_ADDR, "p2sh", "", Some("[]"));
    let spend = wsh_vin(
        P2SH_P2WSH_ADDR,
        "p2sh",
        SCRIPTSIG_P2WSH,
        Some(&wsh_witness()),
    );
    wsh_txs(&[other, spend])
}

fn omits_line(markup: &str, why: &str) {
    assert!(
        !markup.lines().any(|line| line == LINE),
        "{why} painted {LINE}: {markup}"
    );
}

#[test]
fn address_page_paints_multisig_2_of_3_as_its_own_line() {
    assert!(
        SCRIPTSIG_2_OF_3.starts_with("0048"),
        "the Angular 2-of-3 spend was replaced"
    );
    assert!(
        !SCRIPTSIG_2_OF_3.starts_with("160014"),
        "the 2-of-3 spend must not be the wrapped pubkey-hash prefix"
    );
    assert!(
        SCRIPTSIG_P2WPKH.starts_with("160014"),
        "fixture scriptsig does not start with 160014"
    );
    assert!(
        SCRIPTSIG_P2WSH.starts_with("220020"),
        "fixture scriptsig does not start with 220020"
    );

    let markup = page(P2SH_ADDR, &matching_badge_txs());
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
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "no line equals Multisig 2 of 3: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains("<p")),
        "no line equals Multisig 2 of 3: {markup}"
    );
    for absent in [TYPE_MULTISIG, TYPE_P2SH_P2WPKH, TYPE_P2SH_P2WSH] {
        assert!(
            !markup.lines().any(|line| line == absent),
            "a line equal to {absent} was painted on the multisig address: {markup}"
        );
    }

    let wrapped_pkh = page(P2SH_P2WPKH_ADDR, &matching_pkh_txs());
    omits_line(&wrapped_pkh, "the 160014 spend");
    assert!(
        wrapped_pkh.lines().any(|line| line == TYPE_P2SH_P2WPKH),
        "the 160014 spend dropped {TYPE_P2SH_P2WPKH}: {wrapped_pkh}"
    );

    let wrapped_sh = page(P2SH_P2WSH_ADDR, &matching_wsh_txs());
    omits_line(&wrapped_sh, "the 220020 spend");
    assert!(
        wrapped_sh.lines().any(|line| line == TYPE_P2SH_P2WSH),
        "the 220020 spend dropped {TYPE_P2SH_P2WSH}: {wrapped_sh}"
    );

    let tapscript = page(P2TR_ADDR, &matching_wsh_txs());
    omits_line(&tapscript, "a tapscript address");
    assert!(
        tapscript.lines().any(|line| line == "Type P2TR"),
        "the tapscript address dropped Type P2TR: {tapscript}"
    );
}
