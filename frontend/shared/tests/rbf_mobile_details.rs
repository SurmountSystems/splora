//! Mobile replace-by-fee alert on the transaction details view.
//!
//! The function is not defined in this file.

#[test]
fn shows_rbf_transactions_properly_mobile_details() {
    let fixture = "8913ec7ba0ede285dbd120e46f6d61a28f2903c10814a6f6c4f97d0edf3e1f46";
    let other = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let sentence = "This transaction has been replaced by:";
    let class_text = "alert alert-mempool";

    let unconfirmed = splora_frontend_shared::mobile_rbf_replacement_alert(Some(fixture), false);
    assert!(unconfirmed.contains(sentence));
    assert!(unconfirmed.contains(class_text));
    assert!(unconfirmed.contains(fixture));

    let other_unconfirmed =
        splora_frontend_shared::mobile_rbf_replacement_alert(Some(other), false);
    assert!(other_unconfirmed.contains(sentence));
    assert!(other_unconfirmed.contains(other));
    assert!(!other_unconfirmed.contains(fixture));

    let confirmed = splora_frontend_shared::mobile_rbf_replacement_alert(Some(fixture), true);
    assert!(!confirmed.contains(sentence));
    assert!(!confirmed.contains(fixture));

    let absent = splora_frontend_shared::mobile_rbf_replacement_alert(None, false);
    assert!(!absent.contains(sentence));
}
