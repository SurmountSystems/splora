//! Cypress title: shows input labels for currency, BTC, and sats.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts line 66.
//! This test calls `paint_shows_input_labels_for_currency_btc_and_sats` and
//! compares that paint to the shared call. It does not open a window of its
//! own. Labels come from
//! `splora_frontend_shared::shows_input_labels_for_currency_btc_and_sats`.

const CYPRESS_IT: &str = "shows input labels for currency, BTC, and sats";

#[test]
fn shows_input_labels_for_currency_btc_and_sats() {
    let painted = splora_native::paint_shows_input_labels_for_currency_btc_and_sats();
    let shared = splora_frontend_shared::shows_input_labels_for_currency_btc_and_sats();
    assert_eq!(
        painted, shared,
        "{CYPRESS_IT}: painted input labels must match the shared call"
    );
}
