//! Static English label in the status footer.
//!
//! Cypress matches `/Unconfirmed:(.*)/`. The text node is `Unconfirmed`.
//! The colon is a separate node. The interpolated unconfirmed count is
//! not this string.

/// Static label `Unconfirmed` from the status footer.
pub fn loads_the_status_screen_unconfirmed() -> &'static str {
    "Unconfirmed"
}
