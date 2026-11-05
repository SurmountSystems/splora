//! Web paint of a mempool block ordinal title.
//! The title is `splora_frontend_shared::mempool_block_ordinal_title`.

/// Forwards the shared mempool block ordinal title.
pub fn paint_mempool_block_ordinal_title(index: u64, block_vsize: u64) -> String {
    splora_frontend_shared::mempool_block_ordinal_title(index, block_vsize)
}
