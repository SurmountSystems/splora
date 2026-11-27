use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-wallet/federation-wallet.component.scss",
        selector_path: "ul",
        properties: &[("margin-bottom", "20px")],
        leptos_css: "margin-bottom: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-wallet/federation-wallet.component.scss",
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
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-wallet/federation-wallet.component.scss",
        selector_path: "@media (max-width: 436px) { .nav-link }",

        properties: &[("padding", "0.8rem 0.8rem"), ("font-size", "0.8rem")],
        leptos_css: "padding: 0.8rem 0.8rem; font-size: 0.8rem;",
        gpui: GpuiOutcome::Paint,
    },
];
