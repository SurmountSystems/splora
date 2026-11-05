//! Cypress it() title: shows warning when entering 21M BTC.
//!
//! Spec: `ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts`
//! around line 168. Typed text `21000000` shows a warning containing
//! "Values were capped at the max supply of 21M BTC". The bitcoin text
//! equals `21000000`. The satoshi text equals `2100000000000000`.
//! Typed text `1` does not show that warning. Its satoshi text equals
//! `100000000`.
//! This test calls `splora_web::paint_shows_warning_when_entering_21m_btc`
//! and the shared function `shows_warning_when_entering_21m_btc`.
//! The paint is not defined on `splora_web`. This test does not add it.

#[test]
fn shows_warning_when_entering_21m_btc() {
    let cap = "21000000";
    let painted = splora_web::paint_shows_warning_when_entering_21m_btc(cap);
    let shared = splora_frontend_shared::shows_warning_when_entering_21m_btc(cap);
    assert!(
        painted
            .warning
            .contains("Values were capped at the max supply of 21M BTC"),
        "warning for {cap:?} must contain the 21M supply cap sentence, got {:?}",
        painted.warning
    );
    assert_eq!(
        painted.bitcoin, "21000000",
        "bitcoin text from {cap:?} must equal \"21000000\""
    );
    assert_eq!(
        painted.satoshis, "2100000000000000",
        "satoshi text from {cap:?} must equal \"2100000000000000\""
    );
    assert_eq!(
        painted.warning, shared.warning,
        "painted warning for {cap:?} must equal shared shows_warning_when_entering_21m_btc"
    );
    assert_eq!(
        painted.bitcoin, shared.bitcoin,
        "painted bitcoin text for {cap:?} must equal shared shows_warning_when_entering_21m_btc"
    );
    assert_eq!(
        painted.satoshis, shared.satoshis,
        "painted satoshi text for {cap:?} must equal shared shows_warning_when_entering_21m_btc"
    );

    let one = "1";
    let painted_one = splora_web::paint_shows_warning_when_entering_21m_btc(one);
    let shared_one = splora_frontend_shared::shows_warning_when_entering_21m_btc(one);
    assert!(
        !painted_one
            .warning
            .contains("Values were capped at the max supply of 21M BTC"),
        "typed text {one:?} must not show the 21M supply cap warning, got {:?}",
        painted_one.warning
    );
    assert_eq!(
        painted_one.satoshis, "100000000",
        "satoshi text from {one:?} must equal \"100000000\""
    );
    assert_eq!(
        painted_one.warning, shared_one.warning,
        "painted warning for {one:?} must equal shared shows_warning_when_entering_21m_btc"
    );
    assert_eq!(
        painted_one.bitcoin, shared_one.bitcoin,
        "painted bitcoin text for {one:?} must equal shared shows_warning_when_entering_21m_btc"
    );
    assert_eq!(
        painted_one.satoshis, shared_one.satoshis,
        "painted satoshi text for {one:?} must equal shared shows_warning_when_entering_21m_btc"
    );
}
