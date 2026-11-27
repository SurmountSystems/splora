use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/nodes-rankings-dashboard/nodes-rankings-dashboard.component.scss",
        selector_path: ".main",
        properties: &[("max-width", "90%")],
        leptos_css: "max-width: 90%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-rankings-dashboard/nodes-rankings-dashboard.component.scss",
        selector_path: ".col",
        properties: &[
            ("padding-bottom", "20px"),
            ("padding-left", "10px"),
            ("padding-right", "10px"),
        ],
        leptos_css: "padding-bottom: 20px; padding-left: 10px; padding-right: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-rankings-dashboard/nodes-rankings-dashboard.component.scss",
        selector_path: ".card",
        properties: &[("background-color", "var(--bg)")],
        leptos_css: "background-color: var(--bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-rankings-dashboard/nodes-rankings-dashboard.component.scss",
        selector_path: ".card-title",
        properties: &[("font-size", "1rem"), ("color", "var(--title-fg)")],
        leptos_css: "font-size: 1rem; color: var(--title-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-rankings-dashboard/nodes-rankings-dashboard.component.scss",
        selector_path: ".card-title > a",
        properties: &[("color", "var(--title-fg)")],
        leptos_css: "color: var(--title-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-rankings-dashboard/nodes-rankings-dashboard.component.scss",
        selector_path: ".card-text",
        properties: &[("font-size", "22px")],
        leptos_css: "font-size: 22px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-rankings-dashboard/nodes-rankings-dashboard.component.scss",
        selector_path: ".title-link, .title-link:hover, .title-link:focus, .title-link:active",
        properties: &[
            ("text-align", "center"),
            ("display", "block"),
            ("margin-bottom", "10px"),
            ("text-decoration", "none"),
            ("color", "inherit"),
        ],
        leptos_css: "text-align: center; display: block; margin-bottom: 10px; text-decoration: none; color: inherit;",
        gpui: GpuiOutcome::Paint,
    },
];
