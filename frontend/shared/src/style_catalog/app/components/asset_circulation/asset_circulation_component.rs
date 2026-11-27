use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[StyleRule {
    source_path: "app/components/asset-circulation/asset-circulation.component.scss",
    selector_path: ".ticker",
    properties: &[("color", "grey")],
    leptos_css: "color: grey;",
    gpui: GpuiOutcome::Paint,
}];
