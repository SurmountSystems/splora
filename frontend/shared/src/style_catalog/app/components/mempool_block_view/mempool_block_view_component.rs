use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/mempool-block-view/mempool-block-view.component.scss",
        selector_path: ".block-wrapper",
        properties: &[
            ("width", "100vw"),
            ("height", "100vh"),
            ("background", "var(--stat-box-bg)"),
        ],
        leptos_css: "width: 100vw; height: 100vh; background: var(--stat-box-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-block-view/mempool-block-view.component.scss",
        selector_path: ".block-container",
        properties: &[
            ("flex-grow", "0"),
            ("flex-shrink", "0"),
            ("width", "100vw"),
            ("max-width", "100vh"),
            ("height", "100vh"),
            ("padding", "0"),
            ("margin", "auto"),
            ("display", "flex"),
            ("justify-content", "center"),
            ("align-items", "center"),
        ],
        leptos_css: "flex-grow: 0; flex-shrink: 0; width: 100vw; max-width: 100vh; height: 100vh; padding: 0; margin: auto; display: flex; justify-content: center; align-items: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-block-view/mempool-block-view.component.scss",
        selector_path: ".block-container *",
        properties: &[("flex-grow", "1")],
        leptos_css: "flex-grow: 1;",
        gpui: GpuiOutcome::Paint,
    },
];
