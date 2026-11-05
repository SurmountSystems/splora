//! Markup for an empty `#blinded=` fragment on the same Liquid transaction.
//!
//! Amount text is `recent_value_text` on each parsed output. An empty
//! `unblinded_tx_preview` leaves those amounts in place. A non-empty preview
//! is the populated unblind painter. Fee is passed to that preview and is
//! not painted as its own amount row.

use splora_frontend_shared::wire::{TxOutput, recent_value_text};

/// Paints vin and vout tables for `fragment` and `fee_sats`.
///
/// Both arguments go to `unblinded_tx_preview`. When that preview has lines,
/// `show_unblinded_tx_markup` paints the asset boxes. When the preview is
/// empty, this parses the one-input two-output transaction and paints each
/// confidential amount. The row class is `ng-star-inserted` because the
/// amount was not replaced.
pub fn show_empty_unblinded_tx_markup(fragment: &str, fee_sats: u64) -> String {
    let preview = splora_frontend_shared::unblinded_tx_preview(fragment, fee_sats);
    if !preview.is_empty() {
        return crate::show_unblinded_tx_markup(fragment, fee_sats);
    }
    let Ok(tx) = crate::parse_transaction_json(EMPTY_UNBLINDED_TX) else {
        return String::new();
    };
    let mut markup = String::from("<table class=\"table-tx-vin\"><tbody>");
    for input in &tx.vin {
        if let Some(prevout) = input.prevout.as_ref() {
            push_confidential_row(&mut markup, prevout);
        }
    }
    markup.push_str("</tbody></table><table class=\"table-tx-vout\"><tbody>");
    for output in &tx.vout {
        push_confidential_row(&mut markup, output);
    }
    markup.push_str("</tbody></table>");
    markup
}

fn push_confidential_row(markup: &mut String, output: &TxOutput) {
    let commitment = output
        .valuecommitment
        .as_deref()
        .filter(|text| !text.is_empty())
        .or_else(|| {
            output
                .assetcommitment
                .as_deref()
                .filter(|text| !text.is_empty())
        })
        .unwrap_or("");
    let amount = recent_value_text(output.value, commitment);
    if amount.is_empty() {
        return;
    }
    markup.push_str("<tr class=\"ng-star-inserted\"><td class=\"amount\">");
    markup.push_str(&amount);
    markup.push_str("</td></tr>");
}

/// Same visit as the populated unblind: one input and two outputs.
/// Values are omitted. A commitment keeps each amount confidential.
/// The commitment hex is not painted.
const EMPTY_UNBLINDED_TX: &str = concat!(
    r#"{"txid":"f2f41c0850e8e7e3f1af233161fd596662e67c11ef10ed15943884186fbb7f46","#,
    r#""version":1,"locktime":0,"vin":[{"txid":"prevouttx","vout":0,"prevout":{"#,
    r#""scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","#,
    r#""valuecommitment":"aa","assetcommitment":"bb"},"scriptsig":"","scriptsig_asm":"","#,
    r#""is_coinbase":false,"sequence":4294967295}],"vout":["#,
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","#,
    r#""valuecommitment":"cc","assetcommitment":"dd"},"#,
    r#"{"scriptpubkey":"0014aa","scriptpubkey_asm":"OP_0","scriptpubkey_type":"v0_p2wpkh","#,
    r#""valuecommitment":"ee","assetcommitment":"ff"}],"#,
    r#""size":10,"weight":40,"sigops":0,"fee":0,"status":{"confirmed":true}}"#
);
