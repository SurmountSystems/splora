//! Fee estimation based on projected mempool blocks.
//!
//! This module calculates recommended transaction fees based on the fee statistics
//! of projected mempool blocks (created by the GBT algorithm).
//!
//! Ported from mempool's fee-api.ts.

use crate::chain::Network;
use crate::util::gbt::MempoolBlock;

/// Recommended fee rates for different confirmation time targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedFees {
    /// Fee rate for confirmation in the next block (sat/vB)
    pub fastest_fee: f64,
    /// Fee rate for confirmation within ~30 minutes / 3 blocks (sat/vB)
    pub half_hour_fee: f64,
    /// Fee rate for confirmation within ~1 hour / 6 blocks (sat/vB)
    pub hour_fee: f64,
    /// Economy fee rate (sat/vB)
    pub economy_fee: f64,
    /// Minimum relay fee rate (sat/vB)
    pub minimum_fee: f64,
}

impl Default for RecommendedFees {
    fn default() -> Self {
        Self {
            fastest_fee: 1.0,
            half_hour_fee: 1.0,
            hour_fee: 1.0,
            economy_fee: 1.0,
            minimum_fee: 1.0,
        }
    }
}

/// Fee estimation configuration
#[derive(Debug, Clone)]
pub struct FeeEstimationConfig {
    /// Minimum fee increment for rounding (sat/vB)
    pub minimum_increment: f64,
    /// Minimum fastest fee (sat/vB)
    pub min_fastest_fee: f64,
    /// Minimum half hour fee (sat/vB)
    pub min_half_hour_fee: f64,
    /// Priority factor added to highest priority recommendations (sat/vB)
    pub priority_factor: f64,
}

impl FeeEstimationConfig {
    /// Configuration for Bitcoin mainnet/testnet
    pub fn bitcoin() -> Self {
        Self {
            minimum_increment: 1.0,
            min_fastest_fee: 1.0,
            min_half_hour_fee: 0.5,
            priority_factor: 0.5,
        }
    }

    /// Configuration for Liquid network
    pub fn liquid() -> Self {
        Self {
            minimum_increment: 0.1,
            min_fastest_fee: 0.1,
            min_half_hour_fee: 0.1,
            priority_factor: 0.0,
        }
    }

    /// Create configuration based on network type
    pub fn for_network(network: Network) -> Self {
        if network.is_liquid() {
            Self::liquid()
        } else {
            Self::bitcoin()
        }
    }
}

/// Fee estimator that calculates recommended fees from projected blocks
pub struct FeeEstimator {
    config: FeeEstimationConfig,
}

impl FeeEstimator {
    pub fn new(config: FeeEstimationConfig) -> Self {
        Self { config }
    }

    /// Create a fee estimator for the given network
    pub fn for_network(network: Network) -> Self {
        Self::new(FeeEstimationConfig::for_network(network))
    }

    /// Calculate recommended fees from projected mempool blocks.
    ///
    /// # Arguments
    /// * `projected_blocks` - Projected mempool blocks from GBT algorithm
    /// * `mempool_min_fee` - Minimum fee to get into mempool (BTC/kvB from getmempoolinfo)
    pub fn calculate_recommended_fees(
        &self,
        projected_blocks: &[MempoolBlock],
        mempool_min_fee: f64,
    ) -> RecommendedFees {
        self.calculate_recommended_fees_with_increment(
            projected_blocks,
            mempool_min_fee,
            self.config.minimum_increment,
        )
    }

    /// Calculate precise recommended fees with sub-satoshi precision.
    ///
    /// # Arguments
    /// * `projected_blocks` - Projected mempool blocks from GBT algorithm
    /// * `mempool_min_fee` - Minimum fee to get into mempool (BTC/kvB from getmempoolinfo)
    pub fn calculate_precise_recommended_fees(
        &self,
        projected_blocks: &[MempoolBlock],
        mempool_min_fee: f64,
    ) -> RecommendedFees {
        // Use 0.001 sat/vB precision (minimum non-zero minrelaytxfee/incrementalrelayfee)
        let mut recommendations = self.calculate_recommended_fees_with_increment(
            projected_blocks,
            mempool_min_fee,
            0.001,
        );

        // Enforce floor & offset for highest priority recommendations
        recommendations.fastest_fee = (recommendations.fastest_fee + self.config.priority_factor)
            .max(self.config.min_fastest_fee);
        recommendations.half_hour_fee = (recommendations.half_hour_fee
            + self.config.priority_factor / 2.0)
            .max(self.config.min_half_hour_fee);

        // Round to 3 decimal places
        RecommendedFees {
            fastest_fee: (recommendations.fastest_fee * 1000.0).round() / 1000.0,
            half_hour_fee: (recommendations.half_hour_fee * 1000.0).round() / 1000.0,
            hour_fee: (recommendations.hour_fee * 1000.0).round() / 1000.0,
            economy_fee: (recommendations.economy_fee * 1000.0).round() / 1000.0,
            minimum_fee: (recommendations.minimum_fee * 1000.0).round() / 1000.0,
        }
    }

