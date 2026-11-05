//! Web paint of the first mempool block click after the skeleton is gone.
//! The view is `splora_frontend_shared::load_first_mempool_block_after_skeleton_loads`.

/// Forwards the shared click of `#mempool-block-0 > .blockLink`.
pub fn paint_load_first_mempool_block_after_skeleton_loads()
-> splora_frontend_shared::LoadedFirstMempoolBlock {
    splora_frontend_shared::load_first_mempool_block_after_skeleton_loads()
}
