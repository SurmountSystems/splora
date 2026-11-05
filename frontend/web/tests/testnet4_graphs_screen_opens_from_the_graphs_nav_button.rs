//! Cypress `it('loads the graphs screen')`.
//! File: `ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts`, lines 55-61.
//!
//! Visit `/testnet4`, wait until the skeleton is gone, then click `#btn-graphs`.
//! `/graphs` redirects to the mempool child (`graphs.routing.module.ts`, the
//! empty path `redirectTo: 'mempool'`). That child is the statistics view.
//!
//! Primary visible text, from
//! `ref/mempool/frontend/src/app/components/statistics/statistics.component.html`
//! line 7:
//! `Mempool by vBytes (sat/vByte)`
//!
//! The router mounts `GraphsPage` at `path!("graphs")`. That page's body is
//! `graphs_page_text`. This test calls that paint. It does not call
//! `after_graphs_button_loads_the_graphs_screen`.

/// Angular card title the graphs click opens. Not the Splora `Graphs` heading.
const ANGULAR_GRAPHS_TITLE: &str = "Mempool by vBytes (sat/vByte)";

#[test]
fn loads_the_graphs_screen() {
    let paint = splora_web::graphs_page_text();
    assert!(
        paint.contains(ANGULAR_GRAPHS_TITLE),
        "GraphsPage paint must include `{ANGULAR_GRAPHS_TITLE}` from statistics.component.html:7, paint was: {paint}"
    );
}
