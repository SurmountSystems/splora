use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/docs/docs/docs.component.scss",
        selector_path: "#main-tab-content",
        properties: &[
            ("text-align", "left"),
            ("padding-top", "1rem"),
            ("scroll-behavior", "smooth"),
            ("margin-bottom", "20px"),
        ],
        leptos_css: "text-align: left; padding-top: 1rem; scroll-behavior: smooth; margin-bottom: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/docs/docs/docs.component.scss",
        selector_path: "#footer",
        properties: &[("clear", "both")],
        leptos_css: "clear: both;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/docs/docs/docs.component.scss",
        selector_path: "@media (max-width: 992px) { .hide-on-mobile }",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
];
