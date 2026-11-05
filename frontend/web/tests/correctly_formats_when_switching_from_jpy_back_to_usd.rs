//! Cypress `it('correctly formats when switching from JPY back to USD')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! lines 96-106.
//!
//! After JPY, the second `.table-cell-fiat` cell (index 1) includes `¥`.
//! After USD, that same index, trimmed, includes `$`.
//! This does not assert a price number, does not hard-code a fixture amount,
//! and does not add `/v1/prices`.
//! `paint_correctly_formats_when_switching_from_jpy_back_to_usd` is not in this
//! crate. The call is the missing paint. The paint forwards the shared return.

#[test]
fn correctly_formats_when_switching_from_jpy_back_to_usd() {
    let painted: [Vec<String>; 2] =
        splora_web::paint_correctly_formats_when_switching_from_jpy_back_to_usd();
    let shared = splora_frontend_shared::correctly_formats_when_switching_from_jpy_back_to_usd();
    assert_eq!(
        painted, shared,
        "paint forwards splora_frontend_shared::correctly_formats_when_switching_from_jpy_back_to_usd"
    );
    assert!(
        painted[0].len() >= 2,
        "JPY fiat cells include a code cell and an amount cell"
    );
    assert!(
        painted[1].len() >= 2,
        "USD fiat cells include a code cell and an amount cell"
    );

    let jpy_text = painted[0][1].trim();
    assert!(
        jpy_text.contains('\u{00A5}'),
        "JPY index-1 fiat cell includes `¥`, got {jpy_text:?}"
    );

    let usd_text = painted[1][1].trim();
    assert!(
        usd_text.contains('$'),
        "USD index-1 fiat cell includes `$`, got {usd_text:?}"
    );
}
