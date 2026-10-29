//! Row 68. Cypress it() title: displays JPY values with yen symbol.
//!
//! Mainnet spec
//! `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! reads the second `.table-cell-fiat` cell, trims the text, and expects
//! that text to include `¥` (the assertion inside
//! `it('displays JPY values with yen symbol')`).
//! The amount and the yen mark are caller-supplied strings. This test does
//! not format those strings and it does not compute a yen amount.
//! This check does not fetch a price and it does not add a price route.
//! `paint_displays_jpy_values_with_yen_symbol` is not defined on
//! `splora_native`.

const CYPRESS_IT: &str = "displays JPY values with yen symbol";

/// Caller-supplied JPY amount text. The paint must show the yen mark.
const SAMPLE_JPY_AMOUNT: &str = "1234.5";

/// Caller-supplied mark from `expect(trimmedText).to.include('¥')`.
const YEN_MARK: &str = "¥";

#[test]
fn row68_displays_jpy_values_with_yen_symbol() {
    let painted: String =
        splora_native::paint_displays_jpy_values_with_yen_symbol(SAMPLE_JPY_AMOUNT);
    let trimmed = painted.trim();
    assert!(
        trimmed.contains(YEN_MARK),
        "{CYPRESS_IT}: trimmed text must include the caller-supplied yen mark {YEN_MARK}, got {trimmed:?}"
    );
}
