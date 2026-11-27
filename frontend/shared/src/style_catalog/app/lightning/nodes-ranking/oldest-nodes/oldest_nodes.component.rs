use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".container-xl",
        properties: &[("max-width", "1400px"), ("padding-bottom", "100px")],
        leptos_css: "max-width: 1400px; padding-bottom: 100px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: "@media (min-width: 767.98px) { .container-xl }",

        properties: &[("padding-left", "50px"), ("padding-right", "50px")],
        leptos_css: "padding-left: 50px; padding-right: 50px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".table td, .table th",
        properties: &[("padding", "0.5rem")],
        leptos_css: "padding: 0.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".full .rank",
        properties: &[("width", "5%")],
        leptos_css: "width: 5%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: "@media (min-width: 767.98px) { .widget .rank }",

        properties: &[("width", "13%")],
        leptos_css: "width: 13%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: "@media (max-width: 767.98px) { .widget .rank }",

        properties: &[("padding-left", "0px"), ("padding-right", "0px")],
        leptos_css: "padding-left: 0px; padding-right: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".full .alias",
        properties: &[
            ("width", "10%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("max-width", "350px"),
        ],
        leptos_css: "width: 10%; overflow: hidden; text-overflow: ellipsis; max-width: 350px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: "@media (max-width: 767.98px) { .full .alias }",

        properties: &[("max-width", "175px")],
        leptos_css: "max-width: 175px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".widget .alias",
        properties: &[
            ("width", "50%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("max-width", "300px"),
        ],
        leptos_css: "width: 50%; overflow: hidden; text-overflow: ellipsis; max-width: 300px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: "@media (max-width: 767.98px) { .widget .alias }",

        properties: &[("max-width", "170px")],
        leptos_css: "max-width: 170px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".full .capacity",
        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: "@media (max-width: 767.98px) { .full .capacity }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".widget .capacity",
        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: "@media (max-width: 767.98px) { .widget .capacity }",

        properties: &[("padding-left", "0px"), ("padding-right", "0px")],
        leptos_css: "padding-left: 0px; padding-right: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".full .channels",
        properties: &[("width", "15%"), ("padding-right", "50px")],
        leptos_css: "width: 15%; padding-right: 50px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: "@media (max-width: 767.98px) { .full .channels }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".full .timestamp-first",
        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".full .timestamp-update",
        properties: &[("width", "20%")],
        leptos_css: "width: 20%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: "@media (max-width: 767.98px) { .full .timestamp-update }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: ".full .location",
        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/oldest-nodes/oldest-nodes.component.scss",
        selector_path: "@media (max-width: 767.98px) { .full .location }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
];
