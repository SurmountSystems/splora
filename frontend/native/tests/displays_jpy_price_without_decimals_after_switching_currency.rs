//! Cypress `it('displays JPY price without decimals after switching currency')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! lines 220-236.
//!
//! After JPY is selected, the calculator `.symbol` text includes `¥`.
//! The bitcoin field and the fiat-per-BTC rate are caller-supplied. The fiat
//! field is the rounded product and must not match `/\.\d+$/`. This does not
//! fetch a URL and does not hard-code a display amount.
//! The test calls
//! `paint_displays_jpy_price_without_decimals_after_switching_currency`.
//! The paint forwards the shared return. The expected fiat strings below are
//! compared to that return. They are not stored in the paint file.

#[test]
fn displays_jpy_price_without_decimals_after_switching_currency() {
    let cases = [
        ("1", "100.4", "100"),
        ("1", "100.5", "101"),
        ("2", "10.2", "20"),
    ];
    for (bitcoin, rate, expect_fiat) in cases {
        let (symbol, fiat): (String, String) =
            splora_native::paint_displays_jpy_price_without_decimals_after_switching_currency(
                bitcoin, rate,
            );
        assert_eq!(
            fiat, expect_fiat,
            "bitcoin {bitcoin:?} at fiat-per-btc {rate:?} rounds to {expect_fiat}, got {fiat:?}"
        );
        assert!(
            !ends_with_dot_digits(&fiat),
            "fiat field must not match /\\.\\d+$/, got {fiat:?}"
        );
        assert!(
            symbol.contains('\u{00A5}'),
            "calculator `.symbol` text includes `¥`, got {symbol:?}"
        );
    }
}

/// `/\.\d+$/`: a `.` followed by one or more ASCII digits at the end of `text`.
fn ends_with_dot_digits(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = bytes.len();
    let end = index;
    while index > 0 && bytes[index - 1].is_ascii_digit() {
        index -= 1;
    }
    if index == end || index == 0 {
        return false;
    }
    bytes[index - 1] == b'.'
}
