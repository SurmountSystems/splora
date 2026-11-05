//! Web paint of the mobile replace-by-fee alert on transaction details.
//! Text comes from `splora_frontend_shared::mobile_rbf_replacement_alert`.

/// Forwards the shared mobile replace-by-fee alert.
pub fn paint_mobile_rbf_replacement_alert(
    replacement_txid: Option<&str>,
    confirmed: bool,
) -> String {
    splora_frontend_shared::mobile_rbf_replacement_alert(replacement_txid, confirmed)
}
