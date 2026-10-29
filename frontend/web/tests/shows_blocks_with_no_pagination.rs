//! Cypress it() name: shows blocks with no pagination.
//!
//! Row 8. The spec visits
//! `/testnet4/block/000000000066e8b6cc78a93f8989587f5819624bae2eb1c05f535cadded19f99`
//! and expects the block-screen `h2` text to equal `18 transactions`, and the
//! first `ul.pagination` to have 5 children. `GET /block/:hash` supplies
//! `tx_count`. `GET /block/:hash/txs` is the transaction list for that block.
//! Eighteen transactions fit on one page of 25, so the pager is first,
//! previous, that one page, next, and last.
//! `shows_blocks_with_no_pagination_markup` is the paint function for that
//! screen. It is not defined on `splora_web`.

use splora_web::shows_blocks_with_no_pagination_markup;

const BLOCK_HASH: &str = "000000000066e8b6cc78a93f8989587f5819624bae2eb1c05f535cadded19f99";

const BLOCK_JSON: &str = r#"{"id":"000000000066e8b6cc78a93f8989587f5819624bae2eb1c05f535cadded19f99","height":0,"version":1,"timestamp":0,"tx_count":18,"size":285,"weight":1140,"merkle_root":"m","mediantime":0}"#;

fn h2_text(markup: &str) -> String {
    let start = markup
        .find("<h2")
        .unwrap_or_else(|| panic!("missing h2 in {markup}"));
    let rest = &markup[start..];
    let open_end = rest.find('>').expect("h2 tag is not closed");
    let inner = &rest[open_end + 1..];
    let end = inner.find("</h2>").expect("h2 is not closed");
    inner[..end].to_string()
}

fn pagination_children(markup: &str) -> usize {
    let start = markup
        .find("<ul class=\"pagination\">")
        .unwrap_or_else(|| panic!("missing pager in {markup}"));
    let rest = &markup[start..];
    let end = rest.find("</ul>").expect("pager is not closed");
    rest[..end].matches("<li").count()
}

#[test]
fn shows_blocks_with_no_pagination() {
    let markup = shows_blocks_with_no_pagination_markup(BLOCK_JSON);
    assert_eq!(
        h2_text(&markup),
        "18 transactions",
        "cy.get('h2').invoke('text').should('equal', '18 transactions'), markup was {markup}"
    );
    assert_eq!(
        pagination_children(&markup),
        5,
        "cy.get('ul.pagination').first().children().should('have.length', 5), markup was {markup}"
    );
    assert!(
        markup.contains(&format!("/block/{BLOCK_HASH}/txs")),
        "GET /block/:hash/txs missing from {markup}"
    );
}
