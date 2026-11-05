//! Native paint of previous and next link flags for one confirmed height.
//! This module does not open a window.

/// Forwards the height pair into the shared link flags.
pub fn paint_block_prev_next_links(
    block_height: Option<u32>,
    latest_confirmed_height: u32,
) -> (bool, bool) {
    splora_frontend_shared::block_prev_next_links(block_height, latest_confirmed_height)
}
