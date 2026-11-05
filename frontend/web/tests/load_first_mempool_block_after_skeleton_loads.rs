//! Cypress title: load first mempool block after skeleton loads.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts.
//! The it() visits the dashboard, waits until the skeleton is gone, clicks
//! `#mempool-block-0 > .blockLink`, and waits until the skeleton is gone again.
//! The web paint must return `load_first_mempool_block_after_skeleton_loads`.
//! The asserted fields come from that shared return, not from a second copy
//! of the route text.

#[test]
fn load_first_mempool_block_after_skeleton_loads() {
    let painted = splora_web::paint_load_first_mempool_block_after_skeleton_loads();
    let shared = splora_frontend_shared::load_first_mempool_block_after_skeleton_loads();
    assert_eq!(painted.link_id, shared.link_id);
    assert_eq!(painted.block_link_class, shared.block_link_class);
    assert_eq!(painted.route, shared.route);
    assert_eq!(painted.skeleton_loader_count, shared.skeleton_loader_count);
    assert_eq!(painted, shared);
    assert_eq!(shared.link_id, format!("mempool-block-{}", 0));
    assert_eq!(shared.block_link_class, "blockLink");
    assert_eq!(shared.route, format!("/mempool-block/{}", 0));
    assert_eq!(shared.skeleton_loader_count, 0);
}
