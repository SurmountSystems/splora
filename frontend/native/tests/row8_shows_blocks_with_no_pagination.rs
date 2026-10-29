//! Row 8 native paint for blocks with no pagination.
//!
//! Cypress it "shows blocks with no pagination" in
//! `ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts`
//! visits
//! `/testnet4/block/000000000066e8b6cc78a93f8989587f5819624bae2eb1c05f535cadded19f99`
//! and expects `h2` text `18 transactions` and the first `ul.pagination`
//! to have 5 children. The transaction list is `GET /block/:hash/txs`.
//! `next-real-row.txt` says `cypress_it=shows blocks with no pagination`
//! and `endpoint=GET /block/:hash/txs`.
//! The native library does not export this paint yet.

use splora_frontend_shared::block_txs_path;
use splora_frontend_shared::wire::Block;

const TESTNET4_BLOCK: &str = "000000000066e8b6cc78a93f8989587f5819624bae2eb1c05f535cadded19f99";

fn block_with_eighteen_transactions() -> Block {
    Block {
        id: TESTNET4_BLOCK.to_string(),
        height: 0,
        version: 1,
        timestamp: 0,
        tx_count: 18,
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
fn shows_blocks_with_no_pagination() {
    let cypress_it = "shows blocks with no pagination";
    let block = block_with_eighteen_transactions();
    let path = format!("/testnet4/block/{TESTNET4_BLOCK}");
    let painted = splora_native::paint_shows_blocks_with_no_pagination(&path, &block);
    assert_eq!(
        painted.heading, "18 transactions",
        "{cypress_it}: cy.get('h2').invoke('text').should('equal', '18 transactions')"
    );
    assert_eq!(
        painted.first_pagination_children, 5,
        "{cypress_it}: cy.get('ul.pagination').first().children().should('have.length', 5)"
    );
    assert_eq!(
        painted.txs_path,
        block_txs_path(&block.id),
        "{cypress_it}: GET /block/:hash/txs"
    );
    assert_eq!(painted.txs_path, format!("/block/{TESTNET4_BLOCK}/txs"));
}
