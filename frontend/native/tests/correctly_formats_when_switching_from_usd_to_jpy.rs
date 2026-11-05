//! Cypress it() title: correctly formats when switching from USD to JPY.
//!
//! Mainnet spec
//! `ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts`
//! lines 82-94. After USD, the second `.table-cell-fiat` cell includes `$`.
//! After JPY, that same cell includes `¥` and must not match `/\.\d+$/`.
//! This test does not assert a price. It does not call the shared function.
//! The cell strings come from
//! `paint_correctly_formats_when_switching_from_usd_to_jpy`.
//! That paint is not defined on `splora_native`.

const CYPRESS_IT: &str = "correctly formats when switching from USD to JPY";

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

#[test]
fn correctly_formats_when_switching_from_usd_to_jpy() {
    let cells: [Vec<String>; 2] =
        splora_native::paint_correctly_formats_when_switching_from_usd_to_jpy();
    assert!(
        cells[0].len() >= 2,
        "{CYPRESS_IT}: USD fiat cells include a second cell"
    );
    assert!(
        cells[1].len() >= 2,
        "{CYPRESS_IT}: JPY fiat cells include a second cell"
    );

    let usd_text = cells[0][1].trim();
    assert!(
        usd_text.contains('$'),
        "{CYPRESS_IT}: USD second fiat cell must include '$', got {usd_text:?}"
    );

    let jpy_text = cells[1][1].trim();
    assert!(
        jpy_text.contains('¥'),
        "{CYPRESS_IT}: JPY second fiat cell must include '¥', got {jpy_text:?}"
    );
    assert!(
        !ends_with_decimal_fraction(jpy_text),
        "{CYPRESS_IT}: JPY second fiat cell must not match /\\.\\d+$/, got {jpy_text:?}"
    );
}
