//! Painted `.table-cell-fiat` texts after JPY is selected.
//! This is not a live price feed.

/// Forwards the shared JPY fiat cell texts.
pub fn paint_formats_all_jpy_values_correctly_without_decimals() -> Vec<String> {
    splora_frontend_shared::formats_all_jpy_values_correctly_without_decimals()
}
