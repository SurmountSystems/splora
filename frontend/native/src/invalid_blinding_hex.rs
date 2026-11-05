//! Native paint of an invalid blinding-data hex fragment.
//! This module does not open a window.

/// Forwards the fragment into the shared invalid blinding data hex check.
pub fn paint_invalid_blinding_data_hex_error(fragment: &str) -> String {
    splora_frontend_shared::invalid_blinding_data_hex_error(fragment)
}
