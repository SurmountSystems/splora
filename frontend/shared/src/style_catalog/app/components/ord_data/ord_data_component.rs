use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/ord-data/ord-data.component.scss",
        selector_path: ".amount",
        properties: &[("font-weight", "bold")],
        leptos_css: "font-weight: bold;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/ord-data/ord-data.component.scss",
        selector_path: "a.rune-link",
        properties: &[("color", "inherit")],
        leptos_css: "color: inherit;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/ord-data/ord-data.component.scss",
        selector_path: "a.rune-link:hover",
        properties: &[
            ("text-decoration", "underline"),
            ("text-decoration-color", "var(--transparent-fg)"),
        ],
        leptos_css: "text-decoration: underline; text-decoration-color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/ord-data/ord-data.component.scss",
        selector_path: "a.disabled",
        properties: &[("text-decoration", "none")],
        leptos_css: "text-decoration: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/ord-data/ord-data.component.scss",
        selector_path: ".name",
        properties: &[("color", "var(--transparent-fg)"), ("font-weight", "700")],
        leptos_css: "color: var(--transparent-fg); font-weight: 700;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/ord-data/ord-data.component.scss",
        selector_path: ".badge-ord",
        properties: &[
            ("--bs-btn-bg", "var(--grey)"),
            ("--bs-btn-border-color", "transparent"),
            ("background-color", "var(--bs-btn-bg)"),
            ("border-color", "var(--bs-btn-border-color)"),
            ("position", "relative"),
            ("top", "-2px"),
            ("font-size", "81%"),
        ],
        leptos_css: "--bs-btn-bg: var(--grey); --bs-btn-border-color: transparent; background-color: var(--bs-btn-bg); border-color: var(--bs-btn-border-color); position: relative; top: -2px; font-size: 81%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/ord-data/ord-data.component.scss",
        selector_path: ".badge-ord.primary",
        properties: &[("--bs-btn-bg", "var(--primary)")],
        leptos_css: "--bs-btn-bg: var(--primary);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/ord-data/ord-data.component.scss",
        selector_path: "pre",
        properties: &[("margin-top", "5px"), ("max-height", "200px")],
        leptos_css: "margin-top: 5px; max-height: 200px;",
        gpui: GpuiOutcome::Paint,
    },
];
