use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/node/node-preview.component.scss",
        selector_path: ".table-col",
        properties: &[("max-width", "calc(100% - 470px)"), ("overflow", "hidden")],
        leptos_css: "max-width: calc(100% - 470px); overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/node/node-preview.component.scss",
        selector_path: ".table",
        properties: &[("margin-top", "6px"), ("font-size", "32px")],
        leptos_css: "margin-top: 6px; font-size: 32px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/node/node-preview.component.scss",
        selector_path: ".badges",
        properties: &[
            ("font-size", "28px"),
            ("flex-shrink", "0"),
            ("flex-grow", "0"),
            ("display", "flex"),
            ("flex-direction", "row"),
            ("flex-wrap", "nowrap"),
            ("align-items", "baseline"),
            ("justify-content", "flex-end"),
        ],
        leptos_css: "font-size: 28px; flex-shrink: 0; flex-grow: 0; display: flex; flex-direction: row; flex-wrap: nowrap; align-items: baseline; justify-content: flex-end;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/node/node-preview.component.scss",
        selector_path: ".badges ::ng-deep .badge",
        properties: &[("margin-left", "0.5em")],
        leptos_css: "margin-left: 0.5em;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/lightning/node/node-preview.component.scss",
        selector_path: ".map-col",
        properties: &[
            ("flex-grow", "0"),
            ("flex-shrink", "0"),
            ("width", "470px"),
            ("height", "408px"),
            ("min-width", "470px"),
            ("min-height", "408px"),
            ("max-height", "408px"),
            ("padding", "0"),
            ("background", "var(--stat-box-bg)"),
            ("overflow", "hidden"),
            ("margin-top", "6px"),
        ],
        leptos_css: "flex-grow: 0; flex-shrink: 0; width: 470px; height: 408px; min-width: 470px; min-height: 408px; max-height: 408px; padding: 0; background: var(--stat-box-bg); overflow: hidden; margin-top: 6px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/node/node-preview.component.scss",
        selector_path: ".row",
        properties: &[("margin-right", "0")],
        leptos_css: "margin-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/node/node-preview.component.scss",
        selector_path: ".full-width-row",
        properties: &[("padding-left", "15px"), ("flex-wrap", "nowrap")],
        leptos_css: "padding-left: 15px; flex-wrap: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/node/node-preview.component.scss",
        selector_path: "::ng-deep .symbol",
        properties: &[("font-size", "24px")],
        leptos_css: "font-size: 24px;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
];
