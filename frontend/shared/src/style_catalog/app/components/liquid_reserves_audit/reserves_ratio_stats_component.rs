use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: ".fee-estimation-container",
        properties: &[("display", "flex"), ("justify-content", "space-between")],
        leptos_css: "display: flex; justify-content: space-between;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: "@media (min-width: 376px) { .fee-estimation-container }",

        properties: &[("flex-direction", "row")],
        leptos_css: "flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: ".fee-estimation-container .emergency-keys",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: "@media (min-width: 480px) and (max-width: 767px), (min-width: 915px) { .fee-estimation-container .emergency-keys }",

        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: ".fee-estimation-container .item",
        properties: &[
            ("max-width", "300px"),
            ("margin", "0"),
            ("width", "-webkit-fill-available"),
        ],
        leptos_css: "max-width: 300px; margin: 0; width: -webkit-fill-available;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: "@media (min-width: 376px) { .fee-estimation-container .item }",

        properties: &[("margin", "0 auto 0px")],
        leptos_css: "margin: 0 auto 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: ".fee-estimation-container .item .card-title",
        properties: &[
            ("margin-bottom", "4px"),
            ("color", "var(--title-fg)"),
            ("font-size", "10px"),
            ("font-size", "1rem"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "margin-bottom: 4px; color: var(--title-fg); font-size: 10px; font-size: 1rem; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: ".fee-estimation-container .item .card-text",
        properties: &[("font-size", "22px")],
        leptos_css: "font-size: 22px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
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
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
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
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: ".fee-estimation-container .item .fee-text",
        properties: &[
            ("width", "fit-content"),
            ("margin", "auto"),
            ("line-height", "1.45"),
            ("padding", "0px 2px"),
        ],
        leptos_css: "width: fit-content; margin: auto; line-height: 1.45; padding: 0px 2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: ".loading-container",
        properties: &[("min-height", "76px")],
        leptos_css: "min-height: 76px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/reserves-ratio-stats/reserves-ratio-stats.component.scss",
        selector_path: ".card-text .skeleton-loader",
        properties: &[
            ("width", "100%"),
            ("display", "block"),
            ("max-width", "90px"),
            ("margin", "15px auto 3px"),
        ],
        leptos_css: "width: 100%; display: block; max-width: 90px; margin: 15px auto 3px;",
        gpui: GpuiOutcome::Paint,
    },
];
