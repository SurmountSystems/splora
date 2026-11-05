//! Cypress it() title: formats all JPY values correctly without decimals.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts
//! lines 71-78. For each `.table-cell-fiat` cell, trim the text. When the
//! trimmed text includes `¥`, it must not match `/\.\d+/`. A cell without
//! the yen sign is not checked. This does not assert a dollar amount, a yen
//! amount, a satoshi count, or an indexer field. It does not read a price feed.

#[test]
fn formats_all_jpy_values_correctly_without_decimals() {
    let cells = splora_frontend_shared::formats_all_jpy_values_correctly_without_decimals();
    assert!(!cells.is_empty(), "fiat cells are non-empty");
    let mut saw_yen = false;
    for cell in &cells {
        let text = cell.trim();
        if !text.contains('\u{00A5}') {
            continue;
        }
        saw_yen = true;
        assert!(
            !text_matches_dot_digits(text),
            "yen fiat cell must not match /\\.\\d+/, got {text:?}"
        );
    }
    assert!(
        saw_yen,
        "at least one trimmed fiat cell includes the yen sign"
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
