//! Painted `.table-cell-fiat` texts after USD, then after JPY.
//! This is not a live price feed.

/// Forwards the shared USD-then-JPY fiat cell texts.
pub fn paint_correctly_formats_when_switching_from_usd_to_jpy() -> [Vec<String>; 2] {
    splora_frontend_shared::correctly_formats_when_switching_from_usd_to_jpy()
}
