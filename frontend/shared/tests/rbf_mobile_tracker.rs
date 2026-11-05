//! Mobile replace-by-fee tracker alert.
//!
//! The function is not defined in this file.

#[test]
fn shows_rbf_transactions_properly_mobile_tracker() {
    let first = "8913ec7ba0ede285dbd120e46f6d61a28f2903c10814a6f6c4f97d0edf3e1f46";
    let second = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let replaced_class = "alert-replaced";
    let replaced_by = "This transaction has been replaced by:";
    let newer = "Your transaction has been replaced by a newer version!";
    let mempool_alert = "alert alert-mempool";

    let replaced = splora_frontend_shared::mobile_rbf_tracker_alert(Some(first), true);
    assert!(replaced.contains(replaced_class));
    assert!(replaced.contains(replaced_by));
    assert!(replaced.contains(first));
    assert!(replaced.contains(newer));
    assert!(!replaced.contains(mempool_alert));

    let other = splora_frontend_shared::mobile_rbf_tracker_alert(Some(second), true);
    assert!(other.contains(replaced_class));
    assert!(other.contains(newer));
    assert!(other.contains(second));
    assert!(!other.contains(first));

    let not_replaced = splora_frontend_shared::mobile_rbf_tracker_alert(Some(first), false);
    assert!(!not_replaced.contains(replaced_class));
    assert!(!not_replaced.contains(newer));
    assert!(!not_replaced.contains(first));

    let absent = splora_frontend_shared::mobile_rbf_tracker_alert(None, false);
    assert!(!absent.contains(replaced_class));
    assert!(!absent.contains(newer));
}
