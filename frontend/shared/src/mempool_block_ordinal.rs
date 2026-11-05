//! Mempool block ordinal titles, from mempool `getOrdinal`.
//!
//! Index 0 is "Next Block" and does not read virtual size. The last kept
//! index is `KEEP_BLOCKS_AMOUNT - 1`. That slot is a stack when virtual size
//! ceils to more than one block. Every other index is one numbered block.

/// Default block weight (`BLOCK_WEIGHT_UNITS`).
const BLOCK_WEIGHT_UNITS: u64 = 4_000_000;

/// Virtual-size limit is block weight divided by 4.
const BLOCK_VSIZE_LIMIT: u64 = BLOCK_WEIGHT_UNITS / 4;

/// Preview slots the mempool keeps (`KEEP_BLOCKS_AMOUNT`).
const KEEP_BLOCKS_AMOUNT: u64 = 8;

/// Title for projected mempool block `index` of virtual size `block_vsize`.
///
/// Index 0 returns before `block_vsize` is read.
pub fn mempool_block_ordinal_title(index: u64, block_vsize: u64) -> String {
    if index == 0 {
        return "Next Block".to_string();
    }
    let last_kept_index = KEEP_BLOCKS_AMOUNT - 1;
    let blocks_in_block = block_vsize.div_ceil(BLOCK_VSIZE_LIMIT);
    if index == last_kept_index && blocks_in_block > 1 {
        format!("Stack of {blocks_in_block} mempool blocks")
    } else {
        format!("Mempool block {}", index + 1)
    }
}
