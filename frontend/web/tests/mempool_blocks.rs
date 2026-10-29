//! First mempool block after the dashboard skeleton has loaded.
//!
//! Cypress: it "check first mempool block after skeleton loads".
//! After the skeleton is gone, `#mempool-block-0 > .blockLink` is present.

#[test]
fn check_first_mempool_block_after_skeleton_loads() {
    let block = splora_frontend_shared::first_mempool_block_from_backlog(
        splora_frontend_shared::MempoolBacklog {
            count: 0,
            vsize: 0,
            total_fee: 0,
            fee_histogram_len: 0,
        },
    );
    assert_eq!(
        (block.skeleton_gone, block.block_link_exists),
        (true, true),
        "after cy.visit('/') and cy.waitForSkeletonGone(), #mempool-block-0 > .blockLink exists"
    );
}

#[test]
fn first_mempool_block_markup_contains_shared_link_id_and_block_link() {
    let backlog = splora_frontend_shared::MempoolBacklog {
        count: 0,
        vsize: 0,
        total_fee: 0,
        fee_histogram_len: 0,
    };
    let view = splora_frontend_shared::first_mempool_block_from_backlog(backlog);
    assert!(
        view.block_link_exists,
        "a received backlog keeps the block link"
    );
    assert!(
        !view.link_id.is_empty(),
        "a received backlog stores a link id"
    );
    assert_eq!(view.link_id, "mempool-block-0");
    let body = r#"{"count":0,"vsize":0,"total_fee":0,"fee_histogram":[]}"#;
    let markup = splora_web::first_mempool_block_link_markup(body);
    assert!(
        markup.contains(&view.link_id),
        "markup omitted the shared link_id: {markup}"
    );
    assert!(
        markup.contains("blockLink"),
        "markup omitted class blockLink: {markup}"
    );
    let product = include_str!("../src/views.rs");
    assert!(
        !product.contains("mempool-block-0"),
        "views.rs typed the cypress id instead of the shared link_id field"
    );
    assert!(
        !product.contains("FIRST_MEMPOOL_BLOCK_LINK_ID"),
        "views.rs copied a link id constant"
    );
    let absent = splora_web::first_mempool_block_link_markup("");
    assert!(
        !absent.contains(&view.link_id),
        "a body that does not parse emitted the shared link_id: {absent}"
    );
    assert!(
        !absent.contains("blockLink"),
        "a body that does not parse emitted class blockLink: {absent}"
    );
}
