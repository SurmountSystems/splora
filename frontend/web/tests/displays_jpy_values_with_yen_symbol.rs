//! Cypress `it('displays JPY values with yen symbol')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`.
//!
//! After Fiat mode and JPY are selected, the second `.table-cell-fiat` cell
//! is read and trimmed. The visible Cypress assertion is
//! `expect(trimmedText).to.include('¥');`.
//! The sample amount `1234.5` is caller-supplied. The painted text must
//! include the yen sign `¥` (U+00A5) for that amount. This does not read an
//! indexer body and it does not add a price route.
//! `paint_displays_jpy_values_with_yen_symbol` is not in this crate.
//! The call is the missing paint.

#[test]
fn displays_jpy_values_with_yen_symbol() {
    let markup = splora_web::paint_displays_jpy_values_with_yen_symbol("1234.5");
    let trimmed = markup.trim();
    assert!(
        trimmed.contains('¥'),
        "displays JPY values with yen symbol: caller-supplied 1234.5 must include ¥ (U+00A5), got {trimmed:?}"
    );
}
