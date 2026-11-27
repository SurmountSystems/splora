use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/block/block-transactions.component.scss",
        selector_path: ".block-tx-title",
        properties: &[
            ("display", "flex"),
            ("justify-content", "space-between"),
            ("flex-direction", "column"),
            ("margin-top", "-15px"),
            ("position", "relative"),
        ],
        leptos_css: "display: flex; justify-content: space-between; flex-direction: column; margin-top: -15px; position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-transactions.component.scss",
        selector_path: "@media (min-width: 550px) { .block-tx-title }",
        properties: &[("margin-top", "1rem"), ("flex-direction", "row")],
        leptos_css: "margin-top: 1rem; flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-transactions.component.scss",
        selector_path: ".block-tx-title h2",
        properties: &[
            ("line-height", "1"),
            ("margin", "0"),
            ("position", "relative"),
            ("padding-bottom", "10px"),
        ],
        leptos_css: "line-height: 1; margin: 0; position: relative; padding-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-transactions.component.scss",
        selector_path: "@media (min-width: 550px) { .block-tx-title h2 }",
        properties: &[("padding-bottom", "0px"), ("align-self", "end")],
        leptos_css: "padding-bottom: 0px; align-self: end;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-transactions.component.scss",
        selector_path: ".tx-skeleton",
        properties: &[("margin-top", "10px"), ("margin-bottom", "10px")],
        leptos_css: "margin-top: 10px; margin-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-transactions.component.scss",
        selector_path: ".tx-skeleton .header-bg:first-child",
        properties: &[("padding", "10px"), ("margin-bottom", "10px")],
        leptos_css: "padding: 10px; margin-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-transactions.component.scss",
        selector_path: ".tx-skeleton .header-bg:nth-child(2) .row",
        properties: &[("height", "107px")],
        leptos_css: "height: 107px;",
        gpui: GpuiOutcome::Paint,
    },
];
