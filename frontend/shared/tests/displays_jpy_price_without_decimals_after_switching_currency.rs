//! Cypress it() title: displays JPY price without decimals after switching currency.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! lines 220-236, inside describe('Calculator Page'). After `selectCurrency('JPY')`,
//! the trimmed calculator `.symbol` text includes `¥`. The bitcoin field is
//! cleared and `1` is typed. The fiat field is `formatFiat` of that bitcoin
//! amount times the caller-supplied fiat-per-BTC rate. When
//! `getCurrencyDecimals()` is 0, `formatFiat` returns `Math.round(num)`. JPY
//! `maximumFractionDigits` is 0, so the fiat value does not match `/\.\d+$/`.
//! The rate is an argument. This does not fetch a URL and does not hard-code
//! a display amount.

#[test]
fn displays_jpy_price_without_decimals_after_switching_currency() {
    let cases = [
        ("1", "100.4", "100"),
        ("1", "100.5", "101"),
        ("2", "10.2", "20"),
    ];
    for (bitcoin, rate, expect_fiat) in cases {
        let (symbol, fiat): (String, String) =
            splora_frontend_shared::displays_jpy_price_without_decimals_after_switching_currency(
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
        assert!(
            !symbol.contains("11057757") && !symbol.contains("MOCK_BTC_PRICE"),
            "symbol text must not include a hard-coded price, got {symbol:?}"
        );
    }

    let product = include_str!("../src/fiat.rs");
    assert!(
        !product.contains("MOCK_BTC_PRICE"),
        "frontend/shared/src/fiat.rs must not contain MOCK_BTC_PRICE"
    );
    assert!(
        !product.contains("11057757"),
        "frontend/shared/src/fiat.rs must not contain 11057757"
    );
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
