use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".qr-wrapper",
        properties: &[
            ("background-color", "#FFF"),
            ("padding", "10px"),
            ("padding-bottom", "5px"),
            ("display", "inline-block"),
        ],
        leptos_css: "background-color: #FFF; padding: 10px; padding-bottom: 5px; display: inline-block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".treemap-col",
        properties: &[
            ("width", "100%"),
            ("height", "300px"),
            ("margin-top", "1em"),
        ],
        leptos_css: "width: 100%; height: 300px; margin-top: 1em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (min-width: 768px) { .treemap-col }",

        properties: &[("width", "45%"), ("margin-top", "0px")],
        leptos_css: "width: 45%; margin-top: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".fiat",
        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (min-width: 992px) { .fiat }",

        properties: &[("display", "inline-block"), ("margin-left", "10px")],
        leptos_css: "display: inline-block; margin-left: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".table tr td:last-child",
        properties: &[("text-align", "right")],
        leptos_css: "text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (min-width: 576px) { .table tr td:last-child }",

        properties: &[("text-align", "left")],
        leptos_css: "text-align: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".address-list",
        properties: &[("width", "100%"), ("max-width", "200px")],
        leptos_css: "width: 100%; max-width: 200px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "h1",
        properties: &[
            ("margin", "0px"),
            ("padding", "0px"),
            ("margin-right", "10px"),
            ("font-size", "1.9rem"),
        ],
        leptos_css: "margin: 0px; padding: 0px; margin-right: 10px; font-size: 1.9rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (min-width: 576px) { h1 }",

        properties: &[("font-size", "2rem"), ("float", "left")],
        leptos_css: "font-size: 2rem; float: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (min-width: 768px) { h1 }",

        properties: &[("font-size", "2.5rem")],
        leptos_css: "font-size: 2.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".title-address",
        properties: &[("align-items", "baseline")],
        leptos_css: "align-items: baseline;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".address-link",
        properties: &[
            ("line-height", "56px"),
            ("margin-left", "0px"),
            ("top", "-2px"),
            ("position", "relative"),
        ],
        leptos_css: "line-height: 56px; margin-left: 0px; top: -2px; position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (min-width: 768px) { .address-link }",

        properties: &[("line-height", "69px")],
        leptos_css: "line-height: 69px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".row",
        properties: &[("flex-direction", "column")],
        leptos_css: "flex-direction: column;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (min-width: 576px) { .row }",

        properties: &[("flex-direction", "row")],
        leptos_css: "flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (max-width: 767.98px) { .mobile-bottomcol }",

        properties: &[("margin-top", "15px")],
        leptos_css: "margin-top: 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (max-width: 767.98px) { .details-table td:first-child }",

        properties: &[("white-space", "pre-wrap")],
        leptos_css: "white-space: pre-wrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
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
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (min-width: 576px) { .tx-link }",

        properties: &[("top", "11px")],
        leptos_css: "top: 11px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: "@media (min-width: 768px) { .tx-link }",

        properties: &[("max-width", "calc(100% - 180px)"), ("top", "17px")],
        leptos_css: "max-width: calc(100% - 180px); top: 17px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".title-tx h2",
        properties: &[("line-height", "1"), ("margin-bottom", "10px")],
        leptos_css: "line-height: 1; margin-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".liquid-address .address-table",
        properties: &[("table-layout", "fixed")],
        leptos_css: "table-layout: fixed;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".liquid-address .address-table tr td:first-child",
        properties: &[("width", "170px")],
        leptos_css: "width: 170px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".liquid-address .address-table tr td:last-child",
        properties: &[("width", "80%")],
        leptos_css: "width: 80%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/wallet/wallet.component.scss",
        selector_path: ".liquid-address .qrcode-col",
        properties: &[("flex-grow", "0.5")],
        leptos_css: "flex-grow: 0.5;",
        gpui: GpuiOutcome::Paint,
    },
];
