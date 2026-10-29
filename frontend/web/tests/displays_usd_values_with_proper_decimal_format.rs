//! Cypress `it('displays USD values with proper decimal format')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`.
//!
//! After Fiat mode and USD are selected, the second `.table-cell-fiat` cell
//! is read and trimmed. The visible Cypress assertion is
//! `expect(trimmedText).to.match(/^\$[\d,]+(\.\d{2})?$/);`.
//! The sample amount `1234.5` is caller-supplied. It has one fractional
//! digit, so proper USD format is `$` before the digits and exactly two
//! cent digits (`$1234.50` or `$1,234.50`). This does not read an indexer
//! body and it does not add a price route.
//! `paint_displays_usd_values_with_proper_decimal_format` is not in this crate.

#[test]
fn displays_usd_values_with_proper_decimal_format() {
    let markup = splora_web::paint_displays_usd_values_with_proper_decimal_format("1234.5");
    let trimmed = markup.trim();
    let proper = trimmed == "$1234.50" || trimmed == "$1,234.50";
    assert!(
        proper,
        "displays USD values with proper decimal format: caller-supplied 1234.5 must match /^\\$[\\d,]+(\\.\\d{{2}})?$/ as $1234.50 or $1,234.50, got {trimmed:?}"
    );
}
