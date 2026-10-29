use splora_frontend_shared::after_graphs_button_loads_the_graphs_screen;

#[test]
fn loads_the_graphs_screen() {
    let view = after_graphs_button_loads_the_graphs_screen();
    assert!(
        view.graphs_screen_active,
        "after cy.visit('/testnet4'), cy.waitForSkeletonGone(), and #btn-graphs is clicked, the graphs screen is the active view"
    );
}
