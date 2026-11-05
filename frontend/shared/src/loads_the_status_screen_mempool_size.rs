//! Static English label in the status footer.
//!
//! Cypress matches `/Mempool size:(.*) (kB|MB) \((\d+) (block|blocks)\)/`.
//! The text node is `Mempool size`. The colon is a separate node. The
//! interpolated size, unit, and block count are not this string.

/// Static label `Mempool size` from the status footer.
pub fn loads_the_status_screen_mempool_size() -> &'static str {
    "Mempool size"
}
