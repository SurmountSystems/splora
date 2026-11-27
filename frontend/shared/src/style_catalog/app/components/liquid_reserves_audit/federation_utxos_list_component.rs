use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".spinner-border",
        properties: &[
            ("height", "25px"),
            ("width", "25px"),
            ("margin-top", "13px"),
        ],
        leptos_css: "height: 25px; width: 25px; margin-top: 13px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "tr, td, th",
        properties: &[
            ("border", "0px"),
            ("padding-top", "0.65rem !important"),
            ("padding-bottom", "0.6rem !important"),
            ("padding-right", "2rem !important"),
        ],
        leptos_css: "border: 0px; padding-top: 0.65rem !important; padding-bottom: 0.6rem !important; padding-right: 2rem !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "tr .widget, td .widget, th .widget",
        properties: &[("padding-right", "1rem !important")],
        leptos_css: "padding-right: 1rem !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".clear-link",
        properties: &[("color", "white")],
        leptos_css: "color: white;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".disabled",
        properties: &[("pointer-events", "none"), ("opacity", "0.5")],
        leptos_css: "pointer-events: none; opacity: 0.5;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".progress",
        properties: &[("background-color", "var(--secondary)")],
        leptos_css: "background-color: var(--secondary);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".txid",
        properties: &[
            ("width", "25%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "160px"),
        ],
        leptos_css: "width: 25%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 160px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".txid.widget",
        properties: &[("width", "40%")],
        leptos_css: "width: 40%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".address",
        properties: &[
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "160px"),
        ],
        leptos_css: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 160px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "@media (max-width: 527px) { .address }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".amount",
        properties: &[("width", "12%")],
        leptos_css: "width: 12%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".amount.widget",
        properties: &[("width", "30%")],
        leptos_css: "width: 30%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".pegin",
        properties: &[
            ("width", "25%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "160px"),
        ],
        leptos_css: "width: 25%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 160px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "@media (max-width: 872px) { .pegin }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".timestamp",
        properties: &[("width", "18%")],
        leptos_css: "width: 18%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "@media (max-width: 800px) { .timestamp }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "@media (max-width: 1190px) { .timestamp .relative-time }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".timestamp.widget",
        properties: &[("width", "100%")],
        leptos_css: "width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "@media (min-width: 768px) and (max-width: 1050px) { .timestamp.widget }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "@media (max-width: 767px) { .timestamp.widget }",

        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "@media (max-width: 500px) { .timestamp.widget }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "@media (max-width: 987px) { .expires-in }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "@media (max-width: 1090px) { .is-dust }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".fee-estimation-container",
        properties: &[("display", "flex"), ("margin-bottom", "10px")],
        leptos_css: "display: flex; margin-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".fee-estimation-container .item",
        properties: &[("max-width", "300px"), ("margin", "0")],
        leptos_css: "max-width: 300px; margin: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: "@media (min-width: 376px) { .fee-estimation-container .item }",

        properties: &[("margin", "0 auto 0px")],
        leptos_css: "margin: 0 auto 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".fee-estimation-container .item .card-text",
        properties: &[("font-size", "22px")],
        leptos_css: "font-size: 22px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".fee-estimation-container .item .card-text span",
        properties: &[
            ("font-size", "11px"),
            ("position", "relative"),
            ("top", "-2px"),
        ],
        leptos_css: "font-size: 11px; position: relative; top: -2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".fee-estimation-container .item .card-text .skeleton-loader",
        properties: &[
            ("width", "100%"),
            ("display", "block"),
            ("min-width", "90px"),
            ("margin", "15px auto 3px"),
        ],
        leptos_css: "width: 100%; display: block; min-width: 90px; margin: 15px auto 3px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".fee-estimation-container .item .card-text span",
        properties: &[
            ("color", "var(--transparent-fg)"),
            ("font-size", "12px"),
            ("top", "0px"),
        ],
        leptos_css: "color: var(--transparent-fg); font-size: 12px; top: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/federation-utxos-list/federation-utxos-list.component.scss",
        selector_path: ".fee-estimation-container .item .fee-text",
        properties: &[
            ("width", "fit-content"),
            ("margin", "auto"),
            ("line-height", "1.45"),
        ],
        leptos_css: "width: fit-content; margin: auto; line-height: 1.45;",
        gpui: GpuiOutcome::Paint,
    },
];
