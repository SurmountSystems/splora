//! Leptos paint of the reward-stats label for the last 144 blocks.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that title, a reward amount, or a price.

/// Paints the reward-stats label. The string is the shared function.
pub fn paint_shows_the_reward_stats_for_the_last_144_blocks() -> &'static str {
    splora_frontend_shared::shows_the_reward_stats_for_the_last_144_blocks()
}
