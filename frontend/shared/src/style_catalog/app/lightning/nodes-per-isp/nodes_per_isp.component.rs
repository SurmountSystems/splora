use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: ".container-xl",
        properties: &[("max-width", "1400px"), ("padding-bottom", "100px")],
        leptos_css: "max-width: 1400px; padding-bottom: 100px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
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
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: ".alias",
        properties: &[
            ("width", "30%"),
            ("max-width", "400px"),
            ("padding-right", "70px"),
        ],
        leptos_css: "width: 30%; max-width: 400px; padding-right: 70px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: "@media (max-width: 576px) { .alias }",

        properties: &[
            ("width", "50%"),
            ("max-width", "150px"),
            ("padding-right", "0px"),
        ],
        leptos_css: "width: 50%; max-width: 150px; padding-right: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: ".timestamp-first",
        properties: &[("width", "20%")],
        leptos_css: "width: 20%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: "@media (max-width: 1060px) { .timestamp-first }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: ".timestamp-update",
        properties: &[("width", "16%")],
        leptos_css: "width: 16%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: "@media (max-width: 1060px) { .timestamp-update }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: ".capacity",
        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: "@media (max-width: 576px) { .capacity }",

        properties: &[("width", "25%")],
        leptos_css: "width: 25%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: ".channels",
        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: "@media (max-width: 576px) { .channels }",

        properties: &[("width", "25%")],
        leptos_css: "width: 25%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: ".city",
        properties: &[("max-width", "150px")],
        leptos_css: "max-width: 150px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-per-isp/nodes-per-isp.component.scss",
        selector_path: "@media (max-width: 675px) { .city }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
];
