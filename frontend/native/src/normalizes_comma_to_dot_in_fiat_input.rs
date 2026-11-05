//! Fiat input text after a decimal comma is rewritten as a dot. Not a live price.

/// Return the shared decimal-comma rewrite of `input`.
pub fn paint_normalizes_comma_to_dot_in_fiat_input(input: &str) -> String {
    splora_frontend_shared::decimal_comma_to_dot(input)
}
