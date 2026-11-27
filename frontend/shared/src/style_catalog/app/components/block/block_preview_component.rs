use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: ".table",
        properties: &[("font-size", "32px"), ("margin-top", "36px")],
        leptos_css: "font-size: 32px; margin-top: 36px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: ".block-titles",
        properties: &[
            ("margin", "0"),
            ("padding-left", "15px"),
            ("padding-right", "15px"),
            ("display", "flex"),
            ("flex-direction", "row"),
            ("flex-wrap", "nowrap"),
            ("align-items", "center"),
            ("justify-content", "space-between"),
            ("width", "100%"),
            ("flex-grow", "1"),
        ],
        leptos_css: "margin: 0; padding-left: 15px; padding-right: 15px; display: flex; flex-direction: row; flex-wrap: nowrap; align-items: center; justify-content: space-between; width: 100%; flex-grow: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: ".block-titles .title",
        properties: &[
            ("flex-shrink", "0"),
            ("flex-grow", "0"),
            ("margin-right", "30px"),
            ("font-size", "64px"),
        ],
        leptos_css: "flex-shrink: 0; flex-grow: 0; margin-right: 30px; font-size: 64px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: ".block-titles .blockhash",
        properties: &[
            ("width", "0"),
            ("flex-grow", "1"),
            ("flex-shrink", "1"),
            (
                "font-family",
                "Consolas, Liberation Mono, Courier New, monospace",
            ),
            ("font-size", "32px"),
            ("text-align", "right"),
        ],
        leptos_css: "width: 0; flex-grow: 1; flex-shrink: 1; font-family: Consolas, Liberation Mono, Courier New, monospace; font-size: 32px; text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: ".block-titles .blockhash .truncate",
        properties: &[
            ("max-width", "100%"),
            ("overflow", "hidden"),
            ("white-space", "nowrap"),
            ("text-overflow", "ellipsis"),
            ("margin", "0"),
        ],
        leptos_css: "max-width: 100%; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; margin: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: ".block-titles .blockhash .truncate.left",
        properties: &[("direction", "rtl")],
        leptos_css: "direction: rtl;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: ".table-col",
        properties: &[("max-width", "calc(100% - 470px)"), ("overflow", "hidden")],
        leptos_css: "max-width: calc(100% - 470px); overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: ".chart-container",
        properties: &[
            ("flex-grow", "0"),
            ("flex-shrink", "0"),
            ("width", "480px"),
            ("min-width", "480px"),
            ("padding", "0"),
            ("margin-right", "15px"),
        ],
        leptos_css: "flex-grow: 0; flex-shrink: 0; width: 480px; min-width: 480px; padding: 0; margin-right: 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: "::ng-deep .symbol",
        properties: &[("font-size", "24px")],
        leptos_css: "font-size: 24px;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: ".badge",
        properties: &[("transition", "none")],
        leptos_css: "transition: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block/block-preview.component.scss",
        selector_path: ".pool-logo",
        properties: &[
            ("width", "25px"),
            ("height", "25px"),
            ("position", "relative"),
            ("top", "-1px"),
            ("margin-right", "2px"),
        ],
        leptos_css: "width: 25px; height: 25px; position: relative; top: -1px; margin-right: 2px;",
        gpui: GpuiOutcome::Paint,
    },
];
