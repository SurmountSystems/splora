//! Warning badge for a pay-to-witness-script-hash Lightning commitment spend.
//!
//! After a successful no-query `GET /tx/:txid`, a `v0_p2wsh` input shows
//! `Lightning Force Close` when `inner_witnessscript_asm` matches the
//! commitment-output pattern in `script.utils.ts` and the witness item under
//! the last item is not the one-byte revocation marker `01`.
//! `address-labels.component.html` paints that label with
//! `badge rounded-pill bg-warning`. The transaction list passes the input
//! when no channel object is present. `address-utils.ts` keeps the witness
//! script for `v0_p2wsh`. The revoked marker does not paint this badge and
//! does not paint `Revoked Lightning Force Close`. A witness script that is
//! not the commitment pattern does not paint it. The asm, the witness, and
//! the script hex stay off the row. This is not a Lightning route.

use splora_web::{transaction_requests, transaction_row_markup};

/// `transactions-random.json` transaction that spends the commitment output.
const TXID: &str = "b10c0000004da5a9d1d9b4ae32e09f0b3e62d21a5cce5428d4ad714fb444eb5d";
/// Prevout txid on that same input.
const PREV: &str = "637db3928a8fb1b22b81f92dc738ee7637e5b172d650363d0b327429578bd001";
/// Same input, `scriptpubkey_address`.
const ADDRESS: &str = "bc1q49fs59nletdxwtq59mnrdhx3w9uku6003cm658mh7dw93mwh5dts2w2kht";
/// Same input, `scriptpubkey`.
const SCRIPT_HEX: &str = "0020a9530a167fcada672c142ee636dcd171796e69ef8e37aa1f77f35c58edd7a357";
/// Same input, `scriptpubkey_asm`.
const SCRIPT_ASM: &str =
    "OP_0 OP_PUSHBYTES_32 a9530a167fcada672c142ee636dcd171796e69ef8e37aa1f77f35c58edd7a357";
/// Same input, signature witness item.
const WITNESS_SIG: &str = "303d021c32f9454db85cb1a4ca63a9883d4347c5e13f3654e884ae44e9efa3c8021d62f07fe452c06b084bc3e09afd3aac4039136549a465533bc1ca66967902";
/// Same input, witness script hex. The asm below is this script.
const WITNESS_SCRIPT: &str = "632102fd6db4de50399b2aa086edb23f8e140bbc823d6651e024a0eb871288068789cd67012ab27521034134a2bb35c3f83dab2489d96160741888b8b5589bb694dea6e7bc24486e9c6f68ac";
/// Same input, `inner_witnessscript_asm`. Commitment-output pattern.
const COMMITMENT_ASM: &str = "OP_IF OP_PUSHBYTES_33 02fd6db4de50399b2aa086edb23f8e140bbc823d6651e024a0eb871288068789cd OP_ELSE OP_PUSHBYTES_1 2a OP_CSV OP_DROP OP_PUSHBYTES_33 034134a2bb35c3f83dab2489d96160741888b8b5589bb694dea6e7bc24486e9c6f OP_ENDIF OP_CHECKSIG";
/// Same file, the wrapped witness script. It is not a commitment output.
const OTHER_ASM: &str = "OP_DUP OP_HASH160 OP_PUSHBYTES_20 db865fd920959506111079995f1e4017b489bfe3 OP_EQUAL OP_IF OP_CHECKSIG OP_ELSE OP_PUSHBYTES_33 024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c OP_SWAP OP_SIZE OP_PUSHBYTES_1 20 OP_EQUAL OP_NOTIF OP_DROP OP_PUSHNUM_2 OP_SWAP OP_PUSHBYTES_33 03443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca OP_PUSHNUM_2 OP_CHECKMULTISIG OP_ELSE OP_HASH160 OP_PUSHBYTES_20 46c3747322b220fdb925c9802f0e949c1feab999 OP_EQUALVERIFY OP_CHECKSIG OP_ENDIF OP_ENDIF";
/// Same file, witness items on that wrapped spend.
const OTHER_WITNESS: &str = r#"["303f021c65aee6696e80be6e14545cfd64b44f17b0514c150eefdb090c0f0bd9021f3fef4aa95c252a225622aba99e4d5af5a6fe40d177acd593e64cf2f8557ccc03","03b55c6f0749e0f3e2caeca05f68e3699f1b3c62a550730f704985a6a9aae437a1","76a914db865fd920959506111079995f1e4017b489bfe38763ac6721024d560f7f5d28aae5e1a8aa2b7ba615d7fc48e4ea27e5d27336e6a8f5fa0f5c8c7c820120876475527c2103443e8834fa7d79d7b5e95e0e9d0847f6b03ac3ea977979858b4104947fca87ca52ae67a91446c3747322b220fdb925c9802f0e949c1feab99988ac6868"]"#;
/// Same transaction, the multisig output.
const OUT_HEX: &str = "5121028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae2851ae";
const OUT_ASM: &str = "OP_PUSHNUM_1 OP_PUSHBYTES_33 028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae28 OP_PUSHNUM_1 OP_CHECKMULTISIG";

