//! Cypress it() title: displays USD fiat values with dollar symbol.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! lines 116-127. After USD and fiat mode, the transaction-details `.fiat`
//! cell and the first transactions-list `.fiat` cell, trimmed, each include
//! `$`. This does not assert digits. It does not fetch a transaction URL.

#[test]
fn displays_usd_fiat_values_with_dollar_symbol() {
    let cells: [String; 2] = splora_frontend_shared::displays_usd_fiat_values_with_dollar_symbol();
    let details = cells[0].trim();
    let list = cells[1].trim();
    assert!(
        !details.is_empty(),
        "transaction-details fiat cell is non-empty after trim"
    );
    assert!(
        details.contains('$'),
        "transaction-details fiat cell includes `$`, got {details:?}"
    );
    assert!(
        !list.is_empty(),
        "transactions-list fiat cell is non-empty after trim"
    );
    assert!(
        list.contains('$'),
        "transactions-list fiat cell includes `$`, got {list:?}"
    );
}
