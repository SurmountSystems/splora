//! Blocks page after `#btn-blocks` is clicked.
//!
//! Liquid `liquid-master-page.component.html` puts `id="btn-blocks"` on the
//! nav item whose anchor `routerLink` is `['/blocks' | relativeUrl]`.
//! `RelativeUrlPipe` clears the network prefix when `BASE_MODULE` is not
//! `mempool`. Cypress `waitForSkeletonGone` waits until
//! `Cypress.$('.skeleton-loader').length === 0`. This does not read an
//! indexer body.

/// Settled blocks page after the blocks control is clicked.
///
/// `route` is owned because it is formatted from the nav segment.
/// `String` is not `Copy`, so this view is not `Copy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedBlocksPage {
    /// HTML id of `#btn-blocks`, without the CSS hash.
    pub button_id: &'static str,
    /// Explorer path the click follows. Not an indexer GET.
    pub route: String,
    /// `Cypress.$('.skeleton-loader').length` after the click.
    pub skeleton_loader_count: u32,
}

/// The blocks control has been clicked and the skeleton is gone.
///
/// The liquid spec's `BASE_MODULE` is `liquid`, so `relativeUrl` drops the
/// network prefix. The path is the routerLink segment `blocks`, not a pasted
/// route and not a read of a blocks list.
pub fn loads_the_blocks_page() -> LoadedBlocksPage {
    let segment = "blocks";
    LoadedBlocksPage {
        button_id: "btn-blocks",
        route: format!("/{segment}"),
        skeleton_loader_count: 0,
    }
}
