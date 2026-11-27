use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/block-overview-graph/block-overview-graph.component.scss",
        selector_path: ".block-overview-graph",
        properties: &[
            ("position", "relative"),
            ("width", "100%"),
            ("padding-bottom", "100%"),
            ("background", "var(--stat-box-bg)"),
            ("display", "flex"),
            ("justify-content", "center"),
            ("align-items", "center"),
            ("grid-column", "1/-1"),
        ],
        leptos_css: "position: relative; width: 100%; padding-bottom: 100%; background: var(--stat-box-bg); display: flex; justify-content: center; align-items: center; grid-column: 1/-1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-graph/block-overview-graph.component.scss",
        selector_path: ".block-overview-graph .placeholder",
        properties: &[
            ("display", "flex"),
            ("position", "absolute"),
            ("left", "0"),
            ("right", "0"),
            ("top", "0"),
            ("bottom", "0"),
            ("height", "100%"),
            ("width", "100%"),
            ("align-items", "center"),
            ("justify-content", "center"),
        ],
        leptos_css: "display: flex; position: absolute; left: 0; right: 0; top: 0; bottom: 0; height: 100%; width: 100%; align-items: center; justify-content: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-graph/block-overview-graph.component.scss",
        selector_path: ".graph-alignment",
        properties: &[("position", "relative"), ("width", "100%")],
        leptos_css: "position: relative; width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-graph/block-overview-graph.component.scss",
        selector_path: ".grid-align",
        properties: &[
            ("display", "grid"),
            ("grid-template-columns", "repeat(auto-fit, 75px)"),
            ("justify-content", "center"),
        ],
        leptos_css: "display: grid; grid-template-columns: repeat(auto-fit, 75px); justify-content: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-graph/block-overview-graph.component.scss",
        selector_path: ".block-overview-canvas",
        properties: &[
            ("position", "absolute"),
            ("left", "0"),
            ("right", "0"),
            ("top", "0"),
            ("bottom", "0"),
            ("width", "100%"),
            ("height", "100%"),
            ("overflow", "hidden"),
        ],
        leptos_css: "position: absolute; left: 0; right: 0; top: 0; bottom: 0; width: 100%; height: 100%; overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-graph/block-overview-graph.component.scss",
        selector_path: ".block-overview-canvas.clickable",
        properties: &[("cursor", "pointer")],
        leptos_css: "cursor: pointer;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-graph/block-overview-graph.component.scss",
        selector_path: ".loader-wrapper",
        properties: &[
            ("position", "absolute"),
            ("background", "var(--block-goggles-overlay)"),
            ("left", "0"),
            ("right", "0"),
            ("top", "0"),
            ("bottom", "0"),
            ("display", "flex"),
            ("justify-content", "center"),
            ("align-items", "center"),
            ("transition", "opacity 500ms 500ms"),
            ("pointer-events", "none"),
        ],
        leptos_css: "position: absolute; background: var(--block-goggles-overlay); left: 0; right: 0; top: 0; bottom: 0; display: flex; justify-content: center; align-items: center; transition: opacity 500ms 500ms; pointer-events: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-graph/block-overview-graph.component.scss",
        selector_path: ".loader-wrapper.hidden",
        properties: &[("opacity", "0")],
        leptos_css: "opacity: 0;",
        gpui: GpuiOutcome::Paint,
    },
];
