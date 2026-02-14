//! Block template construction (GetBlockTemplate) algorithm.
//!
//! This module implements an approximation of the transaction selection algorithm
//! from Bitcoin Core's BlockAssembler to create projected mempool blocks.
//!
//! Ported from mempool's Rust GBT implementation.

use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::chain::Txid;
use crate::util::fees::TxFeeInfo;

/// Default block weight limit (4MB weight = 1MB vsize for worst case)
pub const DEFAULT_BLOCK_WEIGHT: u32 = 4_000_000;
/// Maximum sigops per block
const BLOCK_SIGOPS: u32 = 80_000;
/// Reserved weight for coinbase
const BLOCK_RESERVED_WEIGHT: u32 = 4_000;
/// Reserved sigops for coinbase
const BLOCK_RESERVED_SIGOPS: u32 = 400;

/// A projected mempool block with fee statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolBlock {
    /// Total size of transactions in bytes
    pub block_size: u64,
    /// Total virtual size of transactions (weight/4)
    pub block_vsize: f64,
    /// Number of transactions
    pub n_tx: usize,
    /// Total fees in satoshis
    pub total_fees: u64,
    /// Median fee rate in sat/vB
    pub median_fee: f64,
    /// Fee rate range [min, 10th, 25th, 50th, 75th, 90th, max] in sat/vB
    pub fee_range: Vec<f64>,
}

impl Default for MempoolBlock {
    fn default() -> Self {
        Self {
            block_size: 0,
            block_vsize: 0.0,
            n_tx: 0,
            total_fees: 0,
            median_fee: 0.0,
            fee_range: vec![0.0; 7],
        }
    }
}

/// Transaction data for block template construction
#[derive(Debug, Clone)]
pub struct GbtTransaction {
    pub txid: Txid,
    pub fee: u64,
    pub weight: u32,
    pub sigops: u32,
    /// Indices of parent transactions in the mempool (by txid)
    pub parents: Vec<Txid>,
}

impl GbtTransaction {
    pub fn new(txid: Txid, fee_info: &TxFeeInfo, parents: Vec<Txid>) -> Self {
        Self {
            txid,
            fee: fee_info.fee,
            weight: fee_info.weight,
            sigops: fee_info.sigops,
            parents,
        }
    }

    #[inline]
    pub fn vsize(&self) -> u32 {
        self.weight.div_ceil(4)
    }

    #[inline]
    pub fn fee_rate(&self) -> f64 {
        self.fee as f64 / self.vsize() as f64
    }

    /// Calculate sigop-adjusted vsize (rounded up)
    #[inline]
    pub fn sigop_adjusted_vsize(&self) -> u32 {
        self.vsize().max(self.sigops * 5)
    }

    /// Calculate sigop-adjusted weight
    #[inline]
    pub fn sigop_adjusted_weight(&self) -> u32 {
        self.weight.max(self.sigops * 20)
    }
}

/// Internal audit transaction for GBT algorithm
#[derive(Debug, Clone)]
struct AuditTransaction {
    fee: u64,
    weight: u32,
    sigop_adjusted_weight: u32,
    sigop_adjusted_vsize: u32,
    sigops: u32,
    effective_fee_per_vsize: f64,
    parents: Vec<Txid>,
    ancestors: HashSet<Txid>,
    children: HashSet<Txid>,
    ancestor_fee: u64,
    ancestor_sigop_adjusted_weight: u32,
    ancestor_sigop_adjusted_vsize: u32,
    ancestor_sigops: u32,
    score: f64,
    used: bool,
    modified: bool,
}

impl AuditTransaction {
    fn from_gbt_tx(tx: &GbtTransaction) -> Self {
        let sigop_adjusted_vsize = tx.sigop_adjusted_vsize();
        let sigop_adjusted_weight = tx.sigop_adjusted_weight();
        let fee_per_vsize = tx.fee_rate();

        Self {
            fee: tx.fee,
            weight: tx.weight,
            sigop_adjusted_weight,
            sigop_adjusted_vsize,
            sigops: tx.sigops,
            effective_fee_per_vsize: fee_per_vsize,
            parents: tx.parents.clone(),
            ancestors: HashSet::new(),
            children: HashSet::new(),
            ancestor_fee: tx.fee,
            ancestor_sigop_adjusted_weight: sigop_adjusted_weight,
            ancestor_sigop_adjusted_vsize: sigop_adjusted_vsize,
            ancestor_sigops: tx.sigops,
            score: fee_per_vsize,
            used: false,
            modified: false,
        }
    }

