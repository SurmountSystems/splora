//! Painted block screen for no pagination. The shared view decides the
//! heading and the first pager child count. This module does not invent a count.

use splora_frontend_shared::wire::Block;

/// Heading, first pager child count, and the block transaction path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowsBlocksWithNoPagination {
    pub heading: String,
    pub first_pagination_children: usize,
    pub txs_path: String,
}

/// Paint for `shows blocks with no pagination`.
///
/// `path` is the block route the test visits. The heading and the first
/// pager child count are `no_pagination_block_screen` of `block.tx_count`.
/// The transaction path is `block_txs_path` of `block.id`.
pub fn paint_shows_blocks_with_no_pagination(
    path: &str,
    block: &Block,
) -> ShowsBlocksWithNoPagination {
    let _ = path;
    let screen = splora_frontend_shared::no_pagination_block_screen(block.tx_count);
    ShowsBlocksWithNoPagination {
        heading: screen.heading,
        first_pagination_children: screen.first_pagination_children,
        txs_path: splora_frontend_shared::block_txs_path(&block.id),
    }
}
