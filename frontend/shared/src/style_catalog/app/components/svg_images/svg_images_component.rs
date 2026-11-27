use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[StyleRule {
    source_path: "app/components/svg-images/svg-images.component.scss",
    selector_path: "svg",
    properties: &[("overflow", "visible")],
    leptos_css: "overflow: visible;",
    gpui: GpuiOutcome::Paint,
}];
