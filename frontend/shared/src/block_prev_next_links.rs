//! Previous and next block-link visibility for one confirmed height.
//!
//! The previous link is shown only when the height is present and not
//! JavaScript-falsy. A missing height and height 0 both hide it. Any other
//! height shows it, including a height equal to the latest confirmed height.
//!
//! The next link is hidden only when the block height equals the latest
//! confirmed height. A missing height is not that height, so the next link
//! stays shown.

/// Returns `(previous_link_shown, next_link_shown)`.
///
/// Previous follows `!this.blockHeight`: missing and zero hide the link.
/// Next follows `latestBlock.height === this.blockHeight` when a latest
/// height is always present: equal heights hide the link.
pub fn block_prev_next_links(
    block_height: Option<u32>,
    latest_confirmed_height: u32,
) -> (bool, bool) {
    let previous_link_shown = matches!(block_height, Some(height) if height != 0);
    let next_link_shown = block_height != Some(latest_confirmed_height);
    (previous_link_shown, next_link_shown)
}

/// Next Block and Previous Block icon visibility on one confirmed block.
///
/// The Cypress selector is the tooltip anchor, not the disabled stand-in.
/// Next is visible only when the next link is shown. Previous is visible
/// only when the previous link is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockNavIcons {
    pub next_block_icon_visible: bool,
    pub previous_block_icon_visible: bool,
}

/// Icons after the newest-block click, then after one ArrowRight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstBlockchainBlockArrowRight {
    pub after_selecting_newest: BlockNavIcons,
    pub after_arrow_right: BlockNavIcons,
}

/// Smallest tip where one older step is still not genesis.
///
/// Dashboard index 0 is the newest confirmed block. Height 2 is that
/// local tip. It is not a block hash and not a fee.
const NEWEST_VISIBLE_CONFIRMED_HEIGHT: u32 = 2;

/// Icon states for the newest visible confirmed block, then one ArrowRight.
///
/// `right` dispatches keydown `ArrowRight`. The default time direction is
/// right to left unless the locale is Arabic, Persian, or Hebrew, or a
/// saved preference forces left to right. ArrowRight is then the previous
/// block key, so one press steps from the tip to the tip minus one. The
/// tip hides Next Block and shows Previous Block. The older block shows
/// both.
pub fn loads_first_blockchain_block_visible_and_keypress_arrow_right()
-> FirstBlockchainBlockArrowRight {
    let tip = NEWEST_VISIBLE_CONFIRMED_HEIGHT;
    let (previous, next) = block_prev_next_links(Some(tip), tip);
    let after_selecting_newest = BlockNavIcons {
        next_block_icon_visible: next,
        previous_block_icon_visible: previous,
    };
    let older = tip - 1;
    let (previous, next) = block_prev_next_links(Some(older), tip);
    FirstBlockchainBlockArrowRight {
        after_selecting_newest,
        after_arrow_right: BlockNavIcons {
            next_block_icon_visible: next,
            previous_block_icon_visible: previous,
        },
    }
}

/// Icons after selecting the newest block, then after the Previous Block click.
///
/// `height` and `route` are the confirmed block after that click. Index 0
/// is the newest visible confirmed block. This is not a block hash, a fee,
/// or a price.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstBlockchainBlocksClickArrowRight {
    pub after_selecting_newest: BlockNavIcons,
    pub after_previous_block_click: BlockNavIcons,
    pub height: u32,
    pub route: String,
}

