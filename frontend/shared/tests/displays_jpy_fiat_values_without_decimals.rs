//! Cypress it() title: displays JPY fiat values without decimals.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! lines 129-145. After JPY and fiat mode, the transaction-details `.fiat`
//! cell and the first transactions-list `.fiat` cell, trimmed, each include
//! `¥` (U+00A5) and do not match `/\.\d+/`. This does not assert digits, an
//! amount, or a price. It does not fetch a transaction URL.

#[test]
fn displays_jpy_fiat_values_without_decimals() {
    let cells: [String; 2] = splora_frontend_shared::displays_jpy_fiat_values_without_decimals();
    let details = cells[0].trim();
    let list = cells[1].trim();
    assert!(
        details.contains('\u{00A5}'),
        "transaction-details fiat cell includes `¥`, got {details:?}"
    );
    assert!(
        !text_matches_dot_digits(details),
        "transaction-details fiat cell must not match /\\.\\d+/, got {details:?}"
    );
    assert!(
        list.contains('\u{00A5}'),
        "transactions-list fiat cell includes `¥`, got {list:?}"
    );
    assert!(
        !text_matches_dot_digits(list),
        "transactions-list fiat cell must not match /\\.\\d+/, got {list:?}"
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