const FORCE_CLOSE_BADGE: &str =
    "<span class=\"badge rounded-pill bg-warning\"><span>Lightning Force Close</span></span>";
const REVOKED_LABEL: &str = "Revoked Lightning Force Close";
const MULTISIG_OUTPUT: &str =
    "<span class=\"output-type-label\">Type</span><span class=\"output-type\">MULTISIG</span>";

/// `script.utils.ts` uses `''` for the delayed to_local output.
/// The stored fixture uses `01`, which is the revocation marker.
fn commitment_witness(under_last: &str) -> String {
    format!(r#"["{WITNESS_SIG}","{under_last}","{WITNESS_SCRIPT}"]"#)
}

fn vin(witness: &str, witness_asm: &str) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{{"scriptpubkey":"{SCRIPT_HEX}","scriptpubkey_asm":"{SCRIPT_ASM}","scriptpubkey_type":"v0_p2wsh","scriptpubkey_address":"{ADDRESS}","value":8149}},"scriptsig":"","scriptsig_asm":"","witness":{witness},"is_coinbase":false,"sequence":4190024921,"inner_witnessscript_asm":"{witness_asm}"}}"#
    )
}

fn tx(vin_json: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":1231006505,"vin":[{vin_json}],"vout":[{{"scriptpubkey":"{OUT_HEX}","scriptpubkey_asm":"{OUT_ASM}","scriptpubkey_type":"multisig","value":582}}],"size":3500,"weight":8186,"sigops":115,"fee":71294}}"#
    )
}

fn assert_absent(markup: &str, needle: &str) {
    assert!(
        !markup.contains(needle),
        "{needle} leaked into the markup: {markup}"
    );
}

fn assert_script_stays_off(markup: &str) {
    assert_absent(markup, COMMITMENT_ASM);
    assert_absent(markup, OTHER_ASM);
    assert_absent(markup, SCRIPT_HEX);
    assert_absent(markup, SCRIPT_ASM);
    assert_absent(markup, WITNESS_SIG);
    assert_absent(markup, WITNESS_SCRIPT);
    assert_absent(markup, OUT_HEX);
    assert_absent(markup, OUT_ASM);
    assert_absent(markup, "title=");
    assert_absent(markup, "tooltip");
    assert_absent(markup, "/lightning");
    assert_absent(markup, REVOKED_LABEL);
}

#[test]
fn p2wsh_input_paints_lightning_force_close_after_no_query_tx() {
    assert!(
        COMMITMENT_ASM.starts_with("OP_IF OP_PUSHBYTES_33 "),
        "the commitment witness script was replaced"
    );
    assert!(
        COMMITMENT_ASM.ends_with(" OP_ENDIF OP_CHECKSIG"),
        "the commitment witness script was replaced"
    );
    assert!(
        !OTHER_ASM.starts_with("OP_IF OP_PUSHBYTES_33 "),
        "the non-commitment witness script was replaced"
    );

    let calls = transaction_requests(TXID);
    assert!(
        calls.iter().any(|call| {
            call.method == "GET"
                && call.path == format!("/tx/{TXID}")
                && !call.path.contains('?')
                && !call.path.contains('#')
        }),
        "no-query GET /tx/:txid is missing: {calls:?}"
    );

    let markup = transaction_row_markup(&tx(&vin(&commitment_witness(""), COMMITMENT_ASM)));
    assert!(
        markup.contains(FORCE_CLOSE_BADGE),
        "Lightning Force Close is absent: {markup}"
    );
    assert_eq!(
        markup.matches(FORCE_CLOSE_BADGE).count(),
        1,
        "Lightning Force Close count: {markup}"
    );
    assert!(
        markup.contains(MULTISIG_OUTPUT),
        "the output type MULTISIG was replaced: {markup}"
    );
    assert!(
        markup.contains("<span class=\"previous-output-type\">V0_P2WSH</span>"),
        "the input type V0_P2WSH was replaced: {markup}"
    );
    assert_absent(&markup, "Multisig 2 of 3");
    assert_script_stays_off(&markup);

    let revoked = transaction_row_markup(&tx(&vin(&commitment_witness("01"), COMMITMENT_ASM)));
    assert_absent(&revoked, "Lightning Force Close");
    assert_absent(&revoked, REVOKED_LABEL);
    assert!(
        revoked.contains(MULTISIG_OUTPUT),
        "the output type MULTISIG was replaced: {revoked}"
    );
    assert_script_stays_off(&revoked);

    let other = transaction_row_markup(&tx(&vin(OTHER_WITNESS, OTHER_ASM)));
    assert_absent(&other, "Lightning Force Close");
    assert_absent(&other, REVOKED_LABEL);
    assert!(
        other.contains(MULTISIG_OUTPUT),
        "the output type MULTISIG was replaced: {other}"
    );
    assert!(
        other.contains("<span class=\"previous-output-type\">V0_P2WSH</span>"),
        "the input type V0_P2WSH was replaced: {other}"
    );
    assert_script_stays_off(&other);
}
