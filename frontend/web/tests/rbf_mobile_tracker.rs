//! Mobile replace-by-fee alert on the transaction tracker.
//!
//! The alert text comes from `splora_web::paint_mobile_rbf_tracker_alert`.

#[test]
fn paint_shows_rbf_transactions_properly_mobile_tracker() {
    let first = "8913ec7ba0ede285dbd120e46f6d61a28f2903c10814a6f6c4f97d0edf3e1f46";
    let second = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let replaced_class = "alert-replaced";
    let replaced_by = "This transaction has been replaced by:";
    let newer = "Your transaction has been replaced by a newer version!";
    let mempool_alert = "alert alert-mempool";

    let replaced = splora_web::paint_mobile_rbf_tracker_alert(Some(first), true);
    assert!(replaced.contains(replaced_class), "{replaced}");
    assert!(replaced.contains(replaced_by), "{replaced}");
    assert!(replaced.contains(first), "{replaced}");
    assert!(replaced.contains(newer), "{replaced}");
    assert!(!replaced.contains(mempool_alert), "{replaced}");

    let other = splora_web::paint_mobile_rbf_tracker_alert(Some(second), true);
    assert!(other.contains(replaced_class), "{other}");
    assert!(other.contains(replaced_by), "{other}");
    assert!(other.contains(second), "{other}");
    assert!(other.contains(newer), "{other}");
    assert!(!other.contains(first), "{other}");
    assert!(!other.contains(mempool_alert), "{other}");

    let not_replaced = splora_web::paint_mobile_rbf_tracker_alert(Some(first), false);
    assert!(!not_replaced.contains(replaced_class), "{not_replaced}");
    assert!(!not_replaced.contains(newer), "{not_replaced}");
    assert!(!not_replaced.contains(first), "{not_replaced}");

    let absent = splora_web::paint_mobile_rbf_tracker_alert(None, false);
    assert!(!absent.contains(replaced_class), "{absent}");
    assert!(!absent.contains(newer), "{absent}");
}
