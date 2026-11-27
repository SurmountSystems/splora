use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[StyleRule {
    source_path: "app/lightning/node-fee-chart/node-fee-chart.component.scss",
    selector_path: ".full-container",
    properties: &[
        ("position", "relative"),
        ("margin-top", "25px"),
        ("margin-bottom", "25px"),
        ("min-height", "100%"),
        ("min-height", "450px"),
    ],
    leptos_css: "position: relative; margin-top: 25px; margin-bottom: 25px; min-height: 100%; min-height: 450px;",
    gpui: GpuiOutcome::Paint,
}];
