//! Cypress title: loads the blocks page.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts line 38.
//! The it() visits the dashboard, clicks `#btn-blocks`, and waits until
//! `.skeleton-loader` length is 0.
//! The web paint must return `loads_the_blocks_page`.
//! The asserted fields come from that shared return, not from a second copy
//! of the route text.

#[test]
fn loads_the_blocks_page() {
    let painted = splora_web::paint_loads_the_blocks_page();
    let shared = splora_frontend_shared::loads_the_blocks_page();
    assert_eq!(painted.button_id, shared.button_id);
    assert_eq!(painted.route, shared.route);
    assert_eq!(painted.skeleton_loader_count, shared.skeleton_loader_count);
    assert_eq!(painted, shared);
}
