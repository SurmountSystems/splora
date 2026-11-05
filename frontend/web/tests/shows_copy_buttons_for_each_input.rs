//! Cypress title: shows copy buttons for each input.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts line 52.
//! The web paint must return `shows_copy_buttons_for_each_input`.
//! The comparison is that paint call against the shared function, not a
//! copied clipboard count.

#[test]
fn shows_copy_buttons_for_each_input() {
    assert_eq!(
        splora_web::paint_shows_copy_buttons_for_each_input(),
        splora_frontend_shared::shows_copy_buttons_for_each_input()
    );
}
