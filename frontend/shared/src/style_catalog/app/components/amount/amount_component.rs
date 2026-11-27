use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[StyleRule {
    source_path: "app/components/amount/amount.component.scss",
    selector_path: ".green-color",
    properties: &[("color", "var(--green)")],
    leptos_css: "color: var(--green);",
    gpui: GpuiOutcome::Paint,
}];
