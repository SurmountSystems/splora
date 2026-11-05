//! Web paint of confirmed block link visibility.
//! The pair is `splora_frontend_shared::block_prev_next_links`.

/// Forwards both height arguments to the shared function.
pub fn paint_block_prev_next_links(
    block_height: Option<u32>,
    latest_confirmed_height: u32,
) -> (bool, bool) {
    splora_frontend_shared::block_prev_next_links(block_height, latest_confirmed_height)
}
