//! Last kept mempool block over the weight limit titles a stack.
//!
//! The native paint `paint_mempool_block_ordinal_title` must equal
//! `splora_frontend_shared::mempool_block_ordinal_title` on every row.
//! The shared text must also equal the string written on that row.
//! Those strings live in this file. This file does not implement the paint.

#[test]
fn last_kept_mempool_block_over_the_weight_limit_titles_a_stack() {
    // index, block virtual size, shared text that lives only in this file
    let rows: &[(u64, u64, &str)] = &[
        (7, 2_000_000, "Stack of 2 mempool blocks"),
        (7, 1_000_000, "Mempool block 8"),
        (7, 1_000_001, "Stack of 2 mempool blocks"),
        (0, 2_000_000, "Next Block"),
    ];
    for &(index, block_vsize, expected) in rows {
        let shared = splora_frontend_shared::mempool_block_ordinal_title(index, block_vsize);
        assert_eq!(
            shared, expected,
            "shared title for index {index} virtual size {block_vsize}"
        );
        let painted = splora_native::paint_mempool_block_ordinal_title(index, block_vsize);
        assert_eq!(
            painted, shared,
            "native paint must equal the shared title for index {index} virtual size {block_vsize}"
        );
    }
}
