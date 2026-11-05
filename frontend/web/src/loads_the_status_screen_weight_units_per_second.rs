//! Leptos paint of the status-screen weight-units-per-second unit.
//!
//! The visible text is the shared function's return value. This file does
//! not hard-code that unit, a rate, or a size.

/// Paints the weight-units-per-second unit. The string is the shared function.
pub fn paint_loads_the_status_screen_weight_units_per_second() -> &'static str {
    splora_frontend_shared::loads_the_status_screen_weight_units_per_second()
}
