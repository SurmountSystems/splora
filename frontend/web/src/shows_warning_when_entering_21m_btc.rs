//! Painted max-supply fields for typed bitcoin text.
//! The typed field is caller-supplied.
//! This is not a live indexer price.

/// Returns the shared max-supply entry for typed bitcoin text.
pub fn paint_shows_warning_when_entering_21m_btc(
    input: &str,
) -> splora_frontend_shared::MaxSupplyEntry {
    splora_frontend_shared::shows_warning_when_entering_21m_btc(input)
}