/// Newest visible confirmed block, then one Previous Block click.
///
/// The first click selects index 0, the newest visible confirmed block.
/// `block_prev_next_links` at that tip hides Next Block and shows Previous
/// Block. Default `timeLtr` is false, so the Previous Block click is the
/// older-block step, the same direction as ArrowRight. One older step is
/// the tip minus one. Icon flags after that click come from
/// `block_prev_next_links` at the older height, not from a hard-coded pair.
pub fn loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right()
-> FirstBlockchainBlocksClickArrowRight {
    let tip = NEWEST_VISIBLE_CONFIRMED_HEIGHT;
    let (previous, next) = block_prev_next_links(Some(tip), tip);
    let after_selecting_newest = BlockNavIcons {
        next_block_icon_visible: next,
        previous_block_icon_visible: previous,
    };
    // Default timeLtr is false, so the Previous Block click is the older
    // step, the same direction as ArrowRight. The previous link is shown
    // at the tip, so the click navigates to the tip minus one.
    let time_ltr = false;
    let previous_click_is_older = !time_ltr;
    let height = if previous_click_is_older && previous {
        tip - 1
    } else {
        tip
    };
    let (previous, next) = block_prev_next_links(Some(height), tip);
    FirstBlockchainBlocksClickArrowRight {
        after_selecting_newest,
        after_previous_block_click: BlockNavIcons {
            next_block_icon_visible: next,
            previous_block_icon_visible: previous,
        },
        height,
        route: format!("/block/{height}"),
    }
}

/// Newest-block icons, then the mempool heading after one ArrowLeft.
///
/// `heading` is `.title-block h1` after the key. `route` is where that key
/// navigates. Both are mempool index 0, not the confirmed block title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstBlockchainBlockArrowLeft {
    pub after_selecting_newest: BlockNavIcons,
    pub heading: String,
    pub route: String,
}

/// Icon pair on the newest visible confirmed block, then one ArrowLeft.
///
/// `left` dispatches keydown `ArrowLeft`. Default `timeLtr` is false, so
/// ArrowLeft is the next-block key. The tip hides Next Block and shows
/// Previous Block. The next link is hidden, so that key navigates to
/// mempool index 0 instead of a confirmed block hash. The heading is
/// `mempool_block_ordinal_title` for index 0. An empty mempool list still
/// uses that index, and index 0 does not read virtual size.
pub fn loads_first_blockchain_block_visible_and_keypress_arrow_left()
-> FirstBlockchainBlockArrowLeft {
    let tip = NEWEST_VISIBLE_CONFIRMED_HEIGHT;
    let (previous, next) = block_prev_next_links(Some(tip), tip);
    let after_selecting_newest = BlockNavIcons {
        next_block_icon_visible: next,
        previous_block_icon_visible: previous,
    };
    // Default timeLtr is false, so ArrowLeft is the next key. The tip hides
    // that link, and the key opens mempool index 0. Empty-list virtual size
    // is 0. Index 0 does not read it.
    let mempool_index = 0_u64;
    let empty_list_vsize = 0_u64;
    debug_assert!(!after_selecting_newest.next_block_icon_visible);
    let heading =
        crate::mempool_block_ordinal::mempool_block_ordinal_title(mempool_index, empty_list_vsize);
    FirstBlockchainBlockArrowLeft {
        after_selecting_newest,
        heading,
        route: format!("/mempool-block/{mempool_index}"),
    }
}

/// Height, route, and icons after ArrowRight on the genesis block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisBlockArrowRight {
    pub height: u32,
    pub route: String,
    pub after_arrow_right: BlockNavIcons,
}

/// Genesis stays on `/block/0` when ArrowRight is the previous-block key.
///
/// The visit is height 0. Default `timeLtr` is false, so ArrowRight is the
/// previous-block key. `block_prev_next_links` hides that previous link at
/// height 0, so the key does not navigate. A local latest height that is
/// not 0 keeps the next link shown.
pub fn loads_genesis_block_and_keypress_arrow_right() -> GenesisBlockArrowRight {
    let height = 0_u32;
    let latest_confirmed_height = NEWEST_VISIBLE_CONFIRMED_HEIGHT;
    let (previous_link_shown, next_link_shown) =
        block_prev_next_links(Some(height), latest_confirmed_height);
    // Default timeLtr is false, so ArrowRight is the previous key.
    // The previous link is hidden, so the key does not change height or route.
    let time_ltr = false;
    let arrow_right_is_previous = !time_ltr;
    let stayed = !(arrow_right_is_previous && previous_link_shown);
    let height = if stayed {
        height
    } else {
        height.saturating_sub(1)
    };
    GenesisBlockArrowRight {
        height,
        route: format!("/block/{height}"),
        after_arrow_right: BlockNavIcons {
            next_block_icon_visible: next_link_shown,
            previous_block_icon_visible: previous_link_shown,
        },
    }
}

