use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/assets/assets-featured/assets-featured.component.scss",
        selector_path: ".featuredBox",
        properties: &[
            ("display", "flex"),
            ("flex-flow", "row wrap"),
            ("justify-content", "center"),
            ("gap", "27px"),
        ],
        leptos_css: "display: flex; flex-flow: row wrap; justify-content: center; gap: 27px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-featured/assets-featured.component.scss",
        selector_path: ".card",
        properties: &[
            ("background-color", "var(--bg)"),
            ("width", "200px"),
            ("height", "200px"),
            ("align-items", "center"),
            ("justify-content", "center"),
            ("flex-wrap", "wrap"),
        ],
        leptos_css: "background-color: var(--bg); width: 200px; height: 200px; align-items: center; justify-content: center; flex-wrap: wrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-featured/assets-featured.component.scss",
        selector_path: "@media (max-width: 767.98px) { .card }",

        properties: &[("width", "150px"), ("height", "150px")],
        leptos_css: "width: 150px; height: 150px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-featured/assets-featured.component.scss",
        selector_path: ".title",
        properties: &[
            ("font-size", "14px"),
            ("font-weight", "bold"),
            ("margin-top", "10px"),
            ("text-align", "center"),
        ],
        leptos_css: "font-size: 14px; font-weight: bold; margin-top: 10px; text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-featured/assets-featured.component.scss",
        selector_path: ".sub-title",
        properties: &[("color", "grey"), ("font-size", "12px")],
        leptos_css: "color: grey; font-size: 12px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-featured/assets-featured.component.scss",
        selector_path: ".assetIcon",
        properties: &[("width", "100px"), ("height", "100px")],
        leptos_css: "width: 100px; height: 100px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-featured/assets-featured.component.scss",
        selector_path: "@media (max-width: 767.98px) { .assetIcon }",

        properties: &[("width", "50px"), ("height", "50px")],
        leptos_css: "width: 50px; height: 50px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-featured/assets-featured.component.scss",
        selector_path: ".view-link",
        properties: &[("margin-top", "30px")],
        leptos_css: "margin-top: 30px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-featured/assets-featured.component.scss",
        selector_path: ".ticker",
        properties: &[("color", "grey")],
        leptos_css: "color: grey;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets-featured/assets-featured.component.scss",
        selector_path: ".symbol",
        properties: &[
            ("color", "grey"),
            ("font-size", "1.5rem"),
            ("font-style", "italic"),
        ],
        leptos_css: "color: grey; font-size: 1.5rem; font-style: italic;",
        gpui: GpuiOutcome::Paint,
    },
];
