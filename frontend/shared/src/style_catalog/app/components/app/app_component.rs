use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/app/app.component.scss",
        selector_path: "footer",
        properties: &[("max-width", "960px")],
        leptos_css: "max-width: 960px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/app/app.component.scss",
        selector_path: ".logo",
        properties: &[("height", "40px")],
        leptos_css: "height: 40px;",
        gpui: GpuiOutcome::Paint,
    },
];
