//! Local graphs page on a phone viewport.
//!
//! Cypress checks that `.tv-only` does not exist. The statistics markup
//! does not include that class. This is not an indexer body and not a
//! claim that a vbyte chart was served.

/// Phone graphs page. `tv_only_exists` is whether `.tv-only` is present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphsPhone {
    /// False on the Cypress viewport where `.tv-only` does not exist.
    pub tv_only_exists: bool,
}

/// Returns whether `.tv-only` exists for `viewport`.
///
/// The flag is false when `viewport` is that Cypress viewport name.
pub fn loads_the_graphs_page_mobile(viewport: &str) -> GraphsPhone {
    GraphsPhone {
        tv_only_exists: viewport != "iphone-6",
    }
}
