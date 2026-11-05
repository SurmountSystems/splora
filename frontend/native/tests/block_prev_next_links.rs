//! Previous and next link flags for one confirmed block height.
//!
//! `paint_block_prev_next_links` returns the same pair as
//! `splora_frontend_shared::block_prev_next_links`. The pair is
//! `(previous_link_shown, next_link_shown)`. This test does not open a
//! window. `paint_block_prev_next_links` is not defined on `splora_native`.
//! This test does not add it.

#[test]
fn block_prev_next_links() {
    let latest_confirmed_height: u32 = 800_000;
    let cases = [
        (Some(800_000), (true, false)),
        (None, (false, true)),
        (Some(0), (false, true)),
        (Some(400_000), (true, true)),
    ];
    for (block_height, expected) in cases {
        let painted =
            splora_native::paint_block_prev_next_links(block_height, latest_confirmed_height);
        let shared =
            splora_frontend_shared::block_prev_next_links(block_height, latest_confirmed_height);
        assert_eq!(painted, shared);
        assert_eq!(painted, expected);
    }
}
