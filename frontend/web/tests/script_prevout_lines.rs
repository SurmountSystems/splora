//! Whole lines on a transaction input from `GET /tx/:txid`.
//!
//! GPUI `transaction_screen_paints_p2pk_label` requires one line whose entire
//! text is `input script type p2pk` when the prevout type is `p2pk`.
//! GPUI `transaction_screen_paints_op_return_label` requires one line whose
//! entire text is `input script type op_return` when the prevout type is
//! `op_return`. GPUI `assert_force_close_neighbors` requires one line whose
//! entire text is `Previous output type V0_P2WSH` when the prevout type is
//! `v0_p2wsh`. That same check also requires one line whose entire text is
//! `input script type v0_p2wsh`. The scriptpubkey-type span and the
//! previous-output-type span stay. A span that contains the words is not
//! that line.

use splora_web::{transaction_requests, transaction_row_markup};

const P2PK_TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OP_RETURN_TXID: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const INPUT_V0_P2WSH_TXID: &str =
    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ORDINARY_VOUT: &str = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","scriptpubkey_address":"bc1qordinary","value":9}"#;

/// `transactions-random.json` transaction that spends the commitment output.
const FORCE_CLOSE_TXID: &str = "b10c0000004da5a9d1d9b4ae32e09f0b3e62d21a5cce5428d4ad714fb444eb5d";
const FORCE_CLOSE_PREV: &str = "637db3928a8fb1b22b81f92dc738ee7637e5b172d650363d0b327429578bd001";
const FORCE_CLOSE_ADDRESS: &str = "bc1q49fs59nletdxwtq59mnrdhx3w9uku6003cm658mh7dw93mwh5dts2w2kht";
const FORCE_CLOSE_SCRIPT_HEX: &str =
    "0020a9530a167fcada672c142ee636dcd171796e69ef8e37aa1f77f35c58edd7a357";
const FORCE_CLOSE_SCRIPT_ASM: &str =
    "OP_0 OP_PUSHBYTES_32 a9530a167fcada672c142ee636dcd171796e69ef8e37aa1f77f35c58edd7a357";
const FORCE_CLOSE_SIG: &str = "303d021c32f9454db85cb1a4ca63a9883d4347c5e13f3654e884ae44e9efa3c8021d62f07fe452c06b084bc3e09afd3aac4039136549a465533bc1ca66967902";
const FORCE_CLOSE_WITNESS_SCRIPT: &str = "632102fd6db4de50399b2aa086edb23f8e140bbc823d6651e024a0eb871288068789cd67012ab27521034134a2bb35c3f83dab2489d96160741888b8b5589bb694dea6e7bc24486e9c6f68ac";
const FORCE_CLOSE_COMMITMENT_ASM: &str = "OP_IF OP_PUSHBYTES_33 02fd6db4de50399b2aa086edb23f8e140bbc823d6651e024a0eb871288068789cd OP_ELSE OP_PUSHBYTES_1 2a OP_CSV OP_DROP OP_PUSHBYTES_33 034134a2bb35c3f83dab2489d96160741888b8b5589bb694dea6e7bc24486e9c6f OP_ENDIF OP_CHECKSIG";
const FORCE_CLOSE_OUT_HEX: &str =
    "5121028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae2851ae";
const FORCE_CLOSE_OUT_ASM: &str = "OP_PUSHNUM_1 OP_PUSHBYTES_33 028b45a50f795be0413680036665d17a3eca099648ea80637bc3a70a7d2b52ae28 OP_PUSHNUM_1 OP_CHECKMULTISIG";

fn tx(txid: &str, vin: &str, vout: &str) -> String {
    format!(
        r#"{{"txid":"{txid}","version":1,"locktime":0,"vin":[{vin}],"vout":[{vout}],"size":10,"weight":400,"sigops":0,"fee":50,"status":{{"confirmed":true,"block_height":4,"block_hash":"tiphash1","block_time":1600000000}}}}"#
    )
}