/// Height, route, and icons after ArrowLeft on the genesis block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisBlockArrowLeft {
    pub height: u32,
    pub route: String,
    pub after_arrow_left: BlockNavIcons,
}

/// Genesis moves from `/block/0` to `/block/1` when ArrowLeft is the next key.
///
/// The visit is height 0. Default `timeLtr` is false, so ArrowLeft is the
/// next-block key. `block_prev_next_links` shows that next link at height 0
/// because that height is not the latest confirmed height, so the key
/// navigates to height 1. Icon flags after the key come from
/// `block_prev_next_links` at the height after the key. This key does not
/// open `/mempool-block/0` and does not invent a block hash.
pub fn loads_genesis_block_and_keypress_arrow_left() -> GenesisBlockArrowLeft {
    let height = 0_u32;
    let latest_confirmed_height = NEWEST_VISIBLE_CONFIRMED_HEIGHT;
    let (_previous_at_start, next_at_start) =
        block_prev_next_links(Some(height), latest_confirmed_height);
    // Default timeLtr is false, so ArrowLeft is the next key.
    // The next link is shown at height 0, so the key navigates to height 1.
    // The newest confirmed block is the one that opens `/mempool-block/0`.
    let time_ltr = false;
    let arrow_left_is_next = !time_ltr;
    let height = if arrow_left_is_next && next_at_start {
        height.saturating_add(1)
    } else {
        height
    };
    let (previous_link_shown, next_link_shown) =
        block_prev_next_links(Some(height), latest_confirmed_height);
    GenesisBlockArrowLeft {
        height,
        route: format!("/block/{height}"),
        after_arrow_left: BlockNavIcons {
            next_block_icon_visible: next_link_shown,
            previous_block_icon_visible: previous_link_shown,
        },
    }
}

/// Icons on the genesis block, then after the Next Block click.
///
/// `height` and `route` are the confirmed block after that click. The visit
/// is `/block/0`. This is not a block hash, a fee, or a price, and the click
/// does not open `/mempool-block/0`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisBlockClickArrowLeft {
    pub on_genesis: BlockNavIcons,
    pub after_next_block_click: BlockNavIcons,
    pub height: u32,
    pub route: String,
}

/// Genesis block, then one Next Block click.
///
/// The visit is height 0. `block_prev_next_links` at height 0 shows Next
/// Block and hides Previous Block. Default `timeLtr` is false, so the Next
/// Block click is the newer-block step, the same direction as ArrowLeft.
/// That click is the same newer-block step as
/// `loads_genesis_block_and_keypress_arrow_left`, so the height and route
/// after the click come from that step. Icon flags on genesis and after
/// the click come from `block_prev_next_links` at each height. This click
/// does not open `/mempool-block/0`.
pub fn loads_genesis_block_and_click_on_the_arrow_left() -> GenesisBlockClickArrowLeft {
    let start = 0_u32;
    let latest_confirmed_height = NEWEST_VISIBLE_CONFIRMED_HEIGHT;
    let (previous, next) = block_prev_next_links(Some(start), latest_confirmed_height);
    let on_genesis = BlockNavIcons {
        next_block_icon_visible: next,
        previous_block_icon_visible: previous,
    };
    // Default timeLtr is false, so the Next Block click is the newer step,
    // the same direction as ArrowLeft. The next link is shown at height 0,
    // so the click takes that newer step.
    let time_ltr = false;
    let next_click_is_newer = !time_ltr;
    let (height, route) = if next_click_is_newer && next {
        let after_key = loads_genesis_block_and_keypress_arrow_left();
        (after_key.height, after_key.route)
    } else {
        (start, format!("/block/{start}"))
    };
    let (previous, next) = block_prev_next_links(Some(height), latest_confirmed_height);
    GenesisBlockClickArrowLeft {
        on_genesis,
        after_next_block_click: BlockNavIcons {
            next_block_icon_visible: next,
            previous_block_icon_visible: previous,
        },
        height,
        route,
    }
}
