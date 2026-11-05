//! Cypress title `loads the dashboard`.
//! File: `ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts` line 15.
//!
//! A signet dashboard visit is redirected and does not open a signet indexer.
//! Splora `/signet` is HTTP 307 to `/mutinynet`.
//! `paint_signet_redirect` forwards `splora_api::signet_redirect`.
//! This test does not add that paint.

#[test]
fn signet_dashboard_visit_redirects_to_mutinynet() {
    let painted = splora_web::paint_signet_redirect();
    assert_eq!(painted, splora_api::signet_redirect());
    let (status, location) = painted;
    assert_eq!(status, 307);
    assert_eq!(location, "/mutinynet");
    assert!(
        !location.contains("signet"),
        "signet is not a backend; redirect location from signet_redirect was {location}"
    );
}
