//! Row 69. Cypress it() title: displays JPY values without decimal places.
//!
//! Mainnet spec
//! `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! reads the second `.table-cell-fiat` cell, trims the text, and expects
//! that text not to match `/\.\d+$/` (the assertion inside
//! `it('displays JPY values without decimal places')`).
//! The amount is a caller-supplied string. This test does not format that
//! string and it does not compute a yen amount.
//! This check does not fetch a price and it does not add a price route.
//! `paint_displays_jpy_values_without_decimal_places` is not defined on
//! `splora_native`.

const CYPRESS_IT: &str = "displays JPY values without decimal places";

/// Caller-supplied JPY amount text. It still has a fractional digit.
/// The Cypress check reads the paint result, not this argument.
const SAMPLE_JPY_AMOUNT: &str = "1234.5";

#[test]
fn row69_displays_jpy_values_without_decimal_places() {
    let painted: String =
        splora_native::paint_displays_jpy_values_without_decimal_places(SAMPLE_JPY_AMOUNT);
    let trimmed = painted.trim();
    let ends_with_decimal_digits = match trimmed.rsplit_once('.') {
        Some((_, frac)) => !frac.is_empty() && frac.bytes().all(|byte| byte.is_ascii_digit()),
        None => false,
    };
    assert!(
        !ends_with_decimal_digits,
        "{CYPRESS_IT}: trimmed text must not match /\\.\\d+$/, got {trimmed:?}"
    );
}
