//! Native paint of an explorer asset page path.
//! This module does not open a window.

/// Forwards the asset id into the shared asset page path.
pub fn paint_asset_page_path(asset_id: &str) -> String {
    splora_frontend_shared::asset_page_path(asset_id)
}
