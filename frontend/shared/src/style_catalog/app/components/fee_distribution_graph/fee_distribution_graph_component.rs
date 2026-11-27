use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[StyleRule {
    source_path: "app/components/fee-distribution-graph/fee-distribution-graph.component.scss",
    selector_path: ".fee-distribution-chart",
    properties: &[("margin-top", "0.75rem")],
    leptos_css: "margin-top: 0.75rem;",
    gpui: GpuiOutcome::Paint,
}];
