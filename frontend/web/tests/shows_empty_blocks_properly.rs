//! Cypress it() name: shows empty blocks properly.
//!
//! Row 6. The spec visits `/testnet4/block/0` and expects the block-screen
//! `h2` text to equal `1 transaction`. `GET /block/:hash` supplies
//! `tx_count`. `shows_empty_blocks_properly_markup` is the paint function
//! for that heading. It is not defined on `splora_web`.

use splora_web::shows_empty_blocks_properly_markup;

const BLOCK_JSON: &str = r#"{"id":"0000000000000000000000000000000000000000000000000000000000000000","height":0,"version":1,"timestamp":0,"tx_count":1,"size":285,"weight":1140,"merkle_root":"m","mediantime":0}"#;

#[test]
fn shows_empty_blocks_properly() {
    let markup = shows_empty_blocks_properly_markup(BLOCK_JSON);
    assert!(
        markup.contains("<h2>1 transaction</h2>"),
        "cy.get('h2').invoke('text').should('equal', '1 transaction'), markup was {markup}"
    );
}
