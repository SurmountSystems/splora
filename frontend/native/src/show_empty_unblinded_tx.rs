//! Text for an empty `#blinded=` fragment.
//!
//! An empty fragment does not unblind. The transaction screen still has one
//! input and two outputs, and each amount is [`crate::screens::blinded_value_text`].
//! That label is the word the empty-fragment screen already paints. Fee stays
//! on the transaction and is not a row. A non-empty fragment is the populated
//! unblind paint, not this screen.

use splora_frontend_shared::wire::{Transaction, TxInput, TxOutput};

/// Paints the empty-fragment transaction.
///
/// `fragment` empty builds one confidential input and two confidential
/// outputs, then lays them out as the vin table and the vout table. Each
/// amount cell is `blinded_value_text` on that `TxOutput`. A non-empty
/// fragment is `show_unblinded_tx_text`.
pub fn show_empty_unblinded_tx_text(fragment: &str, fee_sats: u64) -> String {
    if !fragment.is_empty() {
        return crate::show_unblinded_tx::show_unblinded_tx_text(fragment, fee_sats);
    }
    let tx = empty_confidential_transaction(fee_sats);
    confidential_tables(&tx)
}

/// One input and two outputs, values omitted, commitments present.
///
/// `fee_sats` is stored and not painted. The commitment marker is not painted.
fn empty_confidential_transaction(fee_sats: u64) -> Transaction {
    Transaction {
        txid: String::new(),
        version: 2,
        lock_time: 0,
        vin: vec![confidential_input(confidential_output())],
        vout: vec![confidential_output(), confidential_output()],
        size: 0,
        weight: 0,
        sigops: 0,
        fee: fee_sats,
        status: None,
    }
}

fn confidential_output() -> TxOutput {
    TxOutput {
        script_pubkey: String::new(),
        script_pubkey_asm: String::new(),
        script_pubkey_type: String::new(),
        script_pubkey_address: None,
        value: None,
        valuecommitment: Some("01".to_string()),
        asset: None,
        assetcommitment: None,
        pegout: None,
    }
}

fn confidential_input(prevout: TxOutput) -> TxInput {
    TxInput {
        txid: String::new(),
        vout: 0,
        prevout: Some(prevout),
        script_sig: String::new(),
        script_sig_asm: String::new(),
        witness: None,
        is_coinbase: false,
        is_pegin: None,
        issuance: None,
        sequence: 0,
        inner_redeem_script_asm: None,
        inner_witness_script_asm: None,
    }
}

/// Vin table, then vout table. Same table tags as the populated unblind.
///
/// An empty fragment does not use `assetBox`. Each party row keeps the
/// inserted-row class the empty-fragment screen names.
fn confidential_tables(tx: &Transaction) -> String {
    let mut text = String::from("<table class=\"table-tx-vin\"><tbody>");
    for input in &tx.vin {
        if let Some(prevout) = &input.prevout {
            push_inserted_row(&mut text, &crate::screens::blinded_value_text(prevout));
        }
    }
    text.push_str("</tbody></table><table class=\"table-tx-vout\"><tbody>");
    for output in &tx.vout {
        push_inserted_row(&mut text, &crate::screens::blinded_value_text(output));
    }
    text.push_str("</tbody></table>");
    text
}

fn push_inserted_row(text: &mut String, amount: &str) {
    text.push_str("<tr class=\"ng-star-inserted\"><td class=\"amount\">");
    text.push_str(amount);
    text.push_str("</td></tr>");
}
