use splora_frontend_shared::{
    MempoolBacklog, first_mempool_block_after_skeleton, first_mempool_block_from_backlog,
};

#[test]
fn check_first_mempool_block_after_skeleton_loads() {
    let block = first_mempool_block_from_backlog(MempoolBacklog {
        count: 0,
        vsize: 0,
        total_fee: 0,
        fee_histogram_len: 0,
    });
    assert_eq!(
        (block.skeleton_gone, block.block_link_exists),
        (true, true),
        "after cy.visit('/') and cy.waitForSkeletonGone(), #mempool-block-0 > .blockLink exists"
    );
    assert_eq!(
        block.link_id, "mempool-block-0",
        "a received backlog stores ngFor render index 0 as the block link id"
    );
    let unloaded = first_mempool_block_after_skeleton();
    assert_eq!(
        (unloaded.skeleton_gone, unloaded.block_link_exists),
        (false, false),
        "no backlog response means the skeleton is still up and index 0 has no block link"
    );
    assert_eq!(
        unloaded.link_id, "",
        "no backlog response leaves the block link id empty"
    );
}
