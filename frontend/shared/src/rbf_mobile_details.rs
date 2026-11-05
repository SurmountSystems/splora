//! Mobile replace-by-fee alert on the transaction details view.
//!
//! An unconfirmed replacement with a non-empty txid returns that txid
//! in the alert text. Confirmed, missing, and empty replacements return
//! empty text. This function does not build a page.

/// Alert text when an unconfirmed transaction has a non-empty replacement.
///
/// The text includes `alert alert-mempool` and
/// `This transaction has been replaced by:` plus the given txid.
/// Any other case returns an empty string.
pub fn mobile_rbf_replacement_alert(replacement_txid: Option<&str>, confirmed: bool) -> String {
    match (replacement_txid, confirmed) {
        (Some(txid), false) if !txid.is_empty() => {
            format!("alert alert-mempool This transaction has been replaced by: {txid}")
        }
        _ => String::new(),
    }
}
