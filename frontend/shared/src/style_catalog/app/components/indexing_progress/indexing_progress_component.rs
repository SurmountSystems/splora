use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/indexing-progress/indexing-progress.component.scss",
        selector_path: ":host",
        properties: &[
            ("position", "fixed"),
            ("right", "10px"),
            ("z-index", "1000"),
            ("font-size", "14px"),
            ("display", "flex"),
            ("flex-wrap", "wrap"),
            ("gap", "4px"),
        ],
        leptos_css: "position: fixed; right: 10px; z-index: 1000; font-size: 14px; display: flex; flex-wrap: wrap; gap: 4px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/indexing-progress/indexing-progress.component.scss",
        selector_path: "@media (width >= 992px) { :host }",

        properties: &[("left", "32px"), ("top", "55px")],
        leptos_css: "left: 32px; top: 55px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/indexing-progress/indexing-progress.component.scss",
        selector_path: "@media (576px <= width < 992px) { :host }",

        properties: &[("left", "18px"), ("top", "55px")],
        leptos_css: "left: 18px; top: 55px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/indexing-progress/indexing-progress.component.scss",
        selector_path: "@media (width <= 575px) { :host }",

        properties: &[("left", "18px"), ("top", "100px")],
        leptos_css: "left: 18px; top: 100px;",
        gpui: GpuiOutcome::Paint,
    },
];
