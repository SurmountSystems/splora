use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/wallet/wallet-preview.component.scss",
        selector_path: ".title-wrapper",
        properties: &[("padding", "0 15px")],
        leptos_css: "padding: 0 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet-preview.component.scss",
        selector_path: ".graph-col",
        properties: &[
            ("height", "350px"),
            ("text-align", "center"),
            ("padding", "0"),
            ("margin-left", "2px"),
            ("margin-right", "15px"),
        ],
        leptos_css: "height: 350px; text-align: center; padding: 0; margin-left: 2px; margin-right: 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet-preview.component.scss",
        selector_path: ".table-col",
        properties: &[("overflow", "hidden")],
        leptos_css: "overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet-preview.component.scss",
        selector_path: ".table",
        properties: &[("font-size", "32px")],
        leptos_css: "font-size: 32px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet-preview.component.scss",
        selector_path: ".table ::ng-deep .symbol",
        properties: &[("font-size", "24px")],
        leptos_css: "font-size: 24px;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/wallet/wallet-preview.component.scss",
        selector_path: ".table .spacer",
        properties: &[("background", "none")],
        leptos_css: "background: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet-preview.component.scss",
        selector_path: ".fiat",
        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
];
