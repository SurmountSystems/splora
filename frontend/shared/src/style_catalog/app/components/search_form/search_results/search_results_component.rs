use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/search-form/search-results/search-results.component.scss",
        selector_path: ".card-title",
        properties: &[
            ("color", "var(--title-fg)"),
            ("font-size", "10px"),
            ("margin-bottom", "4px"),
            ("font-size", "1rem"),
            ("margin-left", "10px"),
        ],
        leptos_css: "color: var(--title-fg); font-size: 10px; margin-bottom: 4px; font-size: 1rem; margin-left: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-results/search-results.component.scss",
        selector_path: ".danger",
        properties: &[("color", "var(--red)")],
        leptos_css: "color: var(--red);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-results/search-results.component.scss",
        selector_path: ".dropdown-menu",
        properties: &[
            ("position", "absolute"),
            ("top", "42px"),
            ("left", "0px"),
            (
                "box-shadow",
                "0.125rem 0.125rem 0.25rem rgba(0, 0, 0, 0.075)",
            ),
            ("width", "100%"),
        ],
        leptos_css: "position: absolute; top: 42px; left: 0px; box-shadow: 0.125rem 0.125rem 0.25rem rgba(0, 0, 0, 0.075); width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-results/search-results.component.scss",
        selector_path: ".inactive",
        properties: &[("opacity", "0.2")],
        leptos_css: "opacity: 0.2;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-results/search-results.component.scss",
        selector_path: ".active",
        properties: &[("background-color", "var(--active-bg)")],
        leptos_css: "background-color: var(--active-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-results/search-results.component.scss",
        selector_path: ".pool-logo",
        properties: &[
            ("width", "15px"),
            ("height", "15px"),
            ("position", "relative"),
            ("top", "-1px"),
            ("margin-right", "10px"),
        ],
        leptos_css: "width: 15px; height: 15px; position: relative; top: -1px; margin-right: 10px;",
        gpui: GpuiOutcome::Paint,
    },
];
