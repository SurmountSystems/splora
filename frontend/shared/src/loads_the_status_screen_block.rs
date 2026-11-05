//! Static English label in the status footer.
//!
//! Cypress matches `/Mempool size:(.*) (kB|MB) \((\d+) (block|blocks)\)/`.
//! The text node is `block`. The interpolated count is not this string.

/// Static label `block` from the status footer.
pub fn loads_the_status_screen_block() -> &'static str {
    "block"
}
