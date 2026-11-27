use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: ".latest-transactions",
        properties: &[
            ("width", "100%"),
            ("text-align", "left"),
            ("table-layout", "fixed"),
        ],
        leptos_css: "width: 100%; text-align: left; table-layout: fixed;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: ".latest-transactions tr, .latest-transactions td, .latest-transactions th",
        properties: &[
            ("border", "0px"),
            ("padding-top", "0.71rem !important"),
            ("padding-bottom", "0.75rem !important"),
        ],
        leptos_css: "border: 0px; padding-top: 0.71rem !important; padding-bottom: 0.75rem !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: ".latest-transactions td",
        properties: &[("overflow", "hidden"), ("width", "25%")],
        leptos_css: "overflow: hidden; width: 25%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: ".latest-transactions .table-cell-txid",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: "@media (min-width: 420px) and (max-width: 767px), (min-width: 900px) { .latest-transactions .table-cell-txid }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: ".latest-transactions .table-cell-satoshis-desktop",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: "@media (min-width: 420px) and (max-width: 767px), (min-width: 900px) { .latest-transactions .table-cell-satoshis-desktop }",

        properties: &[("display", "table-cell"), ("text-align", "right")],
        leptos_css: "display: table-cell; text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: ".latest-transactions .table-cell-satoshis-mobile",
        properties: &[("display", "table-cell"), ("text-align", "right")],
        leptos_css: "display: table-cell; text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: "@media (min-width: 420px) and (max-width: 767px), (min-width: 900px) { .latest-transactions .table-cell-satoshis-mobile }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: ".latest-transactions .table-cell-fiat",
        properties: &[("display", "none"), ("text-align", "right")],
        leptos_css: "display: none; text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: "@media (min-width: 650px) and (max-width: 767px), (min-width: 1100px) { .latest-transactions .table-cell-fiat }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: ".latest-transactions .table-cell-date",
        properties: &[("text-align", "left")],
        leptos_css: "text-align: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: "@media (min-width: 420px) and (max-width: 767px), (min-width: 900px) { .latest-transactions .table-cell-date }",

        properties: &[("text-align", "right")],
        leptos_css: "text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: ".latest-transactions .mobile-tx-link",
        properties: &[("display", "inline")],
        leptos_css: "display: inline;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: "@media (min-width: 420px) and (max-width: 767px), (min-width: 900px) { .latest-transactions .mobile-tx-link }",

        properties: &[("color", "inherit"), ("pointer-events", "none")],
        leptos_css: "color: inherit; pointer-events: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-transactions-widget/address-transactions-widget.component.scss",
        selector_path: ".skeleton-loader-transactions",
        properties: &[
            ("max-width", "250px"),
            ("position", "relative"),
            ("top", "2px"),
            ("margin-bottom", "-3px"),
            ("height", "18px"),
        ],
        leptos_css: "max-width: 250px; position: relative; top: 2px; margin-bottom: -3px; height: 18px;",
        gpui: GpuiOutcome::Paint,
    },
];
