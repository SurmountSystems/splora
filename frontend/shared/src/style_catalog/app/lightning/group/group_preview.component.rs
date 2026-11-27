use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/group/group-preview.component.scss",
        selector_path: ".table",
        properties: &[("font-size", "32px"), ("margin-top", "0px")],
        leptos_css: "font-size: 32px; margin-top: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group-preview.component.scss",
        selector_path: ".logo-wrapper",
        properties: &[
            ("position", "relative"),
            ("width", "62px"),
            ("height", "62px"),
            ("margin-right", "1em"),
        ],
        leptos_css: "position: relative; width: 62px; height: 62px; margin-right: 1em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group-preview.component.scss",
        selector_path: ".logo-wrapper img",
        properties: &[("position", "absolute"), ("right", "0"), ("top", "0")],
        leptos_css: "position: absolute; right: 0; top: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group-preview.component.scss",
        selector_path: ".description-wrapper",
        properties: &[
            ("width", "100%"),
            ("margin", "16px 0 0"),
            ("padding", "20px 12px"),
            ("background", "var(--stat-box-bg)"),
            ("font-size", "32px"),
        ],
        leptos_css: "width: 100%; margin: 16px 0 0; padding: 20px 12px; background: var(--stat-box-bg); font-size: 32px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group-preview.component.scss",
        selector_path: ".description-text",
        properties: &[
            ("width", "100%"),
            ("line-height", "36px"),
            ("height", "72px"),
            ("max-height", "72px"),
            ("min-height", "72px"),
            ("overflow", "hidden"),
            ("display", "-webkit-box"),
            ("-webkit-box-orient", "vertical"),
            ("-webkit-line-clamp", "2"),
            ("text-overflow", "ellipsis"),
        ],
        leptos_css: "width: 100%; line-height: 36px; height: 72px; max-height: 72px; min-height: 72px; overflow: hidden; display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 2; text-overflow: ellipsis;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group-preview.component.scss",
        selector_path: ".map-col",
        properties: &[
            ("flex-grow", "0"),
            ("flex-shrink", "0"),
            ("width", "470px"),
            ("height", "272px"),
            ("min-width", "470px"),
            ("min-height", "272px"),
            ("max-height", "272px"),
            ("padding", "0"),
            ("background", "var(--stat-box-bg)"),
            ("overflow", "hidden"),
            ("margin-top", "16px"),
        ],
        leptos_css: "flex-grow: 0; flex-shrink: 0; width: 470px; height: 272px; min-width: 470px; min-height: 272px; max-height: 272px; padding: 0; background: var(--stat-box-bg); overflow: hidden; margin-top: 16px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group-preview.component.scss",
        selector_path: ".row",
        properties: &[("margin-right", "0")],
        leptos_css: "margin-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group-preview.component.scss",
        selector_path: ".full-width-row",
        properties: &[("padding-left", "15px"), ("flex-wrap", "nowrap")],
        leptos_css: "padding-left: 15px; flex-wrap: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group-preview.component.scss",
        selector_path: "::ng-deep .symbol",
        properties: &[("font-size", "24px")],
        leptos_css: "font-size: 24px;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
];
