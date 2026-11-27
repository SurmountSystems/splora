use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/difficulty-adjustments-table/difficulty-adjustments-table.component.scss",
        selector_path: ".latest-adjustments",
        properties: &[
            ("width", "100%"),
            ("text-align", "left"),
            ("table-layout", "fixed"),
        ],
        leptos_css: "width: 100%; text-align: left; table-layout: fixed;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/difficulty-adjustments-table/difficulty-adjustments-table.component.scss",
        selector_path: ".latest-adjustments tr, .latest-adjustments th",
        properties: &[
            ("border", "0px"),
            ("padding-top", "0.65rem !important"),
            ("padding-bottom", "0.7rem !important"),
        ],
        leptos_css: "border: 0px; padding-top: 0.65rem !important; padding-bottom: 0.7rem !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/difficulty-adjustments-table/difficulty-adjustments-table.component.scss",
        selector_path: ".latest-adjustments td",
        properties: &[
            ("border", "0px"),
            ("padding-top", "0.71rem !important"),
            ("padding-bottom", "0.75rem !important"),
            ("width", "25%"),
        ],
        leptos_css: "border: 0px; padding-top: 0.71rem !important; padding-bottom: 0.75rem !important; width: 25%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/difficulty-adjustments-table/difficulty-adjustments-table.component.scss",
        selector_path: "@media (max-width: 376px) { .latest-adjustments td }",

        properties: &[("padding", "0.85rem")],
        leptos_css: "padding: 0.85rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/difficulty-adjustments-table/difficulty-adjustments-table.component.scss",
        selector_path: "@media (min-width: 767px) and (max-width: 991px) { .date }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/difficulty-adjustments-table/difficulty-adjustments-table.component.scss",
        selector_path: "@media (max-width: 500px) { .date }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
];
