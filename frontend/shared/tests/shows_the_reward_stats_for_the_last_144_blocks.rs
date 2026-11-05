//! Static mining-dashboard title for the reward-stats element.
//!
//! Cypress only checks that the element exists. The title text is
//! `Reward stats`. The function is not defined in this file.

#[test]
fn shows_the_reward_stats_for_the_last_144_blocks() {
    let title = splora_frontend_shared::shows_the_reward_stats_for_the_last_144_blocks();
    assert!(
        title.contains("Reward stats"),
        "static mining-dashboard title, got {title}"
    );
}
