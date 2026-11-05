//! Cypress it() title: formats all JPY values correctly without decimals.
//!
//! Mainnet spec
//! `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! walks every `.table-cell-fiat` cell. When the trimmed text includes `¥`,
//! that text must not match `/\.\d+/`.
//! This test does not assert a price number, does not add `/v1/prices`,
//! and does not hard-code a yen amount.
//! The cell strings come from
//! `paint_formats_all_jpy_values_correctly_without_decimals`.
//! That paint is not defined on `splora_native`. This test does not call
//! the shared function.

const CYPRESS_IT: &str = "formats all JPY values correctly without decimals";

/// True when `text` matches `/\.\d+/` (a dot followed by one or more digits).
fn matches_dot_digits(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes
        .windows(2)
        .any(|pair| pair[0] == b'.' && pair[1].is_ascii_digit())
}

#[test]
fn formats_all_jpy_values_correctly_without_decimals() {
    let cells: Vec<String> =
        splora_native::paint_formats_all_jpy_values_correctly_without_decimals();
    let mut saw_yen = false;
    for cell in &cells {
        let text = cell.trim();
        if text.contains('¥') {
            saw_yen = true;
            assert!(
                !matches_dot_digits(text),
                "{CYPRESS_IT}: trimmed cell that includes ¥ must not match /\\.\\d+/, got {text:?}"
            );
        }
    }
    assert!(
        saw_yen,
        "{CYPRESS_IT}: at least one trimmed .table-cell-fiat cell must include ¥"
    );
}
