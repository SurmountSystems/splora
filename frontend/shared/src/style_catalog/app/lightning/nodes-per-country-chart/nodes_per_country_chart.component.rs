use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".sats",
        properties: &[
            ("color", "var(--transparent-fg)"),
            ("font-size", "12px"),
            ("top", "0px"),
        ],
        leptos_css: "color: var(--transparent-fg); font-size: 12px; top: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".card-header",
        properties: &[("border-bottom", "0"), ("font-size", "18px")],
        leptos_css: "border-bottom: 0; font-size: 18px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "@media (min-width: 465px) { .card-header }",

        properties: &[("font-size", "20px")],
        leptos_css: "font-size: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".full-container",
        properties: &[
            ("padding", "0px 15px"),
            ("width", "100%"),
            ("height", "calc(100% - 140px)"),
        ],
        leptos_css: "padding: 0px 15px; width: 100%; height: calc(100% - 140px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "@media (max-width: 992px) { .full-container }",

        properties: &[("height", "calc(100% - 190px)")],
        leptos_css: "height: calc(100% - 190px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "@media (max-width: 575px) { .full-container }",

        properties: &[("height", "calc(100% - 230px)")],
        leptos_css: "height: calc(100% - 230px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".chart",
        properties: &[("max-height", "400px")],
        leptos_css: "max-height: 400px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "@media (max-width: 767.98px) { .chart }",

        properties: &[("max-height", "230px"), ("margin-top", "-35px")],
        leptos_css: "max-height: 230px; margin-top: -35px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".rank",
        properties: &[("width", "8%")],
        leptos_css: "width: 8%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "@media (max-width: 576px) { .rank }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".name",
        properties: &[("width", "36%")],
        leptos_css: "width: 36%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "@media (max-width: 576px) { .name }",

        properties: &[
            ("width", "80%"),
            ("max-width", "150px"),
            ("padding-left", "0"),
            ("padding-right", "0"),
        ],
        leptos_css: "width: 80%; max-width: 150px; padding-left: 0; padding-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".share",
        properties: &[("width", "15%")],
        leptos_css: "width: 15%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "@media (max-width: 576px) { .share }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".nodes",
        properties: &[("width", "15%")],
        leptos_css: "width: 15%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "@media (max-width: 576px) { .nodes }",

        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".capacity",
        properties: &[("width", "26%")],
        leptos_css: "width: 26%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "@media (max-width: 576px) { .capacity }",

        properties: &[("width", "10%"), ("max-width", "100px")],
        leptos_css: "width: 10%; max-width: 100px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "a",
        properties: &[("text-decoration", "none")],
        leptos_css: "text-decoration: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: "a:hover .link",
        properties: &[("text-decoration", "underline")],
        leptos_css: "text-decoration: underline;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".flag",
        properties: &[("font-size", "20px")],
        leptos_css: "font-size: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-country-chart/nodes-per-country-chart.component.scss",
        selector_path: ".text-truncate .link",
        properties: &[("overflow", "hidden"), ("text-overflow", "ellipsis")],
        leptos_css: "overflow: hidden; text-overflow: ellipsis;",
        gpui: GpuiOutcome::Paint,
    },
];
