//! Cypress it() title: shows blocks pagination with 5 pages (desktop).
//!
//! The spec sets the viewport to 760 by 800, visits one block, and expects
//! the first `ul.pagination` to have 9 children. The comment in the spec
//! says that is 5 page numbers plus 4 buttons. Width 760 is wider than
//! 670, so the first paint uses maxSize 5. The next it() in the same file
//! sets the viewport to 669 by 800 and expects 7 children. Width 669
//! matches `(max-width: 670px)`, so maxSize is 3. Page size is 25.
//! Visible page numbers are the smaller of the page count and maxSize.
//! Ellipses are off. This it() does not read a transaction JSON body.

use splora_frontend_shared::shows_blocks_pagination_with_5_pages_desktop as block_pager;

/// Five pages of 25. The desktop Cypress block shows 5 page buttons.
const FIVE_PAGES: u32 = 125;

#[test]
fn shows_blocks_pagination_with_5_pages_desktop() {
    let desktop = block_pager(760, FIVE_PAGES);
    assert_eq!(desktop.max_size, 5);
    assert_eq!(desktop.page_count, 5);
    assert_eq!(
        desktop.first_pagination_children, 9,
        "cy.get('.pagination-container ul.pagination').first().children().should('have.length', 9)"
    );

    let phone = block_pager(669, FIVE_PAGES);
    assert_eq!(phone.max_size, 3);
    assert_eq!(phone.page_count, 5);
    assert_eq!(
        phone.first_pagination_children, 7,
        "cy.get('.pagination-container ul.pagination').first().children().should('have.length', 7)"
    );

    // `(max-width: 670px)` still matches at 670. A later resize handler
    // uses innerWidth < 670 and would report 5 here. Cypress sets the
    // viewport before the visit, so the first paint is the media query.
    let at_media_edge = block_pager(670, FIVE_PAGES);
    assert_eq!(at_media_edge.max_size, 3);
    assert_eq!(at_media_edge.first_pagination_children, 7);

    let just_wider = block_pager(671, FIVE_PAGES);
    assert_eq!(just_wider.max_size, 5);
    assert_eq!(just_wider.first_pagination_children, 9);

    // Six pages still show only maxSize page numbers.
    let six_pages = block_pager(760, 126);
    assert_eq!(six_pages.page_count, 6);
    assert_eq!(six_pages.first_pagination_children, 9);
    let six_pages_phone = block_pager(669, 126);
    assert_eq!(six_pages_phone.first_pagination_children, 7);

    // Four pages on a wide viewport are not padded out to 5.
    let four_pages = block_pager(760, 100);
    assert_eq!(four_pages.page_count, 4);
    assert_eq!(four_pages.first_pagination_children, 8);

    // One page is first, previous, that page, next, and last.
    let one_page = block_pager(760, 18);
    assert_eq!(one_page.page_count, 1);
    assert_eq!(one_page.first_pagination_children, 5);

    // Two pages on a phone are not padded out to 3.
    let two_pages_phone = block_pager(669, 50);
    assert_eq!(two_pages_phone.page_count, 2);
    assert_eq!(two_pages_phone.max_size, 3);
    assert_eq!(two_pages_phone.first_pagination_children, 6);
}
