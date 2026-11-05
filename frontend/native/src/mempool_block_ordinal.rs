//! Native paint of a mempool block ordinal title.
//! This module does not open a window.

/// Forwards the index and the block virtual size into the shipped title.
pub fn paint_mempool_block_ordinal_title(index: u64, block_vsize: u64) -> String {
    splora_frontend_shared::mempool_block_ordinal_title(index, block_vsize)
}