fn vin(prevout: &str) -> String {
    format!(
        r#"{{"txid":"{PREV}","vout":0,"prevout":{prevout},"scriptsig":"deadbeef","scriptsig_asm":"OP_CHECKSIG","witness":["abcdwitnesshex"],"is_coinbase":false,"sequence":1}}"#
    )
}

fn paint(txid: &str, body: &str) -> String {
    let requests = transaction_requests(txid);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{txid}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(body)
}

fn assert_whole_line(markup: &str, line: &str) {
    assert!(
        markup.lines().any(|row| row == line),
        "no line equals {line}: {markup}"
    );
}

#[test]
fn p2pk_prevout_paints_input_script_type_p2pk() {
    let prevout = r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"p2pk","value":5}"#;
    let markup = paint(P2PK_TXID, &tx(P2PK_TXID, &vin(prevout), ORDINARY_VOUT));
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">input script type p2pk</span>"),
        "the input script type span was dropped: {markup}"
    );
    assert_whole_line(&markup, "input script type p2pk");
}

#[test]
fn op_return_prevout_paints_input_script_type_op_return() {
    let prevout = r#"{"scriptpubkey":"6a0468656c6c6f","scriptpubkey_asm":"OP_RETURN 68656c6c6f","scriptpubkey_type":"op_return","value":5}"#;
    let markup = paint(
        OP_RETURN_TXID,
        &tx(OP_RETURN_TXID, &vin(prevout), ORDINARY_VOUT),
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">input script type op_return</span>"),
        "the input script type span was dropped: {markup}"
    );
    assert_whole_line(&markup, "input script type op_return");
}

#[test]
fn v0_p2wsh_prevout_paints_previous_output_type_v0_p2wsh() {
    let witness = format!(r#"["{FORCE_CLOSE_SIG}","","{FORCE_CLOSE_WITNESS_SCRIPT}"]"#);
    let vin = format!(
        r#"{{"txid":"{FORCE_CLOSE_PREV}","vout":0,"prevout":{{"scriptpubkey":"{FORCE_CLOSE_SCRIPT_HEX}","scriptpubkey_asm":"{FORCE_CLOSE_SCRIPT_ASM}","scriptpubkey_type":"v0_p2wsh","scriptpubkey_address":"{FORCE_CLOSE_ADDRESS}","value":8149}},"scriptsig":"","scriptsig_asm":"","witness":{witness},"is_coinbase":false,"sequence":4190024921,"inner_witnessscript_asm":"{FORCE_CLOSE_COMMITMENT_ASM}"}}"#
    );
    let vout = format!(
        r#"{{"scriptpubkey":"{FORCE_CLOSE_OUT_HEX}","scriptpubkey_asm":"{FORCE_CLOSE_OUT_ASM}","scriptpubkey_type":"multisig","value":582}}"#
    );
    let body = format!(
        r#"{{"txid":"{FORCE_CLOSE_TXID}","version":1,"locktime":1231006505,"vin":[{vin}],"vout":[{vout}],"size":3500,"weight":8186,"sigops":115,"fee":71294}}"#
    );
    let markup = paint(FORCE_CLOSE_TXID, &body);
    assert!(
        markup.contains("<span class=\"previous-output-type\">V0_P2WSH</span>"),
        "the previous-output-type span was dropped: {markup}"
    );
    assert_whole_line(&markup, "Previous output type V0_P2WSH");
}

#[test]
fn v0_p2wsh_prevout_paints_input_script_type_v0_p2wsh() {
    let prevout = r#"{"scriptpubkey":"0020aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wsh","value":5}"#;
    let markup = paint(
        INPUT_V0_P2WSH_TXID,
        &tx(INPUT_V0_P2WSH_TXID, &vin(prevout), ORDINARY_VOUT),
    );
    assert!(
        markup.contains("<span class=\"scriptpubkey-type\">input script type v0_p2wsh</span>"),
        "the input script type span was dropped: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == "input script type v0_p2wsh"),
        "no line equals input script type v0_p2wsh: {markup}"
    );
}
