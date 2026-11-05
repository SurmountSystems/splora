//! Last kept mempool block over the weight limit titles a stack.
//!
//! The last kept preview index is 7. Virtual size 2000000 and 1000001
//! title a stack of two. Virtual size 1000000 titles Mempool block 8.
//! Index 0 titles Next Block.
//!
//! `paint_mempool_block_ordinal_title` is not defined on `splora_web`.
//! This test does not add it.

#[test]
fn last_kept_mempool_block_over_the_weight_limit_titles_a_stack() {
    let painted_over = splora_web::paint_mempool_block_ordinal_title(7, 2000000);
    let shared_over = splora_frontend_shared::mempool_block_ordinal_title(7, 2000000);
    assert_eq!(painted_over, shared_over);
    assert_eq!(shared_over, "Stack of 2 mempool blocks");

    let painted_at_limit = splora_web::paint_mempool_block_ordinal_title(7, 1000000);
    let shared_at_limit = splora_frontend_shared::mempool_block_ordinal_title(7, 1000000);
    assert_eq!(painted_at_limit, shared_at_limit);
    assert_eq!(shared_at_limit, "Mempool block 8");

    let painted_just_over = splora_web::paint_mempool_block_ordinal_title(7, 1000001);
    let shared_just_over = splora_frontend_shared::mempool_block_ordinal_title(7, 1000001);
    assert_eq!(painted_just_over, shared_just_over);
    assert_eq!(shared_just_over, "Stack of 2 mempool blocks");

    let painted_next = splora_web::paint_mempool_block_ordinal_title(0, 2000000);
    let shared_next = splora_frontend_shared::mempool_block_ordinal_title(0, 2000000);
    assert_eq!(painted_next, shared_next);
    assert_eq!(shared_next, "Next Block");
}
