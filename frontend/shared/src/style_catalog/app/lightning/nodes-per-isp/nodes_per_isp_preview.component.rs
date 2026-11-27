use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp-preview.component.scss",
        selector_path: ".table",
        properties: &[("font-size", "32px"), ("margin-top", "0px")],
        leptos_css: "font-size: 32px; margin-top: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp-preview.component.scss",
        selector_path: ".map-col",
        properties: &[
            ("flex-grow", "0"),
            ("flex-shrink", "0"),
            ("width", "470px"),
            ("height", "360px"),
            ("min-width", "470px"),
            ("min-height", "360px"),
            ("max-height", "360px"),
            ("padding", "0"),
            ("background", "var(--stat-box-bg)"),
            ("overflow", "hidden"),
            ("margin-top", "0"),
        ],
        leptos_css: "flex-grow: 0; flex-shrink: 0; width: 470px; height: 360px; min-width: 470px; min-height: 360px; max-height: 360px; padding: 0; background: var(--stat-box-bg); overflow: hidden; margin-top: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp-preview.component.scss",
        selector_path: ".row",
        properties: &[("margin-right", "0")],
        leptos_css: "margin-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp-preview.component.scss",
        selector_path: ".full-width-row",
        properties: &[("padding-left", "15px"), ("flex-wrap", "nowrap")],
        leptos_css: "padding-left: 15px; flex-wrap: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp-preview.component.scss",
        selector_path: "::ng-deep .symbol",
        properties: &[("font-size", "24px")],
        leptos_css: "font-size: 24px;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
];
