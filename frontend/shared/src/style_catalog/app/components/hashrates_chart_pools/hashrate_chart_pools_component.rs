use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
        selector_path: ".card-header",
        properties: &[("border-bottom", "0"), ("font-size", "18px")],
        leptos_css: "border-bottom: 0; font-size: 18px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
        selector_path: "@media (min-width: 465px) { .card-header }",

        properties: &[("font-size", "20px")],
        leptos_css: "font-size: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
        selector_path: "@media (min-width: 992px) { .card-header }",

        properties: &[("height", "40px")],
        leptos_css: "height: 40px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
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
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
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
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
        selector_path: "@media (min-width: 992px) { .full-container }",

        properties: &[("height", "calc(100vh - 150px)")],
        leptos_css: "height: calc(100vh - 150px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
        selector_path: ".chart",
        properties: &[
            ("display", "flex"),
            ("flex", "1"),
            ("width", "100%"),
            ("padding-bottom", "20px"),
            ("padding-right", "10px"),
        ],
        leptos_css: "display: flex; flex: 1; width: 100%; padding-bottom: 20px; padding-right: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
        selector_path: "@media (max-width: 992px) { .chart }",

        properties: &[("padding-bottom", "25px")],
        leptos_css: "padding-bottom: 25px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
        selector_path: "@media (max-width: 829px) { .chart }",

        properties: &[("padding-bottom", "50px")],
        leptos_css: "padding-bottom: 50px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
        selector_path: "@media (max-width: 767px) { .chart }",

        properties: &[("padding-bottom", "50px")],
        leptos_css: "padding-bottom: 50px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
        selector_path: ".chart-widget",
        properties: &[
            ("width", "100%"),
            ("height", "100%"),
            ("max-height", "293px"),
        ],
        leptos_css: "width: 100%; height: 100%; max-height: 293px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
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
        source_path: "app/components/hashrates-chart-pools/hashrate-chart-pools.component.scss",
        selector_path: ".loadingGraphs.widget",
        properties: &[("top", "75%")],
        leptos_css: "top: 75%;",
        gpui: GpuiOutcome::Paint,
    },
];
