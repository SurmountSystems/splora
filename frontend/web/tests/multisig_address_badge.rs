//! Address Type warning for a 2-of-3 pay-to-script-hash redeem script.
//!
//! After a successful no-query `GET /address/:script/txs`, the Type row shows
//! `Multisig 2 of 3` when a spend of that address keeps a 2-of-3 redeem
//! script. Angular builds the label in `script.utils.ts` as the word
//! Multisig, the required signature count, the word of, and the key count.
//! `address-labels.component.html` paints that label with
//! `badge rounded-pill bg-warning`. The address page places it on the Type
//! row when the address is not tapscript. `address-utils.ts` keeps
//! `inner_redeemscript_asm` only when the spend did not become the wrapped
//! pubkey-hash badge. A `160014` spend stays `P2SH-P2WPKH` and does not show
//! this warning. A `220020` spend stays `P2SH-P2WSH`. A tapscript address
//! does not show it. A redeem script that is not 2-of-3 does not show the
//! exact text `Multisig 2 of 3`. The same warning is on a transaction input
//! from no-query `GET /tx/:txid`. The output type line stays `MULTISIG`.
//! The asm, the script hex, and the witness stay off the badge.

use splora_web::{
    address_requests, address_rows_markup, indexer_failure_text, transaction_requests,
    transaction_row_markup,
};

/// `api-docs-data.ts` mainnet p2sh address for the 2-of-3 redeem script.
const P2SH_ADDR: &str = "3L4YUynB4X44rJBY9CmiLMN8Wjti49JCYB";
/// Same file, the other 2-of-3 address. Used as a spend that is not this page.
const OTHER_P2SH: &str = "3GcWrnGFoNzbn1KaiP5czS5xPELdWcgDX2";
/// `transactions-random.json` 3-of-5 p2sh address.
const THREE_OF_FIVE_ADDR: &str = "344BBtYkhaCXgA7oYSXASUfh4bFieiponG";
/// Same file, the `160014` wrapped spend.
const P2SH_P2WPKH_ADDR: &str = "375PJxsKRtAq4WoS6u82jvgZW94R8Wx3iH";
/// Same file, the `220020` wrapped spend.
const P2SH_P2WSH_ADDR: &str = "3GcrZrbUuvE4UtUdSbKTXcRnTqmfMdyMAC";
/// Same file, a tapscript address.
const P2TR_ADDR: &str = "bc1pwysjmmg07nymrvx9qhvqzfmjutd7nz3u4ecksdmmj58mdwrx4pysq6m68g";

/// `api-docs-data.ts` `inner_redeemscript_asm` for address `3L4Y...`.
const REDEEM_2_OF_3: &str = "OP_PUSHNUM_2 OP_PUSHBYTES_33 027f2a0df8e86535d08ca3e766a178f90c813d2dd1d55b0166e82518efbffb18c0 OP_PUSHBYTES_33 03556a35844b517e2fc8216701e2e0a64dbcbe62ad420ac6dd73dc79e69efeb695 OP_PUSHBYTES_33 031ef21bd55171032b7aec21ec82932735fb986f1d4d8611feee62ab38acf4a6bb OP_PUSHNUM_3 OP_CHECKMULTISIG";
/// `transactions-random.json` `inner_redeemscript_asm` for address `344B...`.
const REDEEM_3_OF_5: &str = "OP_PUSHNUM_3 OP_PUSHBYTES_33 03e0a220d36f6f7ed5f3f58c279d055707c454135baf18fd00d798fec3cb52dfbc OP_PUSHBYTES_33 03cf689db9313b9f7fc0b984dd9cac750be76041b392919b06f6bf94813da34cd4 OP_PUSHBYTES_33 027f8af2eb6e904deddaa60d5af393d430575eb35e4dfd942a8a5882734b078906 OP_PUSHBYTES_65 0411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3 OP_PUSHBYTES_65 04ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84c OP_PUSHNUM_5 OP_CHECKMULTISIG";
/// Same file, the `160014` scriptsig.
const SCRIPTSIG_P2WPKH: &str = "1600149b27f072e4b972927c445d1946162a550b0914d8";
/// Same file, the `220020` scriptsig.
const SCRIPTSIG_P2WSH: &str =
    "220020a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75";
