use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/address-labels/address-labels.component.scss",
        selector_path: ".badge",
        properties: &[("margin-right", "2px")],
        leptos_css: "margin-right: 2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-labels/address-labels.component.scss",
        selector_path: ".badge-positioner",
        properties: &[("position", "absolute")],
        leptos_css: "position: absolute;",
        gpui: GpuiOutcome::Paint,
    },
];
