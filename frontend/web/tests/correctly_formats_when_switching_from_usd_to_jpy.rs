//! Cypress `it('correctly formats when switching from USD to JPY')`.
//! File: `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! lines 82-94.
//!
//! After USD, the second `.table-cell-fiat` cell (index 1) includes `$`.
//! After JPY, that same index includes `¥` and must not match `/\.\d+$/`.
//! This does not assert a price number, does not hard-code a fixture amount,
//! and does not add `/v1/prices`.
//! `paint_correctly_formats_when_switching_from_usd_to_jpy` is not in this
//! crate. The call is the missing paint. The paint forwards the shared return.

#[test]
fn correctly_formats_when_switching_from_usd_to_jpy() {
    let painted: [Vec<String>; 2] =
        splora_web::paint_correctly_formats_when_switching_from_usd_to_jpy();
    let shared = splora_frontend_shared::correctly_formats_when_switching_from_usd_to_jpy();
    assert_eq!(
        painted, shared,
        "paint forwards splora_frontend_shared::correctly_formats_when_switching_from_usd_to_jpy"
    );
    assert!(
        painted[0].len() >= 2,
        "USD fiat cells include a code cell and an amount cell"
    );
    assert!(
        painted[1].len() >= 2,
        "JPY fiat cells include a code cell and an amount cell"
    );

    let usd_text = painted[0][1].trim();
    assert!(
        usd_text.contains('$'),
        "USD index-1 fiat cell includes `$`, got {usd_text:?}"
    );

    let jpy_text = painted[1][1].trim();
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
