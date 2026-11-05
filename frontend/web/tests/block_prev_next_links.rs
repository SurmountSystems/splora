//! Previous and next link flags for one confirmed block height.
//!
//! The web paint returns the same pair as
//! `splora_frontend_shared::block_prev_next_links`. This file does not
//! recompute the flags.

#[test]
fn block_prev_next_links() {
    let latest_confirmed_height: u32 = 800_000;

    let tip: Option<u32> = Some(latest_confirmed_height);
    assert_eq!(
        splora_web::paint_block_prev_next_links(tip, latest_confirmed_height),
        splora_frontend_shared::block_prev_next_links(tip, latest_confirmed_height)
    );
    assert_eq!(
        splora_web::paint_block_prev_next_links(tip, latest_confirmed_height),
        (true, false)
    );

    let missing: Option<u32> = None;
    assert_eq!(
        splora_web::paint_block_prev_next_links(missing, latest_confirmed_height),
        splora_frontend_shared::block_prev_next_links(missing, latest_confirmed_height)
    );
    assert_eq!(
        splora_web::paint_block_prev_next_links(missing, latest_confirmed_height),
        (false, true)
    );

    let genesis: Option<u32> = Some(0);
    assert_eq!(
        splora_web::paint_block_prev_next_links(genesis, latest_confirmed_height),
        splora_frontend_shared::block_prev_next_links(genesis, latest_confirmed_height)
    );
    assert_eq!(
        splora_web::paint_block_prev_next_links(genesis, latest_confirmed_height),
        (false, true)
    );

    let middle: Option<u32> = Some(400_000);
    assert_eq!(
        splora_web::paint_block_prev_next_links(middle, latest_confirmed_height),
        splora_frontend_shared::block_prev_next_links(middle, latest_confirmed_height)
    );
    assert_eq!(
        splora_web::paint_block_prev_next_links(middle, latest_confirmed_height),
        (true, true)
    );
}
