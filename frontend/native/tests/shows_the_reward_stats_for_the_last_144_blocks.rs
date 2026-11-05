//! Static mining-dashboard title for the reward-stats element.
//!
//! Cypress only checks that the element exists. The title text is
//! `Reward stats`. The paint function is not defined in this file.
//! This test does not assert a reward amount.

#[test]
fn shows_the_reward_stats_for_the_last_144_blocks() {
    let title = splora_native::paint_shows_the_reward_stats_for_the_last_144_blocks();
    assert!(
        title.contains("Reward stats"),
        "static mining-dashboard title, got {title}"
    );
}
