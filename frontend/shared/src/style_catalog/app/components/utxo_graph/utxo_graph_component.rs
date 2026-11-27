use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/utxo-graph/utxo-graph.component.scss",
        selector_path: ".card-header",
        properties: &[("border-bottom", "0"), ("font-size", "18px")],
        leptos_css: "border-bottom: 0; font-size: 18px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/utxo-graph/utxo-graph.component.scss",
        selector_path: "@media (min-width: 465px) { .card-header }",

        properties: &[("font-size", "20px")],
        leptos_css: "font-size: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/utxo-graph/utxo-graph.component.scss",
        selector_path: "@media (min-width: 992px) { .card-header }",

        properties: &[("height", "40px")],
        leptos_css: "height: 40px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/utxo-graph/utxo-graph.component.scss",
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
        source_path: "app/components/utxo-graph/utxo-graph.component.scss",
        selector_path: ".full-container",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "column"),
            ("padding", "0px"),
            ("width", "100%"),
            ("height", "400px"),
        ],
        leptos_css: "display: flex; flex-direction: column; padding: 0px; width: 100%; height: 400px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/utxo-graph/utxo-graph.component.scss",
        selector_path: ".error-wrapper",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "column"),
            ("width", "100%"),
            ("height", "100%"),
            ("align-items", "center"),
            ("justify-content", "center"),
            ("font-size", "15px"),
            ("color", "grey"),
            ("font-weight", "bold"),
        ],
        leptos_css: "display: flex; flex-direction: column; width: 100%; height: 100%; align-items: center; justify-content: center; font-size: 15px; color: grey; font-weight: bold;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/utxo-graph/utxo-graph.component.scss",
        selector_path: ".chart",
        properties: &[
            ("display", "flex"),
            ("flex", "1"),
            ("width", "100%"),
            ("padding-right", "10px"),
        ],
        leptos_css: "display: flex; flex: 1; width: 100%; padding-right: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/utxo-graph/utxo-graph.component.scss",
        selector_path: ".chart-widget",
        properties: &[("width", "100%"), ("height", "100%")],
        leptos_css: "width: 100%; height: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/utxo-graph/utxo-graph.component.scss",
        selector_path: ".disabled",
        properties: &[("pointer-events", "none"), ("opacity", "0.5")],
        leptos_css: "pointer-events: none; opacity: 0.5;",
        gpui: GpuiOutcome::Paint,
    },
];
