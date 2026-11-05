//! Static English unit in the status footer.
//!
//! Cypress matches `/Incoming Transactions.* vB\/s/`. The text node is
//! `vB/s`. The interpolated vbytes-per-second number is not this string.

/// Static unit `vB/s` from the status footer.
pub fn loads_the_status_screen_vbytes_per_second() -> &'static str {
    "vB/s"
}
