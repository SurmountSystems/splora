//! Row 6 native paint for the block heading.
//!
//! Cypress it "shows empty blocks properly" in
//! `ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts`
//! visits `/testnet4/block/0` and expects `h2` text `1 transaction`
//! when `GET /block/:hash` reports `tx_count` 1.
//! The native library does not export this paint yet.

use splora_frontend_shared::wire::Block;

fn block_with_one_transaction() -> Block {
    Block {
        id: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        height: 0,
        version: 1,
        timestamp: 0,
        tx_count: 1,
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
    let cypress_it = "shows empty blocks properly";
    let block = block_with_one_transaction();
    let painted: String =
        splora_native::paint_shows_empty_blocks_properly("/testnet4/block/0", &block);
    assert_eq!(
        painted, "1 transaction",
        "{cypress_it}: cy.get('h2').invoke('text').should('equal', '1 transaction')"
    );
}
