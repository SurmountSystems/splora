//! Cypress `it('displays USD fiat values with dollar symbol')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! lines 116-127.
//!
//! After USD and fiat mode, the transaction-details `.fiat` cell and the
//! first transactions-list `.fiat` cell, each trimmed, include `$`.
//! This does not assert a price, does not assert amount digits, and does
//! not fetch a transaction URL. It does not add `/v1/prices`.
//! `paint_displays_usd_fiat_values_with_dollar_symbol` is not in this crate.
//! The call is the missing paint. The paint forwards the shared return.

#[test]
fn displays_usd_fiat_values_with_dollar_symbol() {
    let cells: [String; 2] = splora_web::paint_displays_usd_fiat_values_with_dollar_symbol();
    let details = cells[0].trim();
    let list = cells[1].trim();
    assert!(
        details.contains('$'),
        "transaction-details fiat includes `$` after trim, got {details:?}"
    );
    assert!(
        list.contains('$'),
        "transactions-list fiat includes `$` after trim, got {list:?}"
    );
}
