//! Cypress it() title: correctly formats when switching from JPY back to USD.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! lines 96-106. After JPY, `.table-cell-fiat` index 1 includes `¥`. After
//! USD, that same index, trimmed, includes `$`. This does not assert a yen
//! amount, a dollar amount, or a price. It does not read a price feed.

#[test]
fn correctly_formats_when_switching_from_jpy_back_to_usd() {
    let cells: [Vec<String>; 2] =
        splora_frontend_shared::correctly_formats_when_switching_from_jpy_back_to_usd();
    let jpy_cells = &cells[0];
    let usd_cells = &cells[1];
    assert!(!jpy_cells.is_empty(), "JPY fiat cells are a non-empty list");
    assert!(
        jpy_cells.len() >= 2,
        "JPY fiat cells include a code cell and an amount cell"
    );
    assert!(!usd_cells.is_empty(), "USD fiat cells are a non-empty list");
    assert!(
        usd_cells.len() >= 2,
        "USD fiat cells include a code cell and an amount cell"
    );

    let jpy_text = jpy_cells[1].trim();
    assert!(
        jpy_text.contains('\u{00A5}'),
        "JPY index-1 fiat cell includes `¥`, got {jpy_text:?}"
    );

    let usd_text = usd_cells[1].trim();
    assert!(
        usd_text.contains('$'),
        "USD index-1 fiat cell includes `$`, got {usd_text:?}"
    );
}
