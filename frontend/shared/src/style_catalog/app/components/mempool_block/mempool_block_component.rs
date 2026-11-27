use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/mempool-block/mempool-block.component.scss",
        selector_path: ".progress",
        properties: &[
            ("background-color", "var(--secondary)"),
            ("position", "relative"),
            ("top", "5px"),
        ],
        leptos_css: "background-color: var(--secondary); position: relative; top: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-block/mempool-block.component.scss",
        selector_path: ".fiat",
        properties: &[
            ("font-size", "13px"),
            ("display", "inline-block"),
            ("margin-left", "10px"),
        ],
        leptos_css: "font-size: 13px; display: inline-block; margin-left: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-block/mempool-block.component.scss",
        selector_path: ".table tr td:last-child",
        properties: &[("text-align", "right")],
        leptos_css: "text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-block/mempool-block.component.scss",
        selector_path: "@media (min-width: 992px) { .table tr td:last-child }",

        properties: &[("text-align", "left")],
        leptos_css: "text-align: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-block/mempool-block.component.scss",
        selector_path: "h1",
        properties: &[("margin", "0px"), ("padding", "0px"), ("line-height", "1")],
        leptos_css: "margin: 0px; padding: 0px; line-height: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-block/mempool-block.component.scss",
        selector_path: "@media (min-width: 576px) { h1 }",

        properties: &[("float", "left")],
        leptos_css: "float: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-block/mempool-block.component.scss",
        selector_path: ".chart-container",
        properties: &[("margin", "20px auto")],
        leptos_css: "margin: 20px auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-block/mempool-block.component.scss",
        selector_path: "@media (min-width: 768px) { .chart-container }",

        properties: &[("margin", "auto")],
        leptos_css: "margin: auto;",
        gpui: GpuiOutcome::Paint,
    },
];
