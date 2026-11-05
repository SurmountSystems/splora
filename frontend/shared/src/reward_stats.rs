//! Static mining-dashboard title on the reward-stats span.
//!
//! The span text is `Reward stats`. This is not a reward amount, a fee,
//! a hashrate, or a price.

/// Static title `Reward stats` from the mining dashboard.
pub fn shows_the_reward_stats_for_the_last_144_blocks() -> &'static str {
    "Reward stats"
}
