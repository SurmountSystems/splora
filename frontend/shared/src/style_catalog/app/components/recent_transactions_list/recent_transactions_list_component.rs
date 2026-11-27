use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".new-tx-pill",
        properties: &[
            ("position", "fixed"),
            ("top", "80px"),
            ("left", "50%"),
            ("transform", "translateX(-50%)"),
            ("z-index", "1000"),
            ("display", "inline-flex"),
            ("align-items", "center"),
            ("gap", "8px"),
            ("padding", "8px 18px"),
            ("border", "none"),
            ("border-radius", "999px"),
            ("background-color", "var(--primary)"),
            ("color", "#fff"),
            ("font-size", "13px"),
            ("font-weight", "500"),
            ("cursor", "pointer"),
            ("box-shadow", "0 4px 14px rgba(0, 0, 0, 0.35)"),
        ],
        leptos_css: "position: fixed; top: 80px; left: 50%; transform: translateX(-50%); z-index: 1000; display: inline-flex; align-items: center; gap: 8px; padding: 8px 18px; border: none; border-radius: 999px; background-color: var(--primary); color: #fff; font-size: 13px; font-weight: 500; cursor: pointer; box-shadow: 0 4px 14px rgba(0, 0, 0, 0.35);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".new-tx-pill:hover",
        properties: &[("filter", "brightness(1.1)")],
        leptos_css: "filter: brightness(1.1);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".title-row",
        properties: &[
            ("display", "flex"),
            ("flex-wrap", "wrap"),
            ("justify-content", "space-between"),
            ("align-items", "center"),
        ],
        leptos_css: "display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".title-row h1",
        properties: &[("margin-bottom", "0.5rem")],
        leptos_css: "margin-bottom: 0.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".controls",
        properties: &[
            ("display", "flex"),
            ("align-items", "center"),
            ("gap", "10px"),
            ("margin-bottom", "0.5rem"),
        ],
        leptos_css: "display: flex; align-items: center; gap: 10px; margin-bottom: 0.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".controls .btn-group .btn",
        properties: &[("font-size", "12px")],
        leptos_css: "font-size: 12px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".controls .btn-pause",
        properties: &[("font-size", "12px")],
        leptos_css: "font-size: 12px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".controls .btn-pause.active",
        properties: &[("opacity", "0.5")],
        leptos_css: "opacity: 0.5;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
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
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
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
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".latest-transactions td",
        properties: &[("overflow", "hidden"), ("width", "25%")],
        leptos_css: "overflow: hidden; width: 25%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".latest-transactions .table-cell-satoshis",
        properties: &[("display", "none"), ("text-align", "right")],
        leptos_css: "display: none; text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: "@media (min-width: 576px) { .latest-transactions .table-cell-satoshis }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".latest-transactions .table-cell-fiat",
        properties: &[("display", "none"), ("text-align", "right")],
        leptos_css: "display: none; text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: "@media (min-width: 485px) { .latest-transactions .table-cell-fiat }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".latest-transactions .table-cell-vsize",
        properties: &[("display", "none"), ("text-align", "right")],
        leptos_css: "display: none; text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: "@media (min-width: 576px) { .latest-transactions .table-cell-vsize }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".latest-transactions .table-cell-fees",
        properties: &[("text-align", "right")],
        leptos_css: "text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: ".widget .latest-transactions",
        properties: &[("overflow-anchor", "none")],
        leptos_css: "overflow-anchor: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: "@media (min-width: 768px) { .widget .latest-transactions .table-cell-satoshis }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: "@media (min-width: 1100px) { .widget .latest-transactions .table-cell-satoshis }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: "@media (min-width: 768px) { .widget .latest-transactions .table-cell-fiat }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/recent-transactions-list/recent-transactions-list.component.scss",
        selector_path: "@media (min-width: 992px) { .widget .latest-transactions .table-cell-fiat }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
];
