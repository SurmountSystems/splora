//! Cypress it() title: pauses updates when clicking the pause icon.
//!
//! Clicking `[data-cy="btn-pause"]` is a local pause control on the
//! recent-transactions list. The visible txids stay the set captured at
//! the pause click. A later txid stays out of that list unless it was
//! already visible. This is not an HTTP GET.

use splora_frontend_shared::after_pause_icon_click_updates_are_paused;

#[test]
fn pauses_updates_when_clicking_the_pause_icon() {
    let paused_ids = [
        "1111111111111111111111111111111111111111111111111111111111111111",
        "2222222222222222222222222222222222222222222222222222222222222222",
    ];
    let later_ids = [
        "2222222222222222222222222222222222222222222222222222222222222222",
        "3333333333333333333333333333333333333333333333333333333333333333",
    ];
    let visible = after_pause_icon_click_updates_are_paused(paused_ids, later_ids);
    assert_eq!(
        visible, paused_ids,
        "visible txids stay the set captured when the pause icon is clicked"
    );
    for later in later_ids {
        if paused_ids.contains(&later) {
            assert!(
                visible.contains(&later),
                "txid {later} was already visible at the pause click, so it stays"
            );
        } else {
            assert!(
                !visible.contains(&later),
                "later txid {later} stays out of the list while updates are paused"
            );
        }
    }
}