    /// Internal fee calculation with configurable increment.
    fn calculate_recommended_fees_with_increment(
        &self,
        projected_blocks: &[MempoolBlock],
        mempool_min_fee: f64,
        min_increment: f64,
    ) -> RecommendedFees {
        let purge_rate = round_up_to_nearest(mempool_min_fee, min_increment);
        let minimum_fee = purge_rate.max(min_increment);

        if projected_blocks.is_empty() {
            return RecommendedFees {
                fastest_fee: minimum_fee,
                half_hour_fee: minimum_fee,
                hour_fee: minimum_fee,
                economy_fee: minimum_fee,
                minimum_fee,
            };
        }

        // Calculate median fees for first 3 blocks
        let first_median_fee = self.optimize_median_fee(
            &projected_blocks[0],
            projected_blocks.get(1),
            None,
            minimum_fee,
            min_increment,
        );

        let second_median_fee = projected_blocks.get(1).map_or(minimum_fee, |block| {
            self.optimize_median_fee(
                block,
                projected_blocks.get(2),
                Some(first_median_fee),
                minimum_fee,
                min_increment,
            )
        });

        let third_median_fee = projected_blocks.get(2).map_or(minimum_fee, |block| {
            self.optimize_median_fee(
                block,
                projected_blocks.get(3),
                Some(second_median_fee),
                minimum_fee,
                min_increment,
            )
        });

        // Enforce minimum fee on all recommendations
        let mut fastest_fee = first_median_fee.max(minimum_fee);
        let mut half_hour_fee = second_median_fee.max(minimum_fee);
        let mut hour_fee = third_median_fee.max(minimum_fee);
        let economy_fee = (2.0 * minimum_fee).min(third_median_fee).max(minimum_fee);

        // Ensure recommendations always increase with priority
        fastest_fee = fastest_fee
            .max(half_hour_fee)
            .max(hour_fee)
            .max(economy_fee);
        half_hour_fee = half_hour_fee.max(hour_fee).max(economy_fee);
        hour_fee = hour_fee.max(economy_fee);

        RecommendedFees {
            fastest_fee: round_to_nearest(fastest_fee, min_increment),
            half_hour_fee: round_to_nearest(half_hour_fee, min_increment),
            hour_fee: round_to_nearest(hour_fee, min_increment),
            economy_fee: round_to_nearest(economy_fee, min_increment),
            minimum_fee: round_to_nearest(minimum_fee, min_increment),
        }
    }

    /// Optimize median fee based on block fullness.
    ///
    /// For partially full blocks, the fee is scaled down proportionally.
    fn optimize_median_fee(
        &self,
        block: &MempoolBlock,
        next_block: Option<&MempoolBlock>,
        previous_fee: Option<f64>,
        min_fee: f64,
        min_increment: f64,
    ) -> f64 {
        let use_fee = match previous_fee {
            Some(prev) => (block.median_fee + prev) / 2.0,
            None => block.median_fee,
        };

        // If block is less than half full or median fee is below minimum, use minimum
        if block.block_vsize <= 500_000.0 || block.median_fee < min_fee {
            return min_fee;
        }

        // If block is between 50-95% full and there's no next block,
        // scale the fee proportionally
        if block.block_vsize <= 950_000.0 && next_block.is_none() {
            let multiplier = (block.block_vsize - 500_000.0) / 500_000.0;
            return round_to_nearest(use_fee * multiplier, min_increment).max(min_fee);
        }

        round_up_to_nearest(use_fee, min_increment).max(min_fee)
    }
}

/// Round up to the nearest increment
fn round_up_to_nearest(value: f64, nearest: f64) -> f64 {
    if nearest != 0.0 {
        (value / nearest).ceil() * nearest
    } else {
        value
    }
}

