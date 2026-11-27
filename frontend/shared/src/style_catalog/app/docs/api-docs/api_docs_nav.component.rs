use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/docs/api-docs/api-docs-nav.component.scss",
        selector_path: "p",
        properties: &[
            ("color", "var(--title-fg)"),
            ("font-weight", "700"),
            ("margin", "10px 0"),
            ("margin", "15px 0 10px 0"),
        ],
        leptos_css: "color: var(--title-fg); font-weight: 700; margin: 10px 0; margin: 15px 0 10px 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/docs/api-docs/api-docs-nav.component.scss",
        selector_path: "a",
        properties: &[
            ("color", "var(--fg)"),
            ("text-decoration", "none"),
            ("display", "block"),
            ("margin", "5px 0"),
            ("cursor", "pointer"),
        ],
        leptos_css: "color: var(--fg); text-decoration: none; display: block; margin: 5px 0; cursor: pointer;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/docs/api-docs/api-docs-nav.component.scss",
        selector_path: "#enterprise-cta-desktop",
        properties: &[
            ("text-align", "center"),
            ("padding", "20px"),
            ("margin", "20px 20px 20px 0"),
            ("background-color", "var(--bg)"),
            ("border-radius", "12px"),
        ],
        leptos_css: "text-align: center; padding: 20px; margin: 20px 20px 20px 0; background-color: var(--bg); border-radius: 12px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/docs/api-docs/api-docs-nav.component.scss",
        selector_path: "#enterprise-cta-desktop p",
        properties: &[
            ("margin", "0 auto 16px auto"),
            ("color", "var(--fg)"),
            ("font-weight", "400"),
        ],
        leptos_css: "margin: 0 auto 16px auto; color: var(--fg); font-weight: 400;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/docs/api-docs/api-docs-nav.component.scss",
        selector_path: "#enterprise-cta-desktop a",
        properties: &[("display", "inline-block")],
        leptos_css: "display: inline-block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/docs/api-docs/api-docs-nav.component.scss",
        selector_path: "fa-icon",
        properties: &[("font-size", "12px")],
        leptos_css: "font-size: 12px;",
        gpui: GpuiOutcome::Paint,
    },
];
