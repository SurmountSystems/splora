//! Cypress title: displays JPY fiat values without decimals.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! lines 129-145. After JPY and fiat mode, the transaction-details `.fiat`
//! cell and the first transactions-list `.fiat` cell, trimmed, each include
//! `¥` (U+00A5) and do not match `/\.\d+/`. This does not assert a price,
//! does not assert amount digits, and does not fetch a transaction URL.
//! It does not add `/v1/prices`.
//! `paint_displays_jpy_fiat_values_without_decimals` opens one headless
//! window with `show: false` and quits before it returns. The two strings
//! are the laid-out transaction-details cell, then the transactions-list cell.

const CYPRESS_IT: &str = "displays JPY fiat values without decimals";

#[test]
fn displays_jpy_fiat_values_without_decimals() {
    let cells: [String; 2] = splora_native::paint_displays_jpy_fiat_values_without_decimals();
    let details = cells[0].trim();
    let list = cells[1].trim();
    assert!(
        details.contains('\u{00A5}'),
        "{CYPRESS_IT}: transaction-details fiat cell includes `¥`, got {details:?}"
    );
    assert!(
        !text_matches_dot_digits(details),
        "{CYPRESS_IT}: transaction-details fiat cell must not match /\\.\\d+/, got {details:?}"
    );
    assert!(
        list.contains('\u{00A5}'),
        "{CYPRESS_IT}: transactions-list fiat cell includes `¥`, got {list:?}"
    );
    assert!(
        !text_matches_dot_digits(list),
        "{CYPRESS_IT}: transactions-list fiat cell must not match /\\.\\d+/, got {list:?}"
    );
}

/// `/\.\d+/`: a `.` followed by one or more ASCII digits, anywhere in `text`.
fn text_matches_dot_digits(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.iter().enumerate().any(|(index, byte)| {
        *byte == b'.'
            && bytes
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_digit())
    })
}
