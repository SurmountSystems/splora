//! First mempool block row from a received backlog, or from no response.
//!
//! Cypress checks `#mempool-block-0 > .blockLink` after the skeleton is gone.
//! The loaded template ids that row from the ngFor index (`mempool-block-{{ i }}`).
//! That index is not `MempoolBlock.index`. A received backlog, including an
//! all-zero summary, still renders index 0 and stores that id on the view.
//! No response leaves both flags false and leaves the id empty.

/// Loaded versus still-loading first mempool block.
///
/// `link_id` is owned because the loaded id is formatted from the render index.
/// `String` is not `Copy`, so this view is not `Copy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstMempoolBlock {
    /// `.skeleton-loader` is absent. That is the loaded template branch.
    pub skeleton_gone: bool,
    /// `#mempool-block-0 > .blockLink` is in the document.
    pub block_link_exists: bool,
    /// ngFor render-index DOM id when a backlog was received. Empty otherwise.
    pub link_id: String,
}

/// Received `GET /mempool` summary (`BacklogStats` in `src/new_index/mempool.rs`).
///
/// `count`, `vsize`, and `total_fee` use that struct's integer widths.
/// `fee_histogram_len` is the length of `fee_histogram`. Zeros are a received
/// summary, not a missing response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MempoolBacklog {
    pub count: u32,
    pub vsize: u32,
    pub total_fee: u64,
    pub fee_histogram_len: usize,
}

/// Project ngFor index 0 from a received backlog.
///
/// The argument is the receipt. An all-zero summary still means the response
/// arrived, so the loaded template renders index 0, including the empty
/// placeholder. The DOM id is `mempool-block-{{ i }}` for that index, stored
/// on `link_id`. Neither flag depends on a positive count, vsize, or histogram
/// length.
pub fn first_mempool_block_from_backlog(stats: MempoolBacklog) -> FirstMempoolBlock {
    // Template `i`, not `MempoolBlock.index`. Not a named link-id constant.
    let render_index = 0;
    let link_id = Some(stats).map(|_| format!("mempool-block-{render_index}"));
    FirstMempoolBlock {
        skeleton_gone: link_id.is_some(),
        block_link_exists: link_id.is_some(),
        link_id: link_id.unwrap_or_default(),
    }
}

/// No backlog response has arrived, so the skeleton is still up.
///
/// This does not call `first_mempool_block_from_backlog`. The id stays empty.
pub fn first_mempool_block_after_skeleton() -> FirstMempoolBlock {
    FirstMempoolBlock {
        skeleton_gone: false,
        block_link_exists: false,
        link_id: String::new(),
    }
}
