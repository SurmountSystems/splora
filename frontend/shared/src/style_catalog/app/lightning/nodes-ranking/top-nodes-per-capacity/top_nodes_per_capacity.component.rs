use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: ".container-xl",
        properties: &[("max-width", "1400px")],
        leptos_css: "max-width: 1400px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: ".container-xl.widget",
        properties: &[
            ("padding-right", "0px"),
            ("padding-left", "0px"),
            ("padding-bottom", "0px"),
        ],
        leptos_css: "padding-right: 0px; padding-left: 0px; padding-bottom: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: "tr, td, th",
        properties: &[
            ("border", "0px"),
            ("padding-top", "0.65rem !important"),
            ("padding-bottom", "0.7rem !important"),
        ],
        leptos_css: "border: 0px; padding-top: 0.65rem !important; padding-bottom: 0.7rem !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: ".clear-link",
        properties: &[("color", "white")],
        leptos_css: "color: white;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: ".pool",
        properties: &[
            ("width", "15%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "160px"),
        ],
        leptos_css: "width: 15%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 160px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: "@media (max-width: 575px) { .pool }",

        properties: &[("width", "75%")],
        leptos_css: "width: 75%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: ".pool-name",
        properties: &[
            ("display", "inline-block"),
            ("vertical-align", "text-top"),
            ("text-overflow", "ellipsis"),
            ("overflow", "hidden"),
        ],
        leptos_css: "display: inline-block; vertical-align: text-top; text-overflow: ellipsis; overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: ".liquidity",
        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: "@media (max-width: 575px) { .liquidity }",

        properties: &[("width", "25%")],
        leptos_css: "width: 25%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: ".capacity-ratio",
        properties: &[("font-size", "12px"), ("color", "var(--transparent-fg)")],
        leptos_css: "font-size: 12px; color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: ".fiat",
        properties: &[("width", "15%")],
        leptos_css: "width: 15%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: "@media (min-width: 768px) and (max-width: 991px) { .fiat }",

        properties: &[("display", "none !important")],
        leptos_css: "display: none !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-ranking/top-nodes-per-capacity/top-nodes-per-capacity.component.scss",
        selector_path: "@media (max-width: 575px) { .fiat }",

        properties: &[("display", "none !important")],
        leptos_css: "display: none !important;",
        gpui: GpuiOutcome::Paint,
    },
];
