//! Static English label in the status footer.
//!
//! Cypress matches `/Mempool size:(.*) (kB|MB) \((\d+) (block|blocks)\)/`.
//! The text node is `blocks`. The interpolated count is not this string.

/// Static label `blocks` from the status footer.
pub fn loads_the_status_screen_blocks() -> &'static str {
    "blocks"
}
