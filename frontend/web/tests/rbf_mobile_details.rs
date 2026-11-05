//! Mobile replace-by-fee alert on the transaction details view.
//!
//! The alert text comes from `splora_web::paint_mobile_rbf_replacement_alert`.

#[test]
fn paint_shows_rbf_transactions_properly_mobile_details() {
    let first = "8913ec7ba0ede285dbd120e46f6d61a28f2903c10814a6f6c4f97d0edf3e1f46";
    let second = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let replaced = "This transaction has been replaced by:";

    let unconfirmed = splora_web::paint_mobile_rbf_replacement_alert(Some(first), false);
    assert!(unconfirmed.contains(replaced), "{unconfirmed}");
    assert!(unconfirmed.contains("alert alert-mempool"), "{unconfirmed}");
    assert!(unconfirmed.contains(first), "{unconfirmed}");

    let other = splora_web::paint_mobile_rbf_replacement_alert(Some(second), false);
    assert!(other.contains(replaced), "{other}");
    assert!(other.contains(second), "{other}");
    assert!(!other.contains(first), "{other}");

    let confirmed = splora_web::paint_mobile_rbf_replacement_alert(Some(first), true);
    assert!(!confirmed.contains(replaced), "{confirmed}");
    assert!(!confirmed.contains(first), "{confirmed}");

    let missing = splora_web::paint_mobile_rbf_replacement_alert(None, false);
    assert!(!missing.contains(replaced), "{missing}");
}
