use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".qr-wrapper",
        properties: &[
            ("position", "absolute"),
            ("top", "30px"),
            ("right", "0px"),
            ("border", "solid 10px var(--secondary)"),
            ("border-radius", "5px"),
            ("background-color", "#fff"),
            ("padding", "10px"),
            ("padding-bottom", "5px"),
            ("display", "block"),
            ("z-index", "99"),
        ],
        leptos_css: "position: absolute; top: 30px; right: 0px; border: solid 10px var(--secondary); border-radius: 5px; background-color: #fff; padding: 10px; padding-bottom: 5px; display: block; z-index: 99;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".qrSpan",
        properties: &[
            ("position", "relative"),
            ("cursor", "pointer"),
            ("padding-left", "0.4rem"),
        ],
        leptos_css: "position: relative; cursor: pointer; padding-left: 0.4rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".fiat",
        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: "@media (min-width: 992px) { .fiat }",

        properties: &[("display", "inline-block"), ("margin-left", "10px")],
        leptos_css: "display: inline-block; margin-left: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".table tr td:last-child",
        properties: &[("text-align", "right")],
        leptos_css: "text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: "@media (min-width: 768px) { .table tr td:last-child }",

        properties: &[("text-align", "left")],
        leptos_css: "text-align: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".table tr td.wrap-cell",
        properties: &[("white-space", "normal")],
        leptos_css: "white-space: normal;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
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
        source_path: "app/components/address/address.component.scss",
        selector_path: "@media (min-width: 576px) { h1 }",

        properties: &[("font-size", "2rem"), ("float", "left")],
        leptos_css: "font-size: 2rem; float: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: "@media (min-width: 768px) { h1 }",

        properties: &[("font-size", "2.5rem")],
        leptos_css: "font-size: 2.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
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
        source_path: "app/components/address/address.component.scss",
        selector_path: "@media (min-width: 768px) { .address-link }",

        properties: &[("line-height", "69px")],
        leptos_css: "line-height: 69px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".row",
        properties: &[("flex-direction", "column")],
        leptos_css: "flex-direction: column;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: "@media (min-width: 576px) { .row }",

        properties: &[("flex-direction", "row")],
        leptos_css: "flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: "@media (max-width: 767.98px) { .mobile-bottomcol }",

        properties: &[("margin-top", "15px")],
        leptos_css: "margin-top: 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: "@media (max-width: 767.98px) { .details-table td:first-child }",

        properties: &[("white-space", "pre-wrap")],
        leptos_css: "white-space: pre-wrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
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
        source_path: "app/components/address/address.component.scss",
        selector_path: "@media (min-width: 576px) { .tx-link }",

        properties: &[("max-width", "calc(100% - 180px)"), ("top", "11px")],
        leptos_css: "max-width: calc(100% - 180px); top: 11px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: "@media (min-width: 768px) { .tx-link }",

        properties: &[("top", "17px")],
        leptos_css: "top: 17px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".title-tx h2",
        properties: &[("line-height", "1"), ("margin-bottom", "10px")],
        leptos_css: "line-height: 1; margin-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".title-with-toggle",
        properties: &[
            ("display", "flex"),
            ("justify-content", "space-between"),
            ("align-items", "baseline"),
            ("gap", "0.5rem"),
        ],
        leptos_css: "display: flex; justify-content: space-between; align-items: baseline; gap: 0.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".title-with-toggle .btn",
        properties: &[("flex-shrink", "0"), ("margin-bottom", "10px")],
        leptos_css: "flex-shrink: 0; margin-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".liquid-address .address-table",
        properties: &[("table-layout", "fixed")],
        leptos_css: "table-layout: fixed;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".widget-toggler",
        properties: &[
            ("font-size", "12px"),
            ("position", "absolute"),
            ("top", "-20px"),
            ("inset-inline-end", "3px"),
            ("text-align", "right"),
        ],
        leptos_css: "font-size: 12px; position: absolute; top: -20px; inset-inline-end: 3px; text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".toggler-option",
        properties: &[("text-decoration", "none")],
        leptos_css: "text-decoration: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".inactive",
        properties: &[("color", "var(--transparent-fg)")],
        leptos_css: "color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".taproot-psbt-btn:focus:not(:focus-visible)",
        properties: &[("outline", "none"), ("box-shadow", "none")],
        leptos_css: "outline: none; box-shadow: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address/address.component.scss",
        selector_path: ".taproot-psbt-btn",
        properties: &[
            ("padding-top", "0"),
            ("padding-bottom", "0"),
            ("font-size", "0.8rem"),
        ],
        leptos_css: "padding-top: 0; padding-bottom: 0; font-size: 0.8rem;",
        gpui: GpuiOutcome::Paint,
    },
];
