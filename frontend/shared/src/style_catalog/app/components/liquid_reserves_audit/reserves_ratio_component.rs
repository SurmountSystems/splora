use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio/reserves-ratio.component.scss",
        selector_path: ".loadingGraphs",
        properties: &[
            ("position", "absolute"),
            ("top", "50%"),
            ("left", "50%"),
            ("transform", "translate(-50%, -50%)"),
            ("width", "100%"),
            ("text-align", "center"),
            ("z-index", "99"),
        ],
        leptos_css: "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); width: 100%; text-align: center; z-index: 99;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio/reserves-ratio.component.scss",
        selector_path: ".loadingGraphs .audit-in-progress-text",
        properties: &[
            ("color", "var(--transparent-fg)"),
            ("font-size", "14px"),
            ("font-weight", "500"),
        ],
        leptos_css: "color: var(--transparent-fg); font-size: 14px; font-weight: 500;",
        gpui: GpuiOutcome::Paint,
    },
];
