use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: ".td-width",
        properties: &[("width", "150px"), ("min-width", "150px")],
        leptos_css: "width: 150px; min-width: 150px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: "@media (max-width: 768px) { .td-width }",

        properties: &[("width", "175px"), ("min-width", "175px")],
        leptos_css: "width: 175px; min-width: 175px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: "@media (max-width: 849px) { .field-label }",

        properties: &[("text-align", "left")],
        leptos_css: "text-align: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: "@media (max-width: 649px) { .field-label }",

        properties: &[("width", "auto"), ("min-width", "auto")],
        leptos_css: "width: auto; min-width: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: ".field-label.chart-left",
        properties: &[("width", "100%")],
        leptos_css: "width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: "@media (max-width: 849px) { .field-value }",

        properties: &[("width", "100%")],
        leptos_css: "width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: ".field-value.chart-left",
        properties: &[("width", "auto")],
        leptos_css: "width: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: "@media (max-width: 420px) { .field-value .hashrate-label }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: ".pie-chart",
        properties: &[
            ("width", "100%"),
            ("vertical-align", "middle"),
            ("text-align", "center"),
        ],
        leptos_css: "width: 100%; vertical-align: middle; text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: ".pie-chart .chart-container",
        properties: &[
            ("width", "72px"),
            ("height", "100%"),
            ("margin-left", "auto"),
        ],
        leptos_css: "width: 72px; height: 100%; margin-left: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: "@media (max-width: 850px) { .pie-chart }",

        properties: &[("width", "150px")],
        leptos_css: "width: 150px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: "@media (max-width: 420px) { .pie-chart }",

        properties: &[("padding-left", "0")],
        leptos_css: "padding-left: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: "::ng-deep .chart",
        properties: &[("overflow", "visible")],
        leptos_css: "overflow: visible;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: "::ng-deep .chart > div, ::ng-deep .chart > div > svg",
        properties: &[("overflow", "visible !important")],
        leptos_css: "overflow: visible !important;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/acceleration/active-acceleration-box/active-acceleration-box.component.scss",
        selector_path: ".oobFees",
        properties: &[("color", "#905cf4")],
        leptos_css: "color: #905cf4;",
        gpui: GpuiOutcome::Paint,
    },
];