/// `api-docs-data.ts` scriptsig for the 2-of-3 spend. It does not start with `160014`.
const SCRIPTSIG_2_OF_3: &str = "0048304502210099219ee0cd5da341650078e3c63885b3cc2211069f2551cf436e0100f421e1760220349b4ec284255b458d6da539fa17314e8330459e0a653c254f775d4ec8f32b3f0147304402203a8353c5ee76a2e266432e5f993f882e05725297e64c0833cf44719f7dda8d3b022058a2f72e7739efd21657b4943cac60a0a3c749e712787f0e85726da4c3adcf8e014c695221027f2a0df8e86535d08ca3e766a178f90c813d2dd1d55b0166e82518efbffb18c02103556a35844b517e2fc8216701e2e0a64dbcbe62ad420ac6dd73dc79e69efeb69521031ef21bd55171032b7aec21ec82932735fb986f1d4d8611feee62ab38acf4a6bb53ae";
/// `api-docs-data.ts` prevout script hex for the 2-of-3 spend.
const SCRIPT_HEX: &str = "a914c9848245ae4f5d5934b5cbdfb79e04cdd337470b87";
const WITNESS_0: &str = "3040021c23902a01d4c5cff2c33c8bdb778a5aadea78a9a0d6d4db60aaa0fba1022069237d9dbf2db8cff9c260ba71250493682d01a746f4a45c5c7ea386e56d2bc902";
const WITNESS_1: &str = "0240187acd3e2fd3d8e1acffefa85907b6550730c24f78dfd3301c829fc4daf3cc";
/// `transactions-random.json` witness script on the `220020` spend. Not a bare multisig.
const WITNESS_ASM_P2WSH: &str = "OP_DUP OP_HASH160 OP_PUSHBYTES_20 db865fd920959506111079995f1e4017b489bfe3 OP_EQUAL OP_IF OP_CHECKSIG OP_ELSE OP_PUSHBYTES_33 024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c OP_SWAP OP_SIZE OP_PUSHBYTES_1 20 OP_EQUAL OP_NOTIF OP_DROP OP_PUSHNUM_2 OP_SWAP OP_PUSHBYTES_33 03443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca OP_PUSHNUM_2 OP_CHECKMULTISIG OP_ELSE OP_HASH160 OP_PUSHBYTES_20 46c3747322b220fdb925c9802f0e949c1feab999 OP_EQUALVERIFY OP_CHECKSIG OP_ENDIF OP_ENDIF";
/// Same file, the redeem script on that wrapped spend. Not a multisig.
const REDEEM_P2WSH: &str =
    "OP_0 OP_PUSHBYTES_32 a18160de7291554f349c7d5cbee4ab97fb542e94cf302ce8d7e9747e4188ca75";

const TYPE_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge rounded-pill bg-warning\"><span>Multisig 2 of 3</span></span>";
const INPUT_BADGE: &str =
    "<span class=\"badge rounded-pill bg-warning\"><span>Multisig 2 of 3</span></span>";
const THREE_OF_FIVE_BADGE: &str = "<span class=\"address-type-label\">Type</span><span class=\"badge rounded-pill bg-warning\"><span>Multisig 3 of 5</span></span>";
const P2WPKH_BADGE: &str = "<span class=\"badge bg-primary\"><span>P2SH-P2WPKH</span></span>";
const P2WSH_BADGE: &str = "<span class=\"badge bg-primary\"><span>P2SH-P2WSH</span></span>";
const P2TR_BADGE: &str = "<span class=\"badge bg-primary\"><span>P2TR</span></span>";
const MULTISIG_OUTPUT: &str =
    "<span class=\"output-type-label\">Type</span><span class=\"output-type\">MULTISIG</span>";
