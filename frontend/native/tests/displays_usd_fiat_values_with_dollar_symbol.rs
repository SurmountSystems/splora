//! Cypress title: displays USD fiat values with dollar symbol.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! lines 116-127. After USD and fiat mode, the transaction-details `.fiat`
//! cell and the first transactions-list `.fiat` cell, trimmed, each include
//! `$`. This does not assert a numeric amount. It does not fetch a price
//! and it does not fetch a transaction URL.
//! `paint_displays_usd_fiat_values_with_dollar_symbol` opens one headless
//! window with `show: false` and quits before it returns. The two strings
//! are the laid-out transaction-details cell, then the transactions-list cell.

const CYPRESS_IT: &str = "displays USD fiat values with dollar symbol";

#[test]
fn displays_usd_fiat_values_with_dollar_symbol() {
    let cells: [String; 2] = splora_native::paint_displays_usd_fiat_values_with_dollar_symbol();
    let details = cells[0].trim();
    let list = cells[1].trim();
    assert!(
        details.contains('$'),
        "{CYPRESS_IT}: transaction-details fiat cell includes `$`, got {details:?}"
    );
    assert!(
        list.contains('$'),
        "{CYPRESS_IT}: transactions-list fiat cell includes `$`, got {list:?}"
    );
}
