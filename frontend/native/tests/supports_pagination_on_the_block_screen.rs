//! Cypress it() title: supports pagination on the block screen.
//!
//! The spec visits the 48-transaction block
//! `000000000000006982d53f8273bdff21dafc380c292eabc669b5ab6d732311c3`,
//! scrolls to `.pagination`, and clicks `.active + li`. The first
//! `.header-bg.box > a` text must change. The later page is
//! `GET /block/:hash/txs/:start_index`.
//! `paint_supports_pagination_on_the_block_screen` is not defined on
//! `splora_native`.

const CYPRESS_IT: &str = "supports pagination on the block screen";
const BLOCK_HASH: &str = "000000000000006982d53f8273bdff21dafc380c292eabc669b5ab6d732311c3";

fn first_header_link(markup: &str) -> String {
    let marker = "class=\"header-bg box\"";
    let start = markup
        .find(marker)
        .unwrap_or_else(|| panic!("missing header-bg box in {markup}"));
    let rest = &markup[start..];
    let anchor = rest
        .find("<a")
        .unwrap_or_else(|| panic!("missing anchor in {markup}"));
    let after = &rest[anchor..];
    let open_end = after.find('>').expect("anchor tag is not closed");
    let inner = &after[open_end + 1..];
    let end = inner.find("</a>").expect("anchor is not closed");
    inner[..end].trim().to_string()
}

#[test]
fn supports_pagination_on_the_block_screen() {
    let first_page = splora_native::paint_supports_pagination_on_the_block_screen(BLOCK_HASH, 0);
    let later_page = splora_native::paint_supports_pagination_on_the_block_screen(BLOCK_HASH, 25);
    assert_ne!(
        first_header_link(&first_page),
        first_header_link(&later_page),
        "{CYPRESS_IT}: cy.get('.active + li').first().click() changes cy.get('.header-bg.box > a')"
    );
    assert!(
        later_page.contains(&format!("/block/{BLOCK_HASH}/txs/25")),
        "{CYPRESS_IT}: GET /block/:hash/txs/:start_index missing from {later_page}"
    );
}
