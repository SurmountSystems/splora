//! Previous and next link flags for one confirmed block height.
//!
//! Each call passes a caller-supplied height into `block_prev_next_links`
//! and checks both returned booleans. This file does not recompute them.

#[test]
fn block_prev_next_links() {
    let latest_confirmed_height: u32 = 800_000;

    assert_eq!(
        splora_frontend_shared::block_prev_next_links(
            Some(latest_confirmed_height),
            latest_confirmed_height
        ),
        (true, false)
    );
    assert_eq!(
        splora_frontend_shared::block_prev_next_links(None, latest_confirmed_height),
        (false, true)
    );
    assert_eq!(
        splora_frontend_shared::block_prev_next_links(Some(0), latest_confirmed_height),
        (false, true)
    );
    assert_eq!(
        splora_frontend_shared::block_prev_next_links(Some(400_000), latest_confirmed_height),
        (true, true)
    );
}
