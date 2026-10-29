//! Cypress `it('loads the graphs screen')`.
//! File: `ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts`, lines 55-61.
//!
//! Visit `/testnet4`, wait until the skeleton is gone, then click `#btn-graphs`.
//! Activating that control shows the graphs screen.
//! This test does not check hashrate, prices, or Lightning data.
//! `graphs_screen_markup` is not in this crate. The call is the missing paint.

#[test]
fn loads_the_graphs_screen() {
    let markup = splora_web::graphs_screen_markup();
    assert!(
        activating_the_graphs_control_shows_the_graphs_screen(&markup),
        "activating the graphs control must show the graphs screen: {markup}"
    );
}

/// `#btn-graphs` is the control Cypress clicks. The graphs screen is shown only
/// when that control is painted once and the rest of the paint names the
/// graphs screen. The control id alone is not the screen. `not-served` is not
/// the screen.
fn activating_the_graphs_control_shows_the_graphs_screen(markup: &str) -> bool {
    let control = r#"id="btn-graphs""#;
    if markup.matches(control).count() != 1 {
        return false;
    }
    let rest = markup.replace(control, "");
    if rest.trim().is_empty() || rest.contains("not-served") {
        return false;
    }
    rest.to_ascii_lowercase().contains("graph")
}
