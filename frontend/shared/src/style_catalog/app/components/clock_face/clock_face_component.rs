use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face",
        properties: &[
            ("position", "relative"),
            ("height", "84.375%"),
            ("margin", "auto"),
            ("overflow", "hidden"),
        ],
        leptos_css: "position: relative; height: 84.375%; margin: auto; overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .cut-out, .clock-face .demo-dial",
        properties: &[
            ("position", "absolute"),
            ("left", "0"),
            ("right", "0"),
            ("top", "0"),
            ("bottom", "0"),
            ("width", "100%"),
            ("height", "100%"),
        ],
        leptos_css: "position: absolute; left: 0; right: 0; top: 0; bottom: 0; width: 100%; height: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .cut-out .face, .clock-face .demo-dial .face",
        properties: &[("fill", "var(--active-bg)")],
        leptos_css: "fill: var(--active-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .gnomon",
        properties: &[("transform-origin", "center"), ("stroke-linejoin", "round")],
        leptos_css: "transform-origin: center; stroke-linejoin: round;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .gnomon.minute",
        properties: &[
            ("fill", "#80C2E1"),
            ("stroke", "#80C2E1"),
            ("stroke-width", "2px"),
        ],
        leptos_css: "fill: #80C2E1; stroke: #80C2E1; stroke-width: 2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .gnomon.hour",
        properties: &[
            ("fill", "var(--primary)"),
            ("stroke", "var(--primary)"),
            ("stroke-width", "6px"),
        ],
        leptos_css: "fill: var(--primary); stroke: var(--primary); stroke-width: 6px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .tick",
        properties: &[
            ("transform-origin", "center"),
            ("fill", "none"),
            ("stroke", "white"),
            ("stroke-width", "2px"),
            ("stroke-linecap", "butt"),
        ],
        leptos_css: "transform-origin: center; fill: none; stroke: white; stroke-width: 2px; stroke-linecap: butt;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .tick.minor",
        properties: &[("stroke-opacity", "0.5")],
        leptos_css: "stroke-opacity: 0.5;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .tick.very.major",
        properties: &[("stroke-width", "4px")],
        leptos_css: "stroke-width: 4px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .block-segment",
        properties: &[
            ("fill", "none"),
            ("stroke", "url(#dial-gradient)"),
            ("stroke-width", "18px"),
        ],
        leptos_css: "fill: none; stroke: url(#dial-gradient); stroke-width: 18px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .dial-segment",
        properties: &[
            ("fill", "none"),
            ("stroke", "white"),
            ("stroke-width", "2px"),
        ],
        leptos_css: "fill: none; stroke: white; stroke-width: 2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clock-face/clock-face.component.scss",
        selector_path: ".clock-face .dial-gradient-img",
        properties: &[("transform-origin", "center")],
        leptos_css: "transform-origin: center;",
        gpui: GpuiOutcome::Paint,
    },
];
