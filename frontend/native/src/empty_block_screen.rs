//! Painted heading for the block screen. The shared function decides the text
//! from this block's transaction count. This module does not invent a count.

use splora_frontend_shared::wire::Block;

/// Heading painted for `shows empty blocks properly`.
///
/// `path` is the block route the test visits. The returned text is only
/// `empty_block_heading_from_tx_count` of `block.tx_count`.
pub fn paint_shows_empty_blocks_properly(path: &str, block: &Block) -> String {
    let _ = path;
    splora_frontend_shared::empty_block_heading_from_tx_count(block.tx_count)
}
