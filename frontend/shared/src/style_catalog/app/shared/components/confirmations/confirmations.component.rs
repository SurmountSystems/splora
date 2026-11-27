use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/shared/components/confirmations/confirmations.component.scss",
        selector_path: ".no-cursor",
        properties: &[("cursor", "default !important"), ("pointer-events", "none")],
        leptos_css: "cursor: default !important; pointer-events: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/confirmations/confirmations.component.scss",
        selector_path: "@media (max-width: 430px) { .confirmation-compact }",
        properties: &[("font-size", "12px")],
        leptos_css: "font-size: 12px;",
        gpui: GpuiOutcome::Paint,
    },
];
