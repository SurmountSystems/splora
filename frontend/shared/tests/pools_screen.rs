use splora_frontend_shared::pools_screen_after_pools_control;

#[test]
fn loads_the_pools_screen() {
    let view = pools_screen_after_pools_control();
    assert!(
        view.pools_screen_active,
        "after #btn-pools is clicked, the pools screen is the active view"
    );
}
