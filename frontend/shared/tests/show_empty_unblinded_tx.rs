//! Row 135. An empty blinded fragment leaves confidential amounts in place.
//!
//! Cypress `ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts`
//! `show empty unblinded TX` visits `#blinded=` and expects the first
//! input amount and both output amounts to contain Confidential.

#[test]
fn show_empty_unblinded_tx() {
    // `#blinded=` is an empty fragment. Fee is not part of that fragment.
    let preview = splora_frontend_shared::unblinded_tx_preview("", 0);
    assert_eq!(
        preview.matches("Confidential").count(),
        3,
        "empty blinded fragment must show Confidential on the first input and both outputs, got {preview:?}"
    );
}

/// An empty fragment is not three parsed amount lines, and the first vout
/// uses the same row class as the second.
#[test]
fn empty_fragment_is_not_an_asset_box() {
    let preview = splora_frontend_shared::unblinded_tx_preview("", 0);
    assert_eq!(
        preview.matches("Confidential").count(),
        3,
        "empty blinded fragment must show Confidential on the first input and both outputs, got {preview:?}"
    );
    let lines: Vec<&str> = preview.lines().filter(|line| !line.is_empty()).collect();
    assert_ne!(
        lines.len(),
        3,
        "unblinded_tx_preview(\"\") is not three parsed amount lines, got {preview:?}"
    );
    assert!(
        lines.iter().all(|line| !line.contains("LBTC")),
        "unblinded_tx_preview(\"\") must not be parsed amount lines, got {preview:?}"
    );

    let first = splora_frontend_shared::show_first_unblinded_vout("");
    let second = splora_frontend_shared::show_second_unblinded_vout("");
    assert_eq!(
        first.row_class, second.row_class,
        "show_first_unblinded_vout(\"\") must use the same row class as show_second_unblinded_vout(\"\"), first {} second {}",
        first.row_class, second.row_class
    );
    assert_eq!(
        first.row_class, "ng-star-inserted",
        "empty fragment row class must be ng-star-inserted, got {}",
        first.row_class
    );
}
