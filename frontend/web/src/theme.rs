// splora-frontend-shared does not export these colors. Keep this palette.
pub const BACKGROUND: &str = "#000000";
pub const TEXT: &str = "#FFFFFF";
pub const ACCENT: &str = "#FFD100";

// `mod theme` is private. The style-sheet test calls this export.
#[cfg_attr(
    not(target_arch = "wasm32"),
    unsafe(export_name = "splora_web_catalog_css")
)]
pub fn catalog_css() -> String {
    splora_frontend_shared::all_rules()
        .into_iter()
        .map(|rule| rule.leptos_css)
        .filter(|css| !css.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
