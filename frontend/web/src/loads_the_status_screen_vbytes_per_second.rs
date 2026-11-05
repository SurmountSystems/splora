//! Leptos paint of the status-screen vbytes-per-second unit.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that unit, a rate number, or Incoming Transactions.

/// Paints the vbytes-per-second unit. The string is the shared function.
pub fn paint_loads_the_status_screen_vbytes_per_second() -> &'static str {
    splora_frontend_shared::loads_the_status_screen_vbytes_per_second()
}
