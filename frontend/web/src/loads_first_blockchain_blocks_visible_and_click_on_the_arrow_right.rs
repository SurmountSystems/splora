//! Web paint of the newest confirmed block, then one Previous Block click.
//! The labels and route come from
//! `splora_frontend_shared::loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right`.

/// Visible Next Block and Previous Block labels, then the route.
///
/// The first string is the markup after the newest confirmed block is
/// selected. The second string is the markup after the Previous Block
/// click, including the route. Each icon label is included only when that
/// icon is visible. This paint does not hard-code a block hash.
pub fn paint_loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right() -> (String, String)
{
    let page =
        splora_frontend_shared::loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right(
        );
    let after_click = format!(
        "{}{}",
        visible_icon_labels(page.after_previous_block_click),
        route_markup(&page.route)
    );
    (
        visible_icon_labels(page.after_selecting_newest),
        after_click,
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

fn route_markup(route: &str) -> String {
    format!("<a href=\"{route}\">{route}</a>")
}
