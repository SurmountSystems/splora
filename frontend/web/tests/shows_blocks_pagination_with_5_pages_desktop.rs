//! Cypress it() title: shows blocks pagination with 5 pages (desktop).
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts
//! Viewport 760 by 800. The first `.pagination-container ul.pagination`
//! has 9 children when the block has at least 125 transactions.
//! The next it(), "shows blocks pagination with 3 pages (mobile)", uses
//! viewport 669 by 800 and expects 7 children. A width wider than 670
//! uses the desktop count. This test calls the shipped block page paint.
//! It does not reimplement the pager.

const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn block_json(tx_count: u32) -> String {
    format!(
        r#"{{"id":"{HASH}","height":2,"version":536870912,"timestamp":1700000000,"tx_count":{tx_count},"size":100,"weight":400,"merkle_root":"merkle-root","previousblockhash":null,"mediantime":1699990000,"nonce":9,"bits":386547056,"difficulty":1}}"#
    )
}

fn pagination_children(markup: &str) -> usize {
    let start = markup
        .find("<ul class=\"pagination\">")
        .unwrap_or_else(|| panic!("missing pager in {markup}"));
    let rest = &markup[start..];
    let end = rest
        .find("</ul>")
        .unwrap_or_else(|| panic!("pager is not closed in {markup}"));
    rest[..end].matches("<li").count()
}

#[test]
fn shows_blocks_pagination_with_5_pages_desktop() {
    let txs = "[]";
    let desktop = splora_web::block_transactions_markup_for_viewport(&block_json(125), txs, 1, 760);
    assert!(
        desktop.contains("class=\"pagination-container\""),
        "the block page did not paint the pagination list: {desktop}"
    );
    assert_eq!(
        pagination_children(&desktop),
        9,
        "cy.get('.pagination-container ul.pagination').first().children().should('have.length', 9): {desktop}"
    );

    let phone = splora_web::block_transactions_markup_for_viewport(&block_json(125), txs, 1, 669);
    assert_eq!(
        pagination_children(&phone),
        7,
        "cy.get('.pagination-container ul.pagination').first().children().should('have.length', 7): {phone}"
    );

    let wider = splora_web::block_transactions_markup_for_viewport(&block_json(125), txs, 1, 671);
    assert_eq!(pagination_children(&wider), 9, "{wider}");

    let at_edge = splora_web::block_transactions_markup_for_viewport(&block_json(125), txs, 1, 670);
    assert_eq!(pagination_children(&at_edge), 7, "{at_edge}");

    let six_pages =
        splora_web::block_transactions_markup_for_viewport(&block_json(126), txs, 1, 760);
    assert_eq!(pagination_children(&six_pages), 9, "{six_pages}");
    let six_pages_phone =
        splora_web::block_transactions_markup_for_viewport(&block_json(126), txs, 1, 669);
    assert_eq!(
        pagination_children(&six_pages_phone),
        7,
        "{six_pages_phone}"
    );
}
