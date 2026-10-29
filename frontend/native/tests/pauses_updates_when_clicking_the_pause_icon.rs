//! Cypress it() title: pauses updates when clicking the pause icon.
//!
//! The it() visits `/txs`, fills the transaction list, clicks
//! `[data-cy="btn-pause"]`, then a later batch arrives. While paused,
//! each txid captured at the click stays in the rendered text. A later
//! txid that was not already visible stays out. The second argument
//! repeats one captured txid, and that id stays. This test does not
//! open a window and does not start an event loop.
//! `paint_pauses_updates_when_clicking_the_pause_icon` is not defined
//! on `splora_native`.

const CYPRESS_IT: &str = "pauses updates when clicking the pause icon";

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
    let painted: String =
        splora_native::paint_pauses_updates_when_clicking_the_pause_icon(paused_ids, later_ids);
    for id in paused_ids {
        assert!(
            painted.contains(id),
            "{CYPRESS_IT}: rendered text must contain paused txid {id}; got {painted:?}"
        );
    }
    for later in later_ids {
        if paused_ids.contains(&later) {
            assert!(
                painted.contains(later),
                "{CYPRESS_IT}: paused txid {later} stays in the rendered text; got {painted:?}"
            );
        } else {
            assert!(
                !painted.contains(later),
                "{CYPRESS_IT}: rendered text must not contain later txid {later}; got {painted:?}"
            );
        }
    }
    assert!(
        painted.contains("btn-pause"),
        "{CYPRESS_IT}: a pause control attribute must contain btn-pause; got {painted:?}"
    );
}
