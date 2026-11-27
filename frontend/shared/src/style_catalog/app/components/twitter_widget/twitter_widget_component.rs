use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[StyleRule {
    source_path: "app/components/twitter-widget/twitter-widget.component.scss",
    selector_path: ".spinner-wrapper, .error-wrapper",
    properties: &[
        ("position", "absolute"),
        ("left", "0"),
        ("right", "0"),
        ("top", "0"),
        ("bottom", "0"),
        ("display", "flex"),
        ("justify-content", "center"),
        ("align-items", "center"),
    ],
    leptos_css: "position: absolute; left: 0; right: 0; top: 0; bottom: 0; display: flex; justify-content: center; align-items: center;",
    gpui: GpuiOutcome::Paint,
}];
