//! Cypress `it('loads the pools screen')`.
//! File: `ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts`, lines 47-53.
//!
//! Visit `/testnet4`, wait until the skeleton is gone, then click `#btn-pools`.
//! Activating that control shows the pools screen.
//! This test does not check hashrate, prices, or Lightning data.

#[test]
fn loads_the_pools_screen() {
    let markup = splora_web::pools_screen_markup();
    assert!(
        activating_the_pools_control_shows_the_pools_screen(&markup),
        "activating the pools control must show the pools screen: {markup}"
    );
}

/// `#btn-pools` is the control Cypress clicks. The pools screen is shown only
/// when that control is painted once and the rest of the paint names the
/// pools screen. The control id alone is not the screen. `not-served` is not
/// the screen.
fn activating_the_pools_control_shows_the_pools_screen(markup: &str) -> bool {
    let control = r#"id="btn-pools""#;
    if markup.matches(control).count() != 1 {
        return false;
    }
    let rest = markup.replace(control, "");
    if rest.trim().is_empty() || rest.contains("not-served") {
        return false;
    }
    rest.to_ascii_lowercase().contains("pool")
}
