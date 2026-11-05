//! Web render of the settled skeleton-loader markers.
//! The network transition lives in frontend/shared.

use splora_frontend_shared::loads_skeleton_when_changes_between_networks;

/// One `.skeleton-loader` element for each settled count after `first`, then `second`.
pub fn paint_loads_skeleton_when_changes_between_networks(first: &str, second: &str) -> String {
    let change = loads_skeleton_when_changes_between_networks(first, second);
    "<div class=\"skeleton-loader\"></div>".repeat(change.settled as usize)
}
