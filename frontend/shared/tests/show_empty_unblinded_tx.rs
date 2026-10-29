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
