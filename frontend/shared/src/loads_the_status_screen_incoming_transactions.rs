//! Static English label in the status footer.
//!
//! Cypress matches `/Incoming Transactions.* vB\/s/`. The span text is
//! `Incoming Transactions`. The vB/s figure on the next lines is
//! interpolated from mempool info and is not this string.

/// Static label `Incoming Transactions` from the status footer.
pub fn loads_the_status_screen_incoming_transactions() -> &'static str {
    "Incoming Transactions"
}
