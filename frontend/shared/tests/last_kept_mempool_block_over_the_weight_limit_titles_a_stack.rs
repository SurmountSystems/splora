//! Last kept mempool block title, from mempool `getOrdinal`.
//!
//! The test calls `mempool_block_ordinal_title`. It does not compute the
//! stack count and it does not reimplement ceil.

use splora_frontend_shared::mempool_block_ordinal_title;

#[test]
fn last_kept_mempool_block_over_the_weight_limit_titles_a_stack() {
    assert_eq!(
        mempool_block_ordinal_title(7, 2_000_000),
        "Stack of 2 mempool blocks",
        "index 7 at virtual size 2000000 is a stack of two blocks"
    );
    assert_eq!(
        mempool_block_ordinal_title(7, 1_000_000),
        "Mempool block 8",
        "index 7 at virtual size 1000000 is one block, titled Mempool block 8"
    );
    assert_eq!(
        mempool_block_ordinal_title(7, 1_000_001),
        "Stack of 2 mempool blocks",
        "index 7 at virtual size 1000001 ceils to a stack of two, not truncation to one"
    );
    assert_eq!(
        mempool_block_ordinal_title(0, 2_000_000),
        "Next Block",
        "index 0 is Next Block even at virtual size 2000000"
    );
}
