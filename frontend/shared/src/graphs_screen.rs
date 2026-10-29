//! Graphs screen after `#btn-graphs` is clicked.
//!
//! The default child is mempool vbyte charts. Splora does not serve
//! `/api/v1/statistics`, and no existing response type is that time series.
//! `graphs_screen_active` is the after-click view only. It is not an index body.

/// View after the graphs button is clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphsScreen {
    pub graphs_screen_active: bool,
    /// HTML id of the clicked control. Not the `#btn-graphs` selector.
    pub button_id: &'static str,
    /// Explorer path that control routes to. Not an indexer path.
    pub route: &'static str,
}

/// No argument: this call is the graphs button already clicked, so the screen is active.
/// The flag is not a caller-set success bit and not a read of an index body.
pub fn after_graphs_button_loads_the_graphs_screen() -> GraphsScreen {
    GraphsScreen {
        graphs_screen_active: true,
        button_id: "btn-graphs",
        route: "/graphs",
    }
}
