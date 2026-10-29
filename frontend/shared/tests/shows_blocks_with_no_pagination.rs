//! Cypress `shows blocks with no pagination` checks the block-screen heading
//! and the first pager.
//!
//! The spec visits
//! `/testnet4/block/000000000066e8b6cc78a93f8989587f5819624bae2eb1c05f535cadded19f99`,
//! then expects `h2` text `18 transactions` and the first `ul.pagination` to
//! have 5 children. The heading is `tx_count` from `GET /block/:hash`. The
//! transaction list is `GET /block/:hash/txs`. Eighteen transactions fit on
//! one page when the page size is 25, so the pager is first, previous, that
//! one page, next, and last.

use splora_frontend_shared::block_txs_path;
use splora_frontend_shared::no_pagination_block_screen;
use splora_frontend_shared::transaction_count_heading;
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
    let block = block_with_eighteen_transactions();
    let screen = no_pagination_block_screen(block.tx_count);
    assert_eq!(
        transaction_count_heading(block.tx_count),
        "18 transactions",
        "cy.get('h2').invoke('text').should('equal', '18 transactions')"
    );
    assert_eq!(
        screen.heading, "18 transactions",
        "cy.get('h2').invoke('text').should('equal', '18 transactions')"
    );
    assert_eq!(
        screen.first_pagination_children, 5,
        "cy.get('ul.pagination').first().children().should('have.length', 5)"
    );
    assert_eq!(
        block_txs_path(&block.id),
        format!("/block/{TESTNET4_BLOCK}/txs")
    );
}
