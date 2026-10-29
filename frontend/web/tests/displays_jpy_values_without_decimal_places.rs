//! Cypress `it('displays JPY values without decimal places')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`.
//!
//! After Fiat mode and JPY are selected, the second `.table-cell-fiat` cell
//! is read and trimmed. The visible Cypress assertion is
//! `expect(trimmedText).to.not.match(/\.\d+$/);`.
//! The sample amount `1234.5` is caller-supplied. The painted text must
//! not end with a decimal point and digits. This does not read an indexer
//! body and it does not add a price route.
//! `paint_displays_jpy_values_without_decimal_places` is not in this crate.
//! The call is the missing paint.

#[test]
fn displays_jpy_values_without_decimal_places() {
    let markup = splora_web::paint_displays_jpy_values_without_decimal_places("1234.5");
    let trimmed = markup.trim();
    let ends_with_decimal_digits = match trimmed.rsplit_once('.') {
        Some((_, frac)) => !frac.is_empty() && frac.bytes().all(|byte| byte.is_ascii_digit()),
        None => false,
    };
    assert!(
        !ends_with_decimal_digits,
        "displays JPY values without decimal places: caller-supplied 1234.5 must not match /\\.\\d+$/, got {trimmed:?}"
    );
}
