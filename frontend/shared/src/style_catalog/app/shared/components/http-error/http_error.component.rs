use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[StyleRule {
    source_path: "app/shared/components/http-error/http-error.component.scss",
    selector_path: ".http-error",
    properties: &[
        ("width", "100%"),
        ("margin", "1em auto"),
        ("text-align", "center"),
    ],
    leptos_css: "width: 100%; margin: 1em auto; text-align: center;",
    gpui: GpuiOutcome::Paint,
}];