const LOAD_ERROR: &str = "Error loading address data.";
const INDEXER_BODY: &str = "Invalid Bitcoin address";
const TXID: &str = "9d3ea0d131c45450c135d549b62032019bc47a80368e14edc72caf38f5a88033";
const PREV: &str = "69da555a9c69788a3a081958457894e56b1ee6766bc72cecf881b1b4f327f78b";

fn address_body(address: &str) -> String {
    format!(
        r#"{{"address":"{address}","chain_stats":{{"tx_count":1,"funded_txo_count":1,"spent_txo_count":0,"funded_txo_sum":1000,"spent_txo_sum":0}},"mempool_stats":{{"tx_count":0,"funded_txo_count":0,"spent_txo_count":0,"funded_txo_sum":0,"spent_txo_sum":0}}}}"#
    )
}

fn witness_pair() -> String {
    format!(r#"["{WITNESS_0}","{WITNESS_1}"]"#)
}

fn vin(
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

fn txs(vins: &[String], output_type: &str) -> String {
    let vins = vins.join(",");
    format!(
        r#"[{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vins}],"vout":[{{"scriptpubkey":"5121028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae2851ae","scriptpubkey_asm":"OP_PUSHNUM_1 OP_PUSHBYTES_33 028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae28 OP_PUSHNUM_1 OP_CHECKMULTISIG","scriptpubkey_type":"{output_type}","value":582}}],"size":5514,"weight":22056,"sigops":208,"fee":44000}}]"#
    )
}

fn one_tx(vin_json: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{vin_json}],"vout":[{{"scriptpubkey":"5121028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae2851ae","scriptpubkey_asm":"OP_PUSHNUM_1 OP_PUSHBYTES_33 028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae28 OP_PUSHNUM_1 OP_CHECKMULTISIG","scriptpubkey_type":"multisig","value":582}}],"size":5514,"weight":22056,"sigops":208,"fee":44000}}"#
    )
}

fn page(address: &str, txs_json: &str) -> String {
    address_rows_markup(&address_body(address), txs_json, "")
}

fn two_of_three_spend() -> String {
    vin(
        P2SH_ADDR,
        "p2sh",
        SCRIPTSIG_2_OF_3,
        None,
        Some(REDEEM_2_OF_3),
        None,
    )
}

fn matching_txs() -> String {
    let other = vin(OTHER_P2SH, "p2sh", "", None, None, None);
    txs(&[other, two_of_three_spend()], "multisig")
}

fn assert_absent(markup: &str, needle: &str) {
    assert!(
        !markup.contains(needle),
        "{needle} leaked into the markup: {markup}"
    );
}

fn assert_secrets_stay_off(markup: &str) {
    assert_absent(markup, SCRIPTSIG_2_OF_3);
    assert_absent(markup, SCRIPT_HEX);
    assert_absent(markup, WITNESS_0);
    assert_absent(markup, WITNESS_1);
    assert_absent(markup, "title=");
    assert_absent(markup, "tooltip");
}

