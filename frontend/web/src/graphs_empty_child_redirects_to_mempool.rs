//! Web paint of the empty graphs child redirect.
//!
//! The target is the shared function's return value. This file does not
//! hard-code that target.

/// Paints the empty graphs child redirect. The string is the shared function.
pub fn paint_graphs_empty_child_redirects_to_mempool() -> &'static str {
    splora_frontend_shared::graphs_empty_child_redirects_to_mempool()
}
