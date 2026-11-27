use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".card-header",
        properties: &[("border-bottom", "0"), ("font-size", "18px")],
        leptos_css: "border-bottom: 0; font-size: 18px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 465px) { .card-header }",

        properties: &[("font-size", "20px")],
        leptos_css: "font-size: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 992px) { .card-header }",

        properties: &[("height", "40px")],
        leptos_css: "height: 40px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".main-title",
        properties: &[
            ("position", "relative"),
            ("color", "var(--fg)"),
            ("opacity", "var(--opacity)"),
            ("margin-top", "-13px"),
            ("font-size", "10px"),
            ("text-transform", "uppercase"),
            ("font-weight", "500"),
            ("text-align", "center"),
            ("padding-bottom", "3px"),
        ],
        leptos_css: "position: relative; color: var(--fg); opacity: var(--opacity); margin-top: -13px; font-size: 10px; text-transform: uppercase; font-weight: 500; text-align: center; padding-bottom: 3px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".full-container",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "column"),
            ("padding", "0px 15px"),
            ("width", "100%"),
            ("height", "calc(100vh - 225px)"),
            ("min-height", "400px"),
        ],
        leptos_css: "display: flex; flex-direction: column; padding: 0px 15px; width: 100%; height: calc(100vh - 225px); min-height: 400px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 992px) { .full-container }",

        properties: &[("height", "calc(100vh - 150px)")],
        leptos_css: "height: calc(100vh - 150px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".chart",
        properties: &[
            ("display", "flex"),
            ("flex", "1"),
            ("height", "100%"),
            ("padding-bottom", "20px"),
            ("padding-right", "10px"),
        ],
        leptos_css: "display: flex; flex: 1; height: 100%; padding-bottom: 20px; padding-right: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (max-width: 992px) { .chart }",

        properties: &[("padding-bottom", "25px")],
        leptos_css: "padding-bottom: 25px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (max-width: 829px) { .chart }",

        properties: &[("padding-bottom", "50px")],
        leptos_css: "padding-bottom: 50px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (max-width: 767px) { .chart }",

        properties: &[("padding-bottom", "25px")],
        leptos_css: "padding-bottom: 25px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (max-width: 629px) { .chart }",

        properties: &[("padding-bottom", "55px")],
        leptos_css: "padding-bottom: 55px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (max-width: 567px) { .chart }",

        properties: &[("padding-bottom", "55px")],
        leptos_css: "padding-bottom: 55px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".chart-widget",
        properties: &[
            ("width", "100%"),
            ("height", "100%"),
            ("max-height", "238px"),
        ],
        leptos_css: "width: 100%; height: 100%; max-height: 238px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".block-fee-rates",
        properties: &[("min-height", "56px"), ("display", "block")],
        leptos_css: "min-height: 56px; display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 485px) { .block-fee-rates }",

        properties: &[("display", "flex"), ("flex-direction", "row")],
        leptos_css: "display: flex; flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".block-fee-rates h5",
        properties: &[("margin-bottom", "10px")],
        leptos_css: "margin-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".block-fee-rates .item",
        properties: &[
            ("width", "50%"),
            ("display", "inline-block"),
            ("margin", "0px auto 20px"),
        ],
        leptos_css: "width: 50%; display: inline-block; margin: 0px auto 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".block-fee-rates .item:nth-child(2)",
        properties: &[("order", "2")],
        leptos_css: "order: 2;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 485px) { .block-fee-rates .item:nth-child(2) }",

        properties: &[("order", "3")],
        leptos_css: "order: 3;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".block-fee-rates .item:nth-child(3)",
        properties: &[("order", "3")],
        leptos_css: "order: 3;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 485px) { .block-fee-rates .item:nth-child(3) }",

        properties: &[("order", "2"), ("display", "block")],
        leptos_css: "order: 2; display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 768px) { .block-fee-rates .item:nth-child(3) }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 992px) { .block-fee-rates .item:nth-child(3) }",

        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".block-fee-rates .item .card-title",
        properties: &[("font-size", "1rem"), ("color", "var(--title-fg)")],
        leptos_css: "font-size: 1rem; color: var(--title-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".block-fee-rates .item .card-text",
        properties: &[("font-size", "18px")],
        leptos_css: "font-size: 18px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".block-fee-rates .item .card-text span",
        properties: &[("color", "var(--transparent-fg)"), ("font-size", "12px")],
        leptos_css: "color: var(--transparent-fg); font-size: 12px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".formRadioGroup",
        properties: &[
            ("margin-top", "6px"),
            ("display", "flex"),
            ("flex-direction", "column"),
        ],
        leptos_css: "margin-top: 6px; display: flex; flex-direction: column;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 991px) { .formRadioGroup }",

        properties: &[("position", "relative"), ("top", "-100px")],
        leptos_css: "position: relative; top: -100px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 830px) and (max-width: 991px) { .formRadioGroup }",

        properties: &[("position", "relative"), ("top", "0px")],
        leptos_css: "position: relative; top: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 830px) { .formRadioGroup }",

        properties: &[
            ("flex-direction", "row"),
            ("float", "right"),
            ("margin-top", "0px"),
        ],
        leptos_css: "flex-direction: row; float: right; margin-top: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".formRadioGroup .btn-sm",
        properties: &[("font-size", "9px")],
        leptos_css: "font-size: 9px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: "@media (min-width: 830px) { .formRadioGroup .btn-sm }",

        properties: &[("font-size", "14px")],
        leptos_css: "font-size: 14px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-fee-rates-graph/block-fee-rates-graph.component.scss",
        selector_path: ".skeleton-loader",
        properties: &[
            ("width", "100%"),
            ("display", "block"),
            ("max-width", "80px"),
            ("margin", "15px auto 3px"),
        ],
        leptos_css: "width: 100%; display: block; max-width: 80px; margin: 15px auto 3px;",
        gpui: GpuiOutcome::Paint,
    },
];
