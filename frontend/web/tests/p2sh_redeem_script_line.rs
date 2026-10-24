//! Whole line `P2SH redeem script OP_DUP OP_HASH160` on the transaction input.
//!
//! GPUI `transaction_screen_paints_p2sh_redeem_script` requires one line whose
//! entire text is `P2SH redeem script OP_DUP OP_HASH160` when
//! `vin.inner_redeemscript_asm` is that asm. The spans stay. A missing key,
//! JSON null, and an empty string omit the line.

use splora_web::{transaction_requests, transaction_row_markup};

const TXID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PREV: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const LINE: &str = "P2SH redeem script OP_DUP OP_HASH160";
const ASM: &str = "OP_DUP OP_HASH160";
const LABEL_SPAN: &str = "<span class=\"p2sh-redeem-label\">P2SH redeem script</span>";
const SCRIPT_SIG: &str = "P2SHLINEHEXDEAD";
const SCRIPT_SIG_ASM: &str = "P2SHLINEASMDEAD";
const WITNESS: &str = "P2SHLINEWITDEAD";
const SCRIPT_PUBKEY: &str = "P2SHLINEPUBDEAD";
const SCRIPT_PUBKEY_ASM: &str = "P2SHLINEPUBASMDEAD";

fn tx(redeem_field: &str) -> String {
    format!(
        r#"{{"txid":"{TXID}","version":1,"locktime":0,"vin":[{{"txid":"{PREV}","vout":0,"prevout":null,"scriptsig":"{SCRIPT_SIG}","scriptsig_asm":"{SCRIPT_SIG_ASM}","witness":["{WITNESS}"],"is_coinbase":false,"sequence":1{redeem_field}}}],"vout":[{{"scriptpubkey":"{SCRIPT_PUBKEY}","scriptpubkey_asm":"{SCRIPT_PUBKEY_ASM}","scriptpubkey_type":"p2pkh","value":1}}],"size":10,"weight":40,"sigops":0,"fee":1}}"#
    )
}

fn paint(body: &str) -> String {
    let requests = transaction_requests(TXID);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, format!("/tx/{TXID}"));
    assert!(!requests[0].path.contains('?'));
    assert!(!requests[0].path.contains('#'));
    transaction_row_markup(body)
}

#[test]
fn present_asm_paints_p2sh_redeem_script_op_dup_as_its_own_line() {
    let markup = paint(&tx(&format!(r#","inner_redeemscript_asm":"{ASM}""#)));
    assert!(
        markup.lines().any(|line| line == LINE),
        "no line equals {LINE}: {markup}"
    );
    assert_eq!(
        markup.lines().filter(|line| *line == LINE).count(),
        1,
        "{LINE} was not one line: {markup}"
    );
    assert!(
        markup
            .lines()
            .any(|line| line == LINE && !line.contains('<')),
        "{LINE} was inside a span: {markup}"
    );
    assert_eq!(markup.matches(LABEL_SPAN).count(), 1, "{markup}");

    for omitted in [
        tx(""),
        tx(r#","inner_redeemscript_asm":null"#),
        tx(r#","inner_redeemscript_asm":"""#),
    ] {
        let painted = paint(&omitted);
        assert!(
            !painted.lines().any(|line| line == LINE),
            "an omitted asm painted {LINE}: {painted}"
        );
        assert!(
            !painted.contains(LINE),
            "an omitted asm contained {LINE}: {painted}"
        );
    }
}
