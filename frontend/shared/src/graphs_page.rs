//! Graphs page after `#btn-graphs` is clicked.
//!
//! Liquid `liquid-master-page.component.html` puts `id="btn-graphs"` on the
//! nav item whose anchor `routerLink` is `['/graphs' | relativeUrl]`.
//! `RelativeUrlPipe` clears the network prefix when `BASE_MODULE` is not
//! `mempool`. Cypress `waitForSkeletonGone` waits until
//! `Cypress.$('.skeleton-loader').length === 0` before that click. This
//! does not read an indexer body and does not read `/api/v1/statistics`.

/// Settled graphs page after the graphs control is clicked.
///
/// `route` is owned because it is formatted from the nav segment.
/// `String` is not `Copy`, so this view is not `Copy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedGraphsPage {
    /// HTML id of `#btn-graphs`, without the CSS hash.
    pub button_id: &'static str,
    /// Explorer path the click follows. Not an indexer GET.
    pub route: String,
    /// `Cypress.$('.skeleton-loader').length` before the click.
    pub skeleton_loader_count: u32,
}

/// The graphs control has been clicked and the skeleton was already gone.
///
/// The liquid spec's `BASE_MODULE` is `liquid`, so `relativeUrl` drops the
/// network prefix. The path is the routerLink segment `graphs`, not a pasted
/// route and not a read of a statistics series.
pub fn loads_the_graphs_page() -> LoadedGraphsPage {
    let segment = "graphs";
    LoadedGraphsPage {
        button_id: "btn-graphs",
        route: format!("/{segment}"),
        skeleton_loader_count: 0,
    }
}
