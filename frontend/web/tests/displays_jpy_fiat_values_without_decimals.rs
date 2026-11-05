//! Cypress `it('displays JPY fiat values without decimals')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! lines 129-145.
//!
//! After JPY and fiat mode, the transaction-details `.fiat` cell and the
//! first transactions-list `.fiat` cell, each trimmed, include `¥` and do
//! not match `/\.\d+/`. This does not assert a price, does not assert amount
//! digits, and does not fetch a transaction URL. It does not add `/v1/prices`.
//! `paint_displays_jpy_fiat_values_without_decimals` is not in this crate.
//! The call is the missing paint. The paint forwards the shared return.

#[test]
fn displays_jpy_fiat_values_without_decimals() {
    let cells: [String; 2] = splora_web::paint_displays_jpy_fiat_values_without_decimals();
    let details = cells[0].trim();
    let list = cells[1].trim();
    assert!(
        contains_yen(details),
        "transaction-details fiat includes ¥ after trim, got {details:?}"
    );
    assert!(
        !has_decimal_fraction(details),
        "transaction-details fiat must not match /\\.\\d+/, got {details:?}"
    );
    assert!(
        contains_yen(list),
        "transactions-list fiat includes ¥ after trim, got {list:?}"
    );
    assert!(
        !has_decimal_fraction(list),
        "transactions-list fiat must not match /\\.\\d+/, got {list:?}"
    );
}

/// Cypress `¥`: yen sign U+00A5, or the fullwidth yen sign U+FFE5.
fn contains_yen(text: &str) -> bool {
    text.contains('\u{00A5}') || text.contains('\u{FFE5}')
}

/// Cypress `/\.\d+/`: a `.` followed by one or more digits, anywhere.
fn has_decimal_fraction(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| *byte == b'.' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit))
}
