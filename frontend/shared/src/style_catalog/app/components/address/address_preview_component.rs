use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/address/address-preview.component.scss",
        selector_path: ".title-wrapper",
        properties: &[("padding", "0 15px")],
        leptos_css: "padding: 0 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address-preview.component.scss",
        selector_path: ".qr-wrapper",
        properties: &[
            ("background-color", "#fff"),
            ("padding", "10px"),
            ("padding-bottom", "5px"),
            ("display", "inline-block"),
        ],
        leptos_css: "background-color: #fff; padding: 10px; padding-bottom: 5px; display: inline-block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address-preview.component.scss",
        selector_path: ".qrcode-col",
        properties: &[
            ("width", "468px"),
            ("min-width", "468px"),
            ("flex-grow", "0"),
            ("flex-shrink", "0"),
            ("text-align", "center"),
            ("padding", "0"),
            ("margin-left", "2px"),
            ("margin-right", "15px"),
        ],
        leptos_css: "width: 468px; min-width: 468px; flex-grow: 0; flex-shrink: 0; text-align: center; padding: 0; margin-left: 2px; margin-right: 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address-preview.component.scss",
        selector_path: ".table-col",
        properties: &[("max-width", "calc(100% - 470px)"), ("overflow", "hidden")],
        leptos_css: "max-width: calc(100% - 470px); overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address-preview.component.scss",
        selector_path: ".table",
        properties: &[("font-size", "32px"), ("margin-top", "48px")],
        leptos_css: "font-size: 32px; margin-top: 48px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address-preview.component.scss",
        selector_path: ".table ::ng-deep .symbol",
        properties: &[("font-size", "24px")],
        leptos_css: "font-size: 24px;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
];
