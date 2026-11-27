use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: ".qr-wrapper",
        properties: &[
            ("background-color", "#fff"),
            ("padding", "10px"),
            ("padding-bottom", "5px"),
            ("display", "inline-block"),
            ("margin-right", "25px"),
        ],
        leptos_css: "background-color: #fff; padding: 10px; padding-bottom: 5px; display: inline-block; margin-right: 25px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: "@media (min-width: 576px) { .qrcode-col }",

        properties: &[("text-align", "right")],
        leptos_css: "text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: "@media (max-width: 575.98px) { .qrcode-col }",

        properties: &[("text-align", "center")],
        leptos_css: "text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: "@media (max-width: 575.98px) { .qrcode-col > div }",

        properties: &[("margin-top", "20px"), ("margin-right", "0px")],
        leptos_css: "margin-top: 20px; margin-right: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: "h1",
        properties: &[
            ("margin", "0px"),
            ("padding", "0px"),
            ("margin-right", "15px"),
        ],
        leptos_css: "margin: 0px; padding: 0px; margin-right: 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: "@media (min-width: 576px) { h1 }",

        properties: &[("float", "left")],
        leptos_css: "float: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: ".tx-link",
        properties: &[
            ("display", "block"),
            ("height", "100%"),
            ("top", "9px"),
            ("position", "relative"),
        ],
        leptos_css: "display: block; height: 100%; top: 9px; position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: "@media (min-width: 576px) { .tx-link }",

        properties: &[("top", "11px")],
        leptos_css: "top: 11px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: "@media (min-width: 768px) { .tx-link }",

        properties: &[("top", "17px")],
        leptos_css: "top: 17px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: ".title-tx h2",
        properties: &[("line-height", "1"), ("margin-bottom", "10px")],
        leptos_css: "line-height: 1; margin-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: ".assetIcon",
        properties: &[("height", "150px"), ("margin", "25px")],
        leptos_css: "height: 150px; margin: 25px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: "@media (min-width: 768px) { .assetIcon }",

        properties: &[("height", "250px"), ("margin", "0")],
        leptos_css: "height: 250px; margin: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: ".icon-holder",
        properties: &[
            ("display", "flex"),
            ("justify-content", "center"),
            ("align-items", "center"),
        ],
        leptos_css: "display: flex; justify-content: center; align-items: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: ".defaultIcon",
        properties: &[("margin", "25px"), ("height", "150px")],
        leptos_css: "margin: 25px; height: 150px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: ".defaultIcon.skeleton",
        properties: &[("opacity", "0.5")],
        leptos_css: "opacity: 0.5;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/asset/asset.component.scss",
        selector_path: ".assetName",
        properties: &[("word-break", "break-word"), ("white-space", "normal")],
        leptos_css: "word-break: break-word; white-space: normal;",
        gpui: GpuiOutcome::Paint,
    },
];
