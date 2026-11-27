use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/assets/assets-nav/assets-nav.component.scss",
        selector_path: "ul",
        properties: &[("margin-bottom", "20px"), ("float", "left")],
        leptos_css: "margin-bottom: 20px; float: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-nav/assets-nav.component.scss",
        selector_path: "form",
        properties: &[("float", "right"), ("width", "300px")],
        leptos_css: "float: right; width: 300px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-nav/assets-nav.component.scss",
        selector_path: "@media (max-width: 767.98px) { form }",

        properties: &[("width", "90%"), ("margin-bottom", "15px")],
        leptos_css: "width: 90%; margin-bottom: 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-nav/assets-nav.component.scss",
        selector_path: "@media (max-width: 767.98px) { .nav-container }",

        properties: &[
            ("display", "flex"),
            ("flex-direction", "column"),
            ("align-items", "center"),
            ("justify-content", "center"),
            ("margin", "auto"),
        ],
        leptos_css: "display: flex; flex-direction: column; align-items: center; justify-content: center; margin: auto;",
        gpui: GpuiOutcome::Paint,
    },
];
