//! Mobile replace-by-fee alert on the transaction tracker.
//!
//! A replaced transaction with a non-empty txid returns that txid in
//! the tracker alert text. A transaction that was not replaced, or a
//! missing or empty id, returns empty text. This is not the details
//! alert and does not build a page.

/// Tracker alert when a transaction has been replaced by a non-empty txid.
///
/// The text includes `alert-replaced`,
/// `This transaction has been replaced by:`, the given txid, and
/// `Your transaction has been replaced by a newer version!`.
/// Any other case returns an empty string.
pub fn mobile_rbf_tracker_alert(replacement_txid: Option<&str>, replaced: bool) -> String {
    match (replacement_txid, replaced) {
        (Some(txid), true) if !txid.is_empty() => format!(
            "alert-replaced This transaction has been replaced by: {txid} Your transaction has been replaced by a newer version!"
        ),
        _ => String::new(),
    }
}
