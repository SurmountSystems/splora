//! Api screen after `#btn-docs` is clicked.
//!
//! Signet visits `/signet`, waits for the skeleton, then clicks `#btn-docs`.
//! That control routes to `/docs`. `api_screen_active` is the after-click view
//! only. It is not an indexer body.

/// View after the docs button is clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiScreen {
    pub api_screen_active: bool,
}

/// No argument: this call is the docs button already clicked, so the screen is active.
/// The flag is not a caller-set success bit and not a read of an index body.
pub fn after_docs_button_loads_the_api_screen() -> ApiScreen {
    ApiScreen {
        api_screen_active: true,
    }
}
