//! Native render glue for the settled skeleton-loader markers.
//! The network transition lives in frontend/shared.

use splora_frontend_shared::skeleton_loader_count_after_network_changes;

/// Repeats `skeleton-loader` once for each settled count after `first`, then `second`.
pub fn paint_loads_skeleton_when_changes_between_networks(first: &str, second: &str) -> String {
    let count = skeleton_loader_count_after_network_changes(first, second);
    "skeleton-loader".repeat(count as usize)
}
