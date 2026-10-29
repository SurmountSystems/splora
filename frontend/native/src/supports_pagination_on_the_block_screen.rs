//! Painted block transaction page after the next pagination control.
//! The shared view decides whether the first header differs. This module
//! does not invent a transaction count, header, or transaction id list.

/// Paint for `supports pagination on the block screen`.
///
/// `block_hash` and `start_index` are the arguments the test passes.
/// The header text is that start index only when
/// `after_next_page_click_supports_pagination_on_the_block_screen` says
/// the next page header differs. The href is `block_txs_start_index_path`
/// of those arguments.
pub fn paint_supports_pagination_on_the_block_screen(block_hash: &str, start_index: u64) -> String {
    let view =
        splora_frontend_shared::after_next_page_click_supports_pagination_on_the_block_screen();
    let path = splora_frontend_shared::block_txs_start_index_path(block_hash, start_index);
    let header = if view.later_page_header_differs {
        start_index.to_string()
    } else {
        String::new()
    };
    format!("<div class=\"header-bg box\"><a href=\"{path}\">{header}</a></div>")
}
