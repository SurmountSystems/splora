//! Cypress it() title: correctly formats when switching from USD to JPY.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! lines 82-94. After USD, `.table-cell-fiat` index 1 includes `$`. After
//! JPY, that same index includes `¥` and must not match `/\.\d+$/`. This
//! does not assert a dollar amount, a yen amount, or a price. It does not
//! read a price feed.

#[test]
fn correctly_formats_when_switching_from_usd_to_jpy() {
    let cells: [Vec<String>; 2] =
        splora_frontend_shared::correctly_formats_when_switching_from_usd_to_jpy();
    let usd_cells = &cells[0];
    let jpy_cells = &cells[1];
    assert!(
        usd_cells.len() >= 2,
        "USD fiat cells include a code cell and an amount cell"
    );
    assert!(
        jpy_cells.len() >= 2,
        "JPY fiat cells include a code cell and an amount cell"
    );

    let usd_text = usd_cells[1].trim();
    assert!(
        usd_text.contains('$'),
        "USD index-1 fiat cell includes `$`, got {usd_text:?}"
    );

    let jpy_text = jpy_cells[1].trim();
    assert!(
        jpy_text.contains('\u{00A5}'),
        "JPY index-1 fiat cell includes `¥`, got {jpy_text:?}"
    );
    assert!(
        !ends_with_decimal_fraction(jpy_text),
        "JPY index-1 fiat cell must not match /\\.\\d+$/, got {jpy_text:?}"
    );
}

/// `/\.\d+$/`: the trimmed text ends with `.` and one or more ASCII digits.
fn ends_with_decimal_fraction(text: &str) -> bool {
    match text.rfind('.') {
        Some(index) => {
            let frac = &text[index + 1..];
            !frac.is_empty() && frac.bytes().all(|byte| byte.is_ascii_digit())
        }
        None => false,
    }
}
