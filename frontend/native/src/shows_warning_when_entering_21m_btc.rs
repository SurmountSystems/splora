//! Native paint for typed bitcoin that reaches the supply cap. Delegates to the shared cap.
//! This file does not open a window.

/// Return the shared max-supply fields for typed bitcoin text `input`.
pub fn paint_shows_warning_when_entering_21m_btc(
    input: &str,
) -> splora_frontend_shared::MaxSupplyEntry {
    splora_frontend_shared::shows_warning_when_entering_21m_btc(input)
}
