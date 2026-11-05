//! Native render glue for the settled skeleton-loader markers.
//! The network transition lives in frontend/shared.

/// Paints peak, settled, and network after changing to `first`, then `second`.
pub fn paint_loads_skeleton_when_changes_between_networks(first: &str, second: &str) -> String {
    let change =
        splora_frontend_shared::loads_skeleton_when_changes_between_networks(first, second);
    format!(
        "peak={}\nsettled={}\nnetwork={}",
        change.peak, change.settled, change.network
    )
}
