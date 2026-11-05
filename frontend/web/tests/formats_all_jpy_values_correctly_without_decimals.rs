//! Cypress `it('formats all JPY values correctly without decimals')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! lines 71-78.
//!
//! The body walks every `.table-cell-fiat` cell. It trims the text. When
//! that text includes `¥`, it must not match `/\.\d+/`. A cell without the
//! yen sign is not checked. This does not assert a price number, does not
//! hard-code a fixture yen amount, and does not add `/v1/prices`.
//! `paint_formats_all_jpy_values_correctly_without_decimals` is not in this
//! crate. The call is the missing paint. The paint forwards the shared return.

#[test]
fn formats_all_jpy_values_correctly_without_decimals() {
    let painted: Vec<String> =
        splora_web::paint_formats_all_jpy_values_correctly_without_decimals();
    let shared = splora_frontend_shared::formats_all_jpy_values_correctly_without_decimals();
    assert_eq!(
        painted, shared,
        "paint forwards splora_frontend_shared::formats_all_jpy_values_correctly_without_decimals"
    );
    let mut saw_yen = false;
    for cell in &painted {
        let trimmed = cell.trim();
        if !trimmed.contains('\u{00A5}') && !trimmed.contains('\u{FFE5}') {
            continue;
        }
        saw_yen = true;
        assert!(
            !has_decimal_fraction(trimmed),
            "table-cell-fiat text that includes ¥ must not match /\\.\\d+/, got {trimmed:?}"
        );
    }
    assert!(
        saw_yen,
        "at least one trimmed .table-cell-fiat cell includes ¥"
    );
}

/// Cypress `/\.\d+/`: a `.` followed by one or more digits, anywhere.
fn has_decimal_fraction(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| *byte == b'.' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit))
}
