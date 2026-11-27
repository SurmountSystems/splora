use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/push-transaction/push-transaction.component.scss",
        selector_path: ".accept-results td.allowed, .accept-results th.allowed",
        properties: &[("width", "10%"), ("text-align", "center")],
        leptos_css: "width: 10%; text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/push-transaction/push-transaction.component.scss",
        selector_path: ".accept-results td.txid, .accept-results th.txid",
        properties: &[("width", "50%")],
        leptos_css: "width: 50%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/push-transaction/push-transaction.component.scss",
        selector_path: ".accept-results td.rate, .accept-results th.rate",
        properties: &[
            ("width", "20%"),
            ("text-align", "right"),
            ("white-space", "wrap"),
        ],
        leptos_css: "width: 20%; text-align: right; white-space: wrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/push-transaction/push-transaction.component.scss",
        selector_path: ".accept-results td.reason, .accept-results th.reason",
        properties: &[
            ("width", "20%"),
            ("text-align", "right"),
            ("white-space", "wrap"),
        ],
        leptos_css: "width: 20%; text-align: right; white-space: wrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/push-transaction/push-transaction.component.scss",
        selector_path: "@media (max-width: 950px) { .accept-results }",
        properties: &[("table-layout", "auto")],
        leptos_css: "table-layout: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/push-transaction/push-transaction.component.scss",
        selector_path: "@media (max-width: 950px) { .accept-results td.allowed, .accept-results th.allowed }",
        properties: &[("width", "100px")],
        leptos_css: "width: 100px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/push-transaction/push-transaction.component.scss",
        selector_path: "@media (max-width: 950px) { .accept-results td.txid, .accept-results th.txid }",
        properties: &[("max-width", "200px")],
        leptos_css: "max-width: 200px;",
        gpui: GpuiOutcome::Paint,
    },
];
