//! Cypress it() title: correctly formats when switching from JPY back to USD.
//!
//! Mainnet spec
//! `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! lines 96-106. After JPY, the second `.table-cell-fiat` cell includes `¥`.
//! After USD, that same cell, trimmed, includes `$`.
//! This test does not assert a price. It does not call the shared function.
//! The cell strings come from
//! `paint_correctly_formats_when_switching_from_jpy_back_to_usd`.

const CYPRESS_IT: &str = "correctly formats when switching from JPY back to USD";

#[test]
fn correctly_formats_when_switching_from_jpy_back_to_usd() {
    let cells: [Vec<String>; 2] =
        splora_native::paint_correctly_formats_when_switching_from_jpy_back_to_usd();
    assert!(
        cells[0].len() >= 2,
        "{CYPRESS_IT}: JPY fiat cells include a second cell"
    );
    assert!(
        cells[1].len() >= 2,
        "{CYPRESS_IT}: USD fiat cells include a second cell"
    );

    let jpy_text = cells[0][1].trim();
    assert!(
        jpy_text.contains('¥'),
        "{CYPRESS_IT}: JPY second fiat cell must include '¥', got {jpy_text:?}"
    );

    let usd_text = cells[1][1].trim();
    assert!(
        usd_text.contains('$'),
        "{CYPRESS_IT}: USD second fiat cell must include '$', got {usd_text:?}"
    );
}
