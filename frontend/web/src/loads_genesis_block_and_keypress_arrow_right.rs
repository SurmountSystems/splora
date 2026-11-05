//! Web paint of the genesis block after one ArrowRight.
//! The labels and route come from
//! `splora_frontend_shared::loads_genesis_block_and_keypress_arrow_right`.

/// Visible Next Block and Previous Block labels, plus the route.
///
/// The shared return has no heading. Each icon label is included only when
/// that icon is visible. The route is the shared return's route.
pub fn paint_loads_genesis_block_and_keypress_arrow_right() -> String {
    let page = splora_frontend_shared::loads_genesis_block_and_keypress_arrow_right();
    let mut markup = visible_icon_labels(page.after_arrow_right);
    markup.push_str(&route_markup(&page.route));
    markup
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
