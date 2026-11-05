//! Row 135. An empty `#blinded=` fragment leaves confidential amounts in place.
//!
//! Cypress: `ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts`
//! `show empty unblinded TX`.
//! The visit is the same transaction as the populated unblind, with the
//! fragment empty (`#blinded=`). The first input row and both output rows
//! have class `ng-star-inserted`. The first input amount and both output
//! amounts contain Confidential. Fee is not part of that fragment.
//! This is not the populated unblinded amount case.
//! The native render function under test does not exist yet.

#[test]
fn show_empty_unblinded_tx() {
    let text = splora_native::show_empty_unblinded_tx_text("", 0);
    let vin_at = text.find("table-tx-vin").expect("vin table missing");
    let vout_at = text.find("table-tx-vout").expect("vout table missing");
    assert!(vin_at < vout_at, "{text}");
    let vin = &text[vin_at..vout_at];
    let vout = &text[vout_at..];
    assert!(
        vin.contains("ng-star-inserted"),
        "first input row missing ng-star-inserted: {text}"
    );
    assert!(
        vin.contains("class=\"amount\""),
        "first input amount cell missing: {text}"
    );
    assert!(
        vin.contains("Confidential"),
        "first input amount is not Confidential: {text}"
    );
    assert!(
        vout.matches("ng-star-inserted").count() >= 2,
        "both output rows need ng-star-inserted: {text}"
    );
    assert!(
        vout.matches("Confidential").count() >= 2,
        "both output amounts must stay Confidential: {text}"
    );
}
