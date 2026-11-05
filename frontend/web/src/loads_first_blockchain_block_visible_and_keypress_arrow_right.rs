//! Web paint of the newest confirmed block, then one ArrowRight.
//! The labels come from
//! `splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_right`.

/// Visible Next Block and Previous Block labels.
///
/// The first string is the markup after the newest confirmed block is
/// selected. The second string is the markup after one ArrowRight.
pub fn paint_loads_first_blockchain_block_visible_and_keypress_arrow_right() -> (String, String) {
    let icons =
        splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_right();
    (
        visible_icon_labels(icons.after_selecting_newest),
        visible_icon_labels(icons.after_arrow_right),
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
