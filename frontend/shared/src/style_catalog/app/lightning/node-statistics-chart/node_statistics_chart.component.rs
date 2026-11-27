use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[StyleRule {
    source_path: "app/lightning/node-statistics-chart/node-statistics-chart.component.scss",
    selector_path: ".full-container",
    properties: &[
        ("margin-top", "25px"),
        ("margin-bottom", "25px"),
        ("min-height", "100%"),
    ],
    leptos_css: "margin-top: 25px; margin-bottom: 25px; min-height: 100%;",
    gpui: GpuiOutcome::Paint,
}];
