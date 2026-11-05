//! Painted `.table-cell-fiat` texts after JPY, then after USD.
//! This is not a live price feed.

/// Forwards the shared JPY-then-USD fiat cell texts.
pub fn paint_correctly_formats_when_switching_from_jpy_back_to_usd() -> [Vec<String>; 2] {
    splora_frontend_shared::correctly_formats_when_switching_from_jpy_back_to_usd()
}
