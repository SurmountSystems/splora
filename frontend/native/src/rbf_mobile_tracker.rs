//! Native paint of the mobile replace-by-fee alert on the transaction tracker.
//! Text comes from `splora_frontend_shared::mobile_rbf_tracker_alert`.
//! This module does not open a window.

/// Forwards the shared mobile replace-by-fee tracker alert.
pub fn paint_mobile_rbf_tracker_alert(replacement_txid: Option<&str>, replaced: bool) -> String {
    splora_frontend_shared::mobile_rbf_tracker_alert(replacement_txid, replaced)
}
