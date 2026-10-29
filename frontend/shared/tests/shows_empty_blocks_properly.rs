//! Cypress `shows empty blocks properly` checks the block-screen heading.
//!
//! `GET /block/:hash` (`BlockValue` in `src/rest.rs`) supplies `tx_count`.
//! The spec visits `/testnet4/block/0` and expects `h2` text `1 transaction`
//! when that count is 1.

use splora_frontend_shared::empty_block_heading_from_tx_count;
use splora_frontend_shared::wire::Block;

fn block_with_tx_count(tx_count: u32) -> Block {
    Block {
        id: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        height: 0,
        version: 1,
        timestamp: 0,
        tx_count,
        size: 285,
        weight: 1140,
        merkle_root: "m".to_string(),
        previous_block_hash: None,
        median_time: 0,
        nonce: Some(0),
        bits: Some(0),
        difficulty: Some(1.0),
        ext: None,
        stale: None,
    }
}

#[test]
fn shows_empty_blocks_properly() {
    let block = block_with_tx_count(1);
    let heading = empty_block_heading_from_tx_count(block.tx_count);
    assert_eq!(
        heading, "1 transaction",
        "cy.get('h2').invoke('text').should('equal', '1 transaction')"
    );
}