    #[inline]
    fn ancestor_score(&self) -> f64 {
        if self.ancestor_sigop_adjusted_vsize == 0 {
            0.0
        } else {
            self.ancestor_fee as f64 / self.ancestor_sigop_adjusted_vsize as f64
        }
    }

    fn update_score(&mut self) {
        self.score = self.ancestor_score();
    }
}

/// Priority entry for the modified transactions queue
#[derive(Debug, Clone)]
struct TxPriority {
    txid: Txid,
    score: f64,
}

impl PartialEq for TxPriority {
    fn eq(&self, other: &Self) -> bool {
        self.txid == other.txid
    }
}

impl Eq for TxPriority {}

impl PartialOrd for TxPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TxPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher score = higher priority (reverse order for BinaryHeap)
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Result of the GBT algorithm
#[derive(Debug)]
pub struct GbtResult {
    /// Projected blocks, each containing transaction IDs
    pub blocks: Vec<Vec<Txid>>,
    /// Statistics for each projected block
    pub block_stats: Vec<MempoolBlock>,
}

/// Build projected mempool blocks using an approximation of Bitcoin Core's transaction selection.
///
/// Returns up to `max_blocks` projected blocks with their fee statistics.
pub fn build_projected_blocks(
    transactions: &[GbtTransaction],
    max_block_weight: u32,
    max_blocks: usize,
) -> GbtResult {
    if transactions.is_empty() || max_blocks == 0 {
        return GbtResult {
            blocks: vec![],
            block_stats: vec![],
        };
    }

    // Build audit pool indexed by txid
    let mut audit_pool: HashMap<Txid, AuditTransaction> = transactions
        .iter()
        .map(|tx| (tx.txid, AuditTransaction::from_gbt_tx(tx)))
        .collect();

    // Set up ancestor/descendant relationships
    let txids: Vec<Txid> = audit_pool.keys().cloned().collect();
    for txid in &txids {
        set_relatives(txid, &mut audit_pool);
    }

    // Sort by descending ancestor score
    let mut mempool_stack: Vec<Txid> = txids;
    mempool_stack.sort_by(|a, b| {
        let score_a = audit_pool.get(a).map(|tx| tx.score).unwrap_or(0.0);
        let score_b = audit_pool.get(b).map(|tx| tx.score).unwrap_or(0.0);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Build blocks
    let mut blocks: Vec<Vec<Txid>> = Vec::new();
    let mut block_fee_rates: Vec<Vec<f64>> = Vec::new();
    let mut current_block: Vec<Txid> = Vec::new();
    let mut current_fee_rates: Vec<f64> = Vec::new();
    let mut block_weight: u32 = BLOCK_RESERVED_WEIGHT;
    let mut block_sigops: u32 = BLOCK_RESERVED_SIGOPS;
    #[allow(unused_variables)]
    let mut block_fees: u64 = 0;

    let mut modified: BinaryHeap<TxPriority> = BinaryHeap::new();
    let mut overflow: Vec<Txid> = Vec::new();
    let mut failures = 0;

    while (!mempool_stack.is_empty() || !modified.is_empty()) && blocks.len() < max_blocks {
        // Get next best transaction from either stack or modified queue
        let next_txid = get_next_tx(&mut mempool_stack, &mut modified, &audit_pool);

        let next_txid = match next_txid {
            Some(txid) => txid,
            None => break,
        };

        let (ancestor_weight, ancestor_sigops, _ancestor_fee, ancestor_score) = {
            let tx = match audit_pool.get(&next_txid) {
                Some(tx) if !tx.used => tx,
                _ => continue,
            };
            (
                tx.ancestor_sigop_adjusted_weight,
                tx.ancestor_sigops,
                tx.ancestor_fee,
                tx.score,
            )
        };

        // Check if this package fits in the current block
        if blocks.len() < max_blocks - 1
            && (block_weight + ancestor_weight >= max_block_weight - BLOCK_RESERVED_WEIGHT
                || block_sigops + ancestor_sigops > BLOCK_SIGOPS)
        {
            overflow.push(next_txid);
            failures += 1;
        } else {
            // Add the package (ancestors + this transaction) to the block
            let package = get_package(&next_txid, &audit_pool);

            for pkg_txid in &package {
                if let Some(tx) = audit_pool.get_mut(pkg_txid) {
                    if !tx.used {
                        tx.used = true;
                        current_block.push(*pkg_txid);
                        current_fee_rates.push(tx.effective_fee_per_vsize);
                        block_weight += tx.sigop_adjusted_weight;
                        block_sigops += tx.sigops;
                        block_fees += tx.fee;
                    }
                }
            }

            // Update descendants
            update_descendants(&next_txid, &mut audit_pool, &mut modified, ancestor_score);
            failures = 0;
        }

        // Check if block is full
        let exceeded_tries =
            failures > 1000 && block_weight > (max_block_weight - BLOCK_RESERVED_WEIGHT - 4_000);
        let queues_empty = mempool_stack.is_empty() && modified.is_empty();

        if (exceeded_tries || queues_empty)
            && blocks.len() < max_blocks - 1
            && !current_block.is_empty()
        {
            blocks.push(std::mem::take(&mut current_block));
            block_fee_rates.push(std::mem::take(&mut current_fee_rates));
            block_weight = BLOCK_RESERVED_WEIGHT;
            block_sigops = BLOCK_RESERVED_SIGOPS;
            block_fees = 0;
            failures = 0;

            // Move overflow back to processing
            overflow.reverse();
            for txid in overflow.drain(..) {
                if let Some(tx) = audit_pool.get(&txid) {
                    if tx.modified {
                        modified.push(TxPriority {
                            txid,
                            score: tx.score,
                        });
                    } else {
                        mempool_stack.push(txid);
                    }
                }
            }
        }
    }

    // Add final block if not empty
    if !current_block.is_empty() {
        blocks.push(current_block);
        block_fee_rates.push(current_fee_rates);
    }

    // Calculate block statistics
    let block_stats: Vec<MempoolBlock> = blocks
        .iter()
        .zip(block_fee_rates.iter())
        .map(|(block_txids, fee_rates)| calculate_block_stats(block_txids, fee_rates, &audit_pool))
        .collect();

    GbtResult {
        blocks,
        block_stats,
    }
}

fn get_next_tx(
    mempool_stack: &mut Vec<Txid>,
    modified: &mut BinaryHeap<TxPriority>,
    audit_pool: &HashMap<Txid, AuditTransaction>,
) -> Option<Txid> {
    loop {
        // Get candidates from both queues
        let stack_candidate = mempool_stack.last().and_then(|txid| {
            audit_pool
                .get(txid)
                .filter(|tx| !tx.used && !tx.modified)
                .map(|tx| (*txid, tx.score))
        });

        let modified_candidate = modified.peek().and_then(|priority| {
            audit_pool
                .get(&priority.txid)
                .filter(|tx| !tx.used)
                .map(|tx| (priority.txid, tx.score))
        });

        match (stack_candidate, modified_candidate) {
            (Some((stack_txid, stack_score)), Some((mod_txid, mod_score))) => {
                if mod_score >= stack_score {
                    modified.pop();
                    return Some(mod_txid);
                } else {
                    mempool_stack.pop();
                    return Some(stack_txid);
                }
            }
            (Some((txid, _)), None) => {
                mempool_stack.pop();
                return Some(txid);
            }
            (None, Some((txid, _))) => {
                modified.pop();
                return Some(txid);
            }
            (None, None) => {
                // Try to clean up invalid entries
                if mempool_stack.pop().is_some() {
                    continue;
                }
                if modified.pop().is_some() {
                    continue;
                }
                return None;
            }
        }
    }
}

fn set_relatives(txid: &Txid, audit_pool: &mut HashMap<Txid, AuditTransaction>) {
    // Get parents for this transaction
    let parents: Vec<Txid> = match audit_pool.get(txid) {
        Some(tx) => tx
            .parents
            .iter()
            .filter(|p| audit_pool.contains_key(*p))
            .cloned()
            .collect(),
        None => return,
    };

    // Recursively set relatives for parents first
    for parent_txid in &parents {
        if audit_pool
            .get(parent_txid)
            .map(|tx| tx.ancestors.is_empty() && !tx.parents.is_empty())
            .unwrap_or(false)
        {
            set_relatives(parent_txid, audit_pool);
        }
    }

    // Collect ancestor info
    let mut ancestors: HashSet<Txid> = HashSet::new();
    let mut total_fee: u64 = 0;
    let mut total_sigop_adjusted_weight: u32 = 0;
    let mut total_sigop_adjusted_vsize: u32 = 0;
    let mut total_sigops: u32 = 0;

    for parent_txid in &parents {
        if let Some(parent) = audit_pool.get(parent_txid) {
            ancestors.insert(*parent_txid);
            for ancestor in &parent.ancestors {
                ancestors.insert(*ancestor);
            }
            total_fee += parent.fee;
            total_sigop_adjusted_weight += parent.sigop_adjusted_weight;
            total_sigop_adjusted_vsize += parent.sigop_adjusted_vsize;
            total_sigops += parent.sigops;
        }
    }

    // Add ancestor stats from indirect ancestors
    for ancestor_txid in &ancestors {
        if !parents.contains(ancestor_txid) {
            if let Some(ancestor) = audit_pool.get(ancestor_txid) {
                total_fee += ancestor.fee;
                total_sigop_adjusted_weight += ancestor.sigop_adjusted_weight;
                total_sigop_adjusted_vsize += ancestor.sigop_adjusted_vsize;
                total_sigops += ancestor.sigops;
            }
        }
    }

    // Update the transaction
    if let Some(tx) = audit_pool.get_mut(txid) {
        tx.ancestors = ancestors;
        tx.ancestor_fee += total_fee;
        tx.ancestor_sigop_adjusted_weight += total_sigop_adjusted_weight;
        tx.ancestor_sigop_adjusted_vsize += total_sigop_adjusted_vsize;
        tx.ancestor_sigops += total_sigops;
        tx.update_score();
    }

    // Update children of parents
    for parent_txid in parents {
        if let Some(parent) = audit_pool.get_mut(&parent_txid) {
            parent.children.insert(*txid);
        }
    }
}

fn get_package(txid: &Txid, audit_pool: &HashMap<Txid, AuditTransaction>) -> Vec<Txid> {
    let mut package: Vec<(Txid, usize)> = Vec::new();

    if let Some(tx) = audit_pool.get(txid) {
        // Add ancestors first, sorted by ancestor count (so parents come before children)
        for ancestor_txid in &tx.ancestors {
            if let Some(ancestor) = audit_pool.get(ancestor_txid) {
                if !ancestor.used {
                    package.push((*ancestor_txid, ancestor.ancestors.len()));
                }
            }
        }
        package.sort_by_key(|(_, count)| *count);

        // Add the transaction itself
        package.push((*txid, tx.ancestors.len()));
    }

    package.into_iter().map(|(txid, _)| txid).collect()
}

fn update_descendants(
    root_txid: &Txid,
    audit_pool: &mut HashMap<Txid, AuditTransaction>,
    modified: &mut BinaryHeap<TxPriority>,
    cluster_rate: f64,
) {
    let (root_fee, root_sigop_adjusted_weight, root_sigop_adjusted_vsize, root_sigops, children) = {
        match audit_pool.get(root_txid) {
            Some(tx) => (
                tx.fee,
                tx.sigop_adjusted_weight,
                tx.sigop_adjusted_vsize,
                tx.sigops,
                tx.children.clone(),
            ),
            None => return,
        }
    };

    let mut visited: HashSet<Txid> = HashSet::new();
    let mut stack: Vec<Txid> = children.into_iter().collect();

    while let Some(desc_txid) = stack.pop() {
        if visited.contains(&desc_txid) {
            continue;
        }
        visited.insert(desc_txid);

        let children_to_add: Vec<Txid>;
        let old_score: f64;
        let new_score: f64;

        {
            let descendant = match audit_pool.get_mut(&desc_txid) {
                Some(tx) => tx,
                None => continue,
            };

            old_score = descendant.score;

            // Remove root from ancestors
            descendant.ancestors.remove(root_txid);
            descendant.ancestor_fee = descendant.ancestor_fee.saturating_sub(root_fee);
            descendant.ancestor_sigop_adjusted_weight = descendant
                .ancestor_sigop_adjusted_weight
                .saturating_sub(root_sigop_adjusted_weight);
            descendant.ancestor_sigop_adjusted_vsize = descendant
                .ancestor_sigop_adjusted_vsize
                .saturating_sub(root_sigop_adjusted_vsize);
            descendant.ancestor_sigops = descendant.ancestor_sigops.saturating_sub(root_sigops);

            // Update effective fee rate based on cluster rate
            if cluster_rate < descendant.effective_fee_per_vsize {
                descendant.effective_fee_per_vsize = cluster_rate;
            }

            descendant.update_score();
            new_score = descendant.score;

            children_to_add = descendant.children.iter().cloned().collect();
        }

        // Add to modified queue if score changed
        if (new_score - old_score).abs() > f64::EPSILON {
            if let Some(tx) = audit_pool.get_mut(&desc_txid) {
                tx.modified = true;
            }
            modified.push(TxPriority {
                txid: desc_txid,
                score: new_score,
            });
        }

        // Add children to stack
        for child in children_to_add {
            if !visited.contains(&child) {
                stack.push(child);
            }
        }
    }
}

fn calculate_block_stats(
    txids: &[Txid],
    fee_rates: &[f64],
    audit_pool: &HashMap<Txid, AuditTransaction>,
) -> MempoolBlock {
    if txids.is_empty() {
        return MempoolBlock::default();
    }

    let mut total_size: u64 = 0;
    let mut total_weight: u64 = 0;
    let mut total_fees: u64 = 0;

    for txid in txids {
        if let Some(tx) = audit_pool.get(txid) {
            total_weight += tx.weight as u64;
            total_size += tx.weight as u64 / 4; // Approximate size
            total_fees += tx.fee;
        }
    }

    let mut sorted_rates = fee_rates.to_vec();
    sorted_rates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted_rates.len();
    let median_fee = if n == 0 {
        0.0
    } else if n % 2 == 0 {
        (sorted_rates[n / 2 - 1] + sorted_rates[n / 2]) / 2.0
    } else {
        sorted_rates[n / 2]
    };

    // Calculate percentiles for fee range: [min, 10th, 25th, 50th, 75th, 90th, max]
    let fee_range = if n == 0 {
        vec![0.0; 7]
    } else {
        vec![
            sorted_rates[0],
            sorted_rates[(n as f64 * 0.1) as usize],
            sorted_rates[(n as f64 * 0.25) as usize],
            sorted_rates[(n as f64 * 0.5) as usize],
            sorted_rates[((n as f64 * 0.75) as usize).min(n - 1)],
            sorted_rates[((n as f64 * 0.9) as usize).min(n - 1)],
            sorted_rates[n - 1],
        ]
    };

    MempoolBlock {
        block_size: total_size,
        block_vsize: total_weight as f64 / 4.0,
        n_tx: txids.len(),
        total_fees,
        median_fee,
        fee_range,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::Hash;

    fn make_txid(n: u8) -> Txid {
        let mut bytes = [0u8; 32];
        bytes[0] = n;
        Txid::from_slice(&bytes).unwrap()
    }

    #[test]
    fn test_empty_mempool() {
        let result = build_projected_blocks(&[], DEFAULT_BLOCK_WEIGHT, 8);
        assert!(result.blocks.is_empty());
        assert!(result.block_stats.is_empty());
    }

    #[test]
    fn test_single_transaction() {
        let txid = make_txid(1);
        let tx = GbtTransaction {
            txid,
            fee: 1000,
            weight: 400,
            sigops: 1,
            parents: vec![],
        };

        let result = build_projected_blocks(&[tx], DEFAULT_BLOCK_WEIGHT, 8);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(result.blocks[0].len(), 1);
        assert_eq!(result.blocks[0][0], txid);
    }

    #[test]
    fn test_parent_child_relationship() {
        let parent_txid = make_txid(1);
        let child_txid = make_txid(2);

        let parent = GbtTransaction {
            txid: parent_txid,
            fee: 500,
            weight: 400,
            sigops: 1,
            parents: vec![],
        };

        let child = GbtTransaction {
            txid: child_txid,
            fee: 1000,
            weight: 400,
            sigops: 1,
            parents: vec![parent_txid],
        };

        let result = build_projected_blocks(&[parent, child], DEFAULT_BLOCK_WEIGHT, 8);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(result.blocks[0].len(), 2);
        // Parent should come before child
        let parent_pos = result.blocks[0].iter().position(|&t| t == parent_txid);
        let child_pos = result.blocks[0].iter().position(|&t| t == child_txid);
        assert!(parent_pos < child_pos);
    }
}
