use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[StyleRule {
    source_path: "app/components/tx-fee-rating/tx-fee-rating.component.scss",
    selector_path: ".badge",
    properties: &[("position", "relative"), ("top", "-1px")],
    leptos_css: "position: relative; top: -1px;",
    gpui: GpuiOutcome::Paint,
}];
