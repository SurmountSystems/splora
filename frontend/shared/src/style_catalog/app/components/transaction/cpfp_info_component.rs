use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/transaction/cpfp-info.component.scss",
        selector_path: ".title h2",
        properties: &[
            ("line-height", "1"),
            ("margin", "0"),
            ("padding-bottom", "5px"),
        ],
        leptos_css: "line-height: 1; margin: 0; padding-bottom: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/cpfp-info.component.scss",
        selector_path: ".cpfp-details .txids",
        properties: &[("width", "60%")],
        leptos_css: "width: 60%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/cpfp-info.component.scss",
        selector_path: "@media (max-width: 500px) { .cpfp-details .txids }",
        properties: &[("width", "40%")],
        leptos_css: "width: 40%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/cpfp-info.component.scss",
        selector_path: ".arrow-green",
        properties: &[("color", "var(--success)")],
        leptos_css: "color: var(--success);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/cpfp-info.component.scss",
        selector_path: ".arrow-red",
        properties: &[("color", "var(--red)")],
        leptos_css: "color: var(--red);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/cpfp-info.component.scss",
        selector_path: ".badge",
        properties: &[("position", "relative"), ("top", "-1px")],
        leptos_css: "position: relative; top: -1px;",
        gpui: GpuiOutcome::Paint,
    },
];
