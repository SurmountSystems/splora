//! Api screen after `#btn-docs` is clicked.
//!
//! That control's routerLink is `/docs`. The docs routing module's empty
//! child path `redirectTo` is the client destination. `api_screen_active`
//! is the after-click view only. It is not an indexer body.

/// View after a nav control is clicked.
///
/// `destination` is the empty-path `redirectTo` from `docs_routes` when
/// the control is `#btn-docs`. Any other control leaves it empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiScreen<'a> {
    pub api_screen_active: bool,
    /// Client destination after the docs-area redirect. Not an indexer path.
    pub destination: &'a str,
}

/// Cypress selector for the docs control in `master-page.component.html`.
const DOCS_BUTTON_SELECTOR: &str = "#btn-docs";
/// Same control's `id` attribute, without the CSS hash.
const DOCS_BUTTON_ID: &str = "btn-docs";
/// Docs area entry (`path: ''` with `pathMatch: 'full'`).
const DOCS_ENTRY_PATH: &str = "";

/// Clicked control plus docs child routes as `(path, redirect_to)`.
///
/// The api screen is active only for `#btn-docs`. Its destination is the
/// `redirectTo` of the empty path in `docs_routes`. A different control
/// does not receive that redirect.
pub fn after_docs_button_loads_the_api_screen<'a>(
    clicked_control: &str,
    docs_routes: &'a [(&'a str, &'a str)],
) -> ApiScreen<'a> {
    let docs_button = clicked_control == DOCS_BUTTON_SELECTOR || clicked_control == DOCS_BUTTON_ID;
    if !docs_button {
        return ApiScreen {
            api_screen_active: false,
            destination: "",
        };
    }
    let destination = docs_routes
        .iter()
        .find(|(path, _redirect_to)| *path == DOCS_ENTRY_PATH)
        .map(|(_path, redirect_to)| *redirect_to)
        .unwrap_or("");
    ApiScreen {
        api_screen_active: true,
        destination,
    }
}
