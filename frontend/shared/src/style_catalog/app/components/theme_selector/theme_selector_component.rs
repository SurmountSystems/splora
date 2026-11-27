use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/theme-selector/theme-selector.component.scss",
        selector_path: ".form-select",
        properties: &[("width", "108px")],
        leptos_css: "width: 108px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/theme-selector/theme-selector.component.scss",
        selector_path: ".form-select:disabled",
        properties: &[("opacity", "0.6")],
        leptos_css: "opacity: 0.6;",
        gpui: GpuiOutcome::Paint,
    },
];