#[test]
fn address_type_row_paints_multisig_two_of_three_after_no_query_txs() {
    assert!(
        SCRIPTSIG_2_OF_3.starts_with("0048"),
        "the Angular 2-of-3 spend was replaced"
    );
    assert!(
        !SCRIPTSIG_2_OF_3.starts_with("160014"),
        "the 2-of-3 spend must not be the wrapped pubkey-hash prefix"
    );
    let calls = address_requests(P2SH_ADDR);
    assert!(
        calls.iter().any(|call| {
            call.method == "GET"
                && call.path == format!("/address/{P2SH_ADDR}/txs")
                && !call.path.contains('?')
                && !call.path.contains('#')
        }),
        "no-query GET /address/:script/txs is missing: {calls:?}"
    );

    let markup = page(P2SH_ADDR, &matching_txs());
    assert!(
        markup.contains(TYPE_BADGE),
        "Multisig 2 of 3 is absent: {markup}"
    );
    assert_eq!(
        markup.matches(TYPE_BADGE).count(),
        1,
        "Multisig 2 of 3 type row count: {markup}"
    );
    assert!(
        markup.contains(MULTISIG_OUTPUT),
        "the output type MULTISIG was replaced: {markup}"
    );
    assert_secrets_stay_off(&markup);

    let wrapped_pkh = txs(
        &[vin(
            P2SH_P2WPKH_ADDR,
            "p2sh",
            SCRIPTSIG_P2WPKH,
            Some(&witness_pair()),
            Some(REDEEM_2_OF_3),
            None,
        )],
        "p2sh",
    );
    let wrapped_pkh_page = page(P2SH_P2WPKH_ADDR, &wrapped_pkh);
    assert!(
        wrapped_pkh_page.contains(P2WPKH_BADGE),
        "P2SH-P2WPKH is absent: {wrapped_pkh_page}"
    );
    assert_absent(&wrapped_pkh_page, "Multisig 2 of 3");
    assert_absent(&wrapped_pkh_page, WITNESS_0);

    let wrapped_sh = txs(
        &[vin(
            P2SH_P2WSH_ADDR,
            "p2sh",
            SCRIPTSIG_P2WSH,
            Some(&witness_pair()),
            Some(REDEEM_P2WSH),
            Some(WITNESS_ASM_P2WSH),
        )],
        "v0_p2wsh",
    );
    let wrapped_sh_page = page(P2SH_P2WSH_ADDR, &wrapped_sh);
    assert!(
        wrapped_sh_page.contains(P2WSH_BADGE),
        "P2SH-P2WSH is absent: {wrapped_sh_page}"
    );
    assert_absent(&wrapped_sh_page, "Multisig 2 of 3");

    let tapscript = txs(
        &[vin(
            P2TR_ADDR,
            "v1_p2tr",
            "",
            None,
            Some(REDEEM_2_OF_3),
            None,
        )],
        "v1_p2tr",
    );
    let tapscript_page = page(P2TR_ADDR, &tapscript);
    assert!(
        tapscript_page.contains(P2TR_BADGE),
        "P2TR is absent: {tapscript_page}"
    );
    assert_absent(&tapscript_page, "Multisig 2 of 3");

    let three_of_five = txs(
        &[vin(
            THREE_OF_FIVE_ADDR,
            "p2sh",
            "00443041",
            None,
            Some(REDEEM_3_OF_5),
            None,
        )],
        "multisig",
    );
    let three_page = page(THREE_OF_FIVE_ADDR, &three_of_five);
    assert!(
        three_page.contains(THREE_OF_FIVE_BADGE),
        "the 3-of-5 redeem script did not build its own counts: {three_page}"
    );
    assert_absent(&three_page, "Multisig 2 of 3");

    let tx_calls = transaction_requests(TXID);
    assert!(
        tx_calls.iter().any(|call| {
            call.method == "GET"
                && call.path == format!("/tx/{TXID}")
                && !call.path.contains('?')
                && !call.path.contains('#')
        }),
        "no-query GET /tx/:txid is missing: {tx_calls:?}"
    );
    let tx_markup = transaction_row_markup(&one_tx(&two_of_three_spend()));
    assert!(
        tx_markup.contains(INPUT_BADGE),
        "Multisig 2 of 3 is absent: {tx_markup}"
    );
    assert!(
        tx_markup.contains(MULTISIG_OUTPUT),
        "the output type MULTISIG was replaced: {tx_markup}"
    );
    assert_secrets_stay_off(&tx_markup);

    let failed = indexer_failure_text("GET", &format!("/address/{P2SH_ADDR}"), INDEXER_BODY);
    let failed_page = address_rows_markup(&failed, &matching_txs(), "");
    assert!(
        failed_page.contains(LOAD_ERROR),
        "Error loading address data. is absent: {failed_page}"
    );
    assert_absent(&failed_page, "Multisig 2 of 3");
}
