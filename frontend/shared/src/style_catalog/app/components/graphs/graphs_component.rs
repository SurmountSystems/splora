use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/graphs/graphs.component.scss",
        selector_path: ".menu",
        properties: &[("flex-grow", "1"), ("padding", "0 35px")],
        leptos_css: "flex-grow: 1; padding: 0 35px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/graphs/graphs.component.scss",
        selector_path: "@media (min-width: 576px) { .menu }",

        properties: &[("max-width", "600px")],
        leptos_css: "max-width: 600px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/graphs/graphs.component.scss",
        selector_path: ".menu > *",
        properties: &[("margin", "0"), ("margin-inline-end", "0.25rem")],
        leptos_css: "margin: 0; margin-inline-end: 0.25rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/graphs/graphs.component.scss",
        selector_path: ".menu > *.last-child",
        properties: &[("margin-inline-end", "0")],
        leptos_css: "margin-inline-end: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/graphs/graphs.component.scss",
        selector_path: ".menu > *",
        properties: &[("margin-bottom", "5px")],
        leptos_css: "margin-bottom: 5px;",
        gpui: GpuiOutcome::Paint,
    },
];
