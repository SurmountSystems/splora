//! Web paint of the newest confirmed block, then one ArrowLeft.
//! The labels, heading, and route come from
//! `splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_left`.

/// Visible Next Block and Previous Block labels, then the heading and route.
///
/// The first string is the markup after the newest confirmed block is
/// selected. The second string is the markup after one ArrowLeft.
pub fn paint_loads_first_blockchain_block_visible_and_keypress_arrow_left() -> (String, String) {
    let page =
        splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_left();
    (
        visible_icon_labels(page.after_selecting_newest),
        after_arrow_left_markup(&page.heading, &page.route),
    )
}

fn visible_icon_labels(icons: splora_frontend_shared::BlockNavIcons) -> String {
    let mut markup = String::new();
    if icons.next_block_icon_visible {
        markup.push_str("<span ngbtooltip=\"Next Block\">Next Block</span>");
    }
    if icons.previous_block_icon_visible {
        markup.push_str("<span ngbtooltip=\"Previous Block\">Previous Block</span>");
    }
    markup
}

fn after_arrow_left_markup(heading: &str, route: &str) -> String {
    format!("<div class=\"title-block\"><h1>{heading}</h1></div><a href=\"{route}\">{route}</a>")
}
