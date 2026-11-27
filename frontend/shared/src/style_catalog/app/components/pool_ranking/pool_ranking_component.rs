use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".card-header",
        properties: &[("border-bottom", "0"), ("font-size", "18px")],
        leptos_css: "border-bottom: 0; font-size: 18px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (min-width: 465px) { .card-header }",

        properties: &[("font-size", "20px")],
        leptos_css: "font-size: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".full-container",
        properties: &[
            ("padding", "0px 15px"),
            ("width", "100%"),
            ("height", "calc(100% - 140px)"),
            ("padding-bottom", "20px"),
        ],
        leptos_css: "padding: 0px 15px; width: 100%; height: calc(100% - 140px); padding-bottom: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (max-width: 992px) { .full-container }",

        properties: &[("height", "calc(100% - 190px)")],
        leptos_css: "height: calc(100% - 190px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (max-width: 575px) { .full-container }",

        properties: &[("height", "calc(100% - 230px)")],
        leptos_css: "height: calc(100% - 230px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".chart",
        properties: &[("max-height", "400px"), ("margin-bottom", "20px")],
        leptos_css: "max-height: 400px; margin-bottom: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (max-width: 767.98px) { .chart }",

        properties: &[("max-height", "230px")],
        leptos_css: "max-height: 230px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".chart-widget",
        properties: &[("width", "100%"), ("height", "100%")],
        leptos_css: "width: 100%; height: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (max-width: 767px) { .chart-widget }",

        properties: &[("max-height", "240px")],
        leptos_css: "max-height: 240px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (max-width: 485px) { .chart-widget }",

        properties: &[("max-height", "200px")],
        leptos_css: "max-height: 200px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (max-width: 767.98px) { .pools-table th, .pools-table td }",

        properties: &[("padding", "0.3em !important")],
        leptos_css: "padding: 0.3em !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (max-width: 430px) { .pool-name }",

        properties: &[
            ("max-width", "110px"),
            ("white-space", "nowrap"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
        ],
        leptos_css: "max-width: 110px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (max-width: 430px) { .health-column }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".loadingGraphs",
        properties: &[
            ("position", "absolute"),
            ("top", "50%"),
            ("left", "calc(50% - 15px)"),
            ("z-index", "99"),
        ],
        leptos_css: "position: absolute; top: 50%; left: calc(50% - 15px); z-index: 99;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".pool-distribution",
        properties: &[("min-height", "56px"), ("display", "block")],
        leptos_css: "min-height: 56px; display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (min-width: 485px) { .pool-distribution }",

        properties: &[("display", "flex"), ("flex-direction", "row")],
        leptos_css: "display: flex; flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".pool-distribution h5",
        properties: &[("margin-bottom", "5px")],
        leptos_css: "margin-bottom: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".pool-distribution .item",
        properties: &[
            ("max-width", "160px"),
            ("width", "50%"),
            ("display", "inline-block"),
            ("margin", "0px auto 20px"),
        ],
        leptos_css: "max-width: 160px; width: 50%; display: inline-block; margin: 0px auto 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".pool-distribution .item:nth-child(2)",
        properties: &[("order", "2")],
        leptos_css: "order: 2;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (min-width: 485px) { .pool-distribution .item:nth-child(2) }",

        properties: &[("order", "3")],
        leptos_css: "order: 3;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".pool-distribution .item:nth-child(3)",
        properties: &[("width", "50%"), ("order", "3")],
        leptos_css: "width: 50%; order: 3;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (min-width: 485px) { .pool-distribution .item:nth-child(3) }",

        properties: &[("order", "2"), ("display", "block")],
        leptos_css: "order: 2; display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (min-width: 768px) { .pool-distribution .item:nth-child(3) }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "@media (min-width: 992px) { .pool-distribution .item:nth-child(3) }",

        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".pool-distribution .item .card-title",
        properties: &[
            ("font-size", "1rem"),
            ("color", "var(--title-fg)"),
            ("display", "inline-block"),
            ("max-width", "100%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "font-size: 1rem; color: var(--title-fg); display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".pool-distribution .item .card-text",
        properties: &[("font-size", "18px")],
        leptos_css: "font-size: 18px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: ".pool-distribution .item .card-text span",
        properties: &[("color", "var(--transparent-fg)"), ("font-size", "12px")],
        leptos_css: "color: var(--transparent-fg); font-size: 12px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
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
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "td .difference.positive",
        properties: &[("color", "rgb(66, 183, 71)")],
        leptos_css: "color: rgb(66, 183, 71);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool-ranking/pool-ranking.component.scss",
        selector_path: "td .difference.negative",
        properties: &[("color", "rgb(183, 66, 66)")],
        leptos_css: "color: rgb(183, 66, 66);",
        gpui: GpuiOutcome::Paint,
    },
];
