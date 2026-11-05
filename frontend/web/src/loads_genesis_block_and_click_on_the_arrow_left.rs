//! Web paint of the genesis block, then one Next Block click.
//! The labels and route come from
//! `splora_frontend_shared::loads_genesis_block_and_click_on_the_arrow_left`.

/// Visible Next Block and Previous Block labels, then the route.
///
/// The first string is the markup on height 0. The second string is the
/// markup after the Next Block click, including the route. Each icon label
/// is included only when that icon is visible. This paint does not hard-code
/// a block hash.
pub fn paint_loads_genesis_block_and_click_on_the_arrow_left() -> (String, String) {
    let page = splora_frontend_shared::loads_genesis_block_and_click_on_the_arrow_left();
    let after_click = format!(
        "{}{}",
        visible_icon_labels(page.after_next_block_click),
        route_markup(&page.route)
    );
    (visible_icon_labels(page.on_genesis), after_click)
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
