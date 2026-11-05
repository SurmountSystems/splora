//! First `ul.pagination` child count on a block transaction list.
//!
//! Cypress sets the viewport before the visit. The first paint reads
//! `matchMedia('(max-width: 670px)')`: width 670 and below shows 3 page
//! numbers, and a wider viewport shows 5. The Angular resize handler uses
//! `innerWidth < 670`, which disagrees only at width 670. This function
//! is the first-paint rule.
//!
//! Page size is 25, the Cypress `ITEMS_PER_PAGE`. The child count is
//! first, previous, the visible page numbers, next, and last. Visible
//! page numbers are the smaller of the page count and that max size.
//! Ellipses are off, so they add no items.

/// Transactions on one block page. Cypress sets `ITEMS_PER_PAGE` to 25.
const BLOCK_TXS_PER_PAGE: u32 = 25;

/// Pager window and the first `ul.pagination` child count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPagination {
    pub max_size: u32,
    pub page_count: u32,
    pub first_pagination_children: usize,
}

/// Child count of the first block pager for this viewport and `tx_count`.
///
/// `viewport_width` is the CSS viewport width in pixels. `tx_count` is
/// `block.tx_count`. The page size is 25. A width of 670 or less uses a
/// max of 3 page numbers. A wider viewport uses 5.
pub fn shows_blocks_pagination_with_5_pages_desktop(
    viewport_width: u32,
    tx_count: u32,
) -> BlockPagination {
    let max_size = if viewport_width <= 670 { 3 } else { 5 };
    let page_count = tx_count.div_ceil(BLOCK_TXS_PER_PAGE);
    let visible_pages = page_count.min(max_size);
    BlockPagination {
        max_size,
        page_count,
        first_pagination_children: 4 + visible_pages as usize,
    }
}
