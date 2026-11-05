//! Painted fiat input text. The typed field is caller-supplied.
//! This is not a live indexer price.

use splora_frontend_shared::decimal_comma_to_dot;

/// Returns the shared normalizer's text for a fiat field.
pub fn paint_normalizes_comma_to_dot_in_fiat_input(input: &str) -> String {
    decimal_comma_to_dot(input)
}
