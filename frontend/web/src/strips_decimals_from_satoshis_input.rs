//! Painted satoshis input text. The typed field is caller-supplied.
//! This is not a live indexer price.

use splora_frontend_shared::strips_decimals_from_satoshis_input;

/// Returns the shared satoshis-decimal strip text for a typed field.
pub fn paint_strips_decimals_from_satoshis_input(input: &str) -> String {
    strips_decimals_from_satoshis_input(input)
}
