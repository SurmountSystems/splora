//! Cypress it() title: displays USD values with proper decimal format.
//!
//! Mainnet spec
//! `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! reads the second `.table-cell-fiat` cell, trims the text, and expects
//! that text to match `/^\$[\d,]+(\.\d{2})?$/` (the assertion on the
//! `it('displays USD values with proper decimal format')` body).
//! The sample amount `1234.5` is caller-supplied. It has one fractional
//! digit, so proper USD format is `$` before the digits and exactly two
//! cent digits (`$1234.50` or `$1,234.50`). This check does not fetch a
//! price and it does not add a price route.
//! `paint_displays_usd_values_with_proper_decimal_format` is not defined
//! on `splora_native`.

const CYPRESS_IT: &str = "displays USD values with proper decimal format";

/// Sample USD amount text. One fractional digit must paint as two cent digits.
const SAMPLE_USD_AMOUNT: &str = "1234.5";

#[test]
fn displays_usd_values_with_proper_decimal_format() {
    let painted: String =
        splora_native::paint_displays_usd_values_with_proper_decimal_format(SAMPLE_USD_AMOUNT);
    let trimmed = painted.trim();
    let proper = trimmed == "$1234.50" || trimmed == "$1,234.50";
    assert!(
        proper,
        "{CYPRESS_IT}: caller-supplied 1234.5 must match /^\\$[\\d,]+(\\.\\d{{2}})?$/ as $1234.50 or $1,234.50, got {trimmed:?}"
    );
}