/// Round to the nearest increment
fn round_to_nearest(value: f64, nearest: f64) -> f64 {
    if nearest != 0.0 {
        (value / nearest).round() * nearest
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block(vsize: f64, median_fee: f64) -> MempoolBlock {
        MempoolBlock {
            block_size: vsize as u64,
            block_vsize: vsize,
            n_tx: 1000,
            total_fees: 1000000,
            median_fee,
            fee_range: vec![1.0, 2.0, 3.0, median_fee, 5.0, 6.0, 7.0],
        }
    }

    #[test]
    fn test_empty_mempool() {
        let estimator = FeeEstimator::new(FeeEstimationConfig::bitcoin());
        let fees = estimator.calculate_recommended_fees(&[], 0.00001);

        assert_eq!(fees.fastest_fee, 1.0);
        assert_eq!(fees.half_hour_fee, 1.0);
        assert_eq!(fees.hour_fee, 1.0);
        assert_eq!(fees.economy_fee, 1.0);
        assert_eq!(fees.minimum_fee, 1.0);
    }

    #[test]
    fn test_sub_sat_mempool() {
        let estimator = FeeEstimator::new(FeeEstimationConfig::bitcoin());

        // Use median fee slightly above 1.0 (like the real mempool data: 1.002...)
        // This tests the rounding behavior
        let blocks = vec![
            create_test_block(997953.25, 1.002), // Rounds up to 2
            create_test_block(997963.0, 0.6),
            create_test_block(997821.25, 0.52),
        ];

        let fees = estimator.calculate_recommended_fees(&blocks, 0.000001);

        assert_eq!(fees.fastest_fee, 2.0);
        assert_eq!(fees.half_hour_fee, 1.0);
        assert_eq!(fees.hour_fee, 1.0);
        assert_eq!(fees.economy_fee, 1.0);
        assert_eq!(fees.minimum_fee, 1.0);
    }

    #[test]
    fn test_low_fee_mempool() {
        let estimator = FeeEstimator::new(FeeEstimationConfig::bitcoin());

        let blocks = vec![
            create_test_block(997953.25, 2.0),
            create_test_block(997963.0, 1.5),
            create_test_block(997821.25, 1.0),
        ];

        let fees = estimator.calculate_recommended_fees(&blocks, 0.00001);

        assert_eq!(fees.fastest_fee, 2.0);
        assert_eq!(fees.half_hour_fee, 2.0);
        assert_eq!(fees.hour_fee, 2.0);
        assert_eq!(fees.economy_fee, 2.0);
        assert_eq!(fees.minimum_fee, 1.0);
    }

    #[test]
    fn test_partially_full_block() {
        let estimator = FeeEstimator::new(FeeEstimationConfig::bitcoin());

        // Block that's 75% full (750000 vsize)
        let blocks = vec![create_test_block(750_000.0, 10.0)];

        let fees = estimator.calculate_recommended_fees(&blocks, 0.00001);

        // Fee should be scaled down because block isn't full and there's no next block
        // multiplier = (750000 - 500000) / 500000 = 0.5
        // So fee should be 10 * 0.5 = 5, rounded to 5
        assert_eq!(fees.fastest_fee, 5.0);
    }

    #[test]
    fn test_liquid_config() {
        let estimator = FeeEstimator::new(FeeEstimationConfig::liquid());
        let fees = estimator.calculate_recommended_fees(&[], 0.000001);

        // Liquid uses 0.1 as minimum
        assert_eq!(fees.minimum_fee, 0.1);
    }

    #[test]
    fn test_round_up_to_nearest() {
        assert_eq!(round_up_to_nearest(1.1, 1.0), 2.0);
        assert_eq!(round_up_to_nearest(1.0, 1.0), 1.0);
        assert_eq!(round_up_to_nearest(0.15, 0.1), 0.2);
        assert_eq!(round_up_to_nearest(5.0, 0.0), 5.0);
    }

    #[test]
    fn test_round_to_nearest() {
        assert_eq!(round_to_nearest(1.4, 1.0), 1.0);
        assert_eq!(round_to_nearest(1.6, 1.0), 2.0);
        assert_eq!(round_to_nearest(0.14, 0.1), 0.1);
        assert_eq!(round_to_nearest(0.16, 0.1), 0.2);
    }
}
