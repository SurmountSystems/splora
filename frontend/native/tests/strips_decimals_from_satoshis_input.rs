//! Cypress it() title: strips decimals from satoshis input.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts
//! The satoshis field is cleared, then the typed text `10000.99` is entered.
//! The field value must not include '.'. A trailing dot `10000.` is `10000`.
//! This is local field formatting. It does not fetch a rate.
//! `paint_strips_decimals_from_satoshis_input` is not defined on
//! `splora_native`. This test does not add it.

#[test]
fn strips_decimals_from_satoshis_input() {
    let typed = "10000.99";
    let painted: String = splora_native::paint_strips_decimals_from_satoshis_input(typed);
    assert_eq!(
        painted, "10001",
        "satoshis field from {typed:?} must equal \"10001\""
    );
    assert_eq!(
        painted,
        splora_frontend_shared::strips_decimals_from_satoshis_input(typed),
        "painted satoshis field from {typed:?} must equal shared strips_decimals_from_satoshis_input"
    );
    assert!(
        !painted.contains('.'),
        "satoshis field {painted:?} from {typed:?} must not contain '.'"
    );

    let trailing = "10000.";
    let trailing_painted: String =
        splora_native::paint_strips_decimals_from_satoshis_input(trailing);
    assert_eq!(
        trailing_painted, "10000",
        "satoshis field from {trailing:?} must equal \"10000\""
    );
    assert_eq!(
        trailing_painted,
        splora_frontend_shared::strips_decimals_from_satoshis_input(trailing),
        "painted satoshis field from {trailing:?} must equal shared strips_decimals_from_satoshis_input"
    );
    assert!(
        !trailing_painted.contains('.'),
        "satoshis field {trailing_painted:?} from {trailing:?} must not contain '.'"
    );
}
