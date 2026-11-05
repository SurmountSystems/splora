//! Leptos paint of the status-screen mempool-size label.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that label, a byte count, a unit, a colon, or a block count.

/// Paints the mempool-size label. The string is the shared function.
pub fn paint_loads_the_status_screen_mempool_size() -> &'static str {
    splora_frontend_shared::loads_the_status_screen_mempool_size()
}
