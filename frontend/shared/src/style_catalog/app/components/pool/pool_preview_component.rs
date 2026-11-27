use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: ".stats",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "row"),
            ("justify-content", "center"),
            ("align-items", "flex-start"),
            ("width", "100%"),
            ("max-width", "100%"),
            ("margin", "15px 0"),
            ("font-size", "32px"),
            ("overflow", "hidden"),
        ],
        leptos_css: "display: flex; flex-direction: row; justify-content: center; align-items: flex-start; width: 100%; max-width: 100%; margin: 15px 0; font-size: 32px; overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: ".stats .stat-box",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "row"),
            ("flex-wrap", "nowrap"),
            ("align-items", "baseline"),
            ("justify-content", "space-between"),
            ("width", "100%"),
            ("margin-left", "15px"),
            ("background", "var(--stat-box-bg)"),
            ("padding", "0.75rem"),
            ("width", "0"),
            ("flex-grow", "1"),
        ],
        leptos_css: "display: flex; flex-direction: row; flex-wrap: nowrap; align-items: baseline; justify-content: space-between; width: 100%; margin-left: 15px; background: var(--stat-box-bg); padding: 0.75rem; width: 0; flex-grow: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: ".stats .stat-box:first-child",
        properties: &[("margin-left", "0")],
        leptos_css: "margin-left: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: ".stats .stat-box .label",
        properties: &[
            ("flex-shrink", "0"),
            ("flex-grow", "0"),
            ("margin-right", "1em"),
        ],
        leptos_css: "flex-shrink: 0; flex-grow: 0; margin-right: 1em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: ".stats .stat-box .data",
        properties: &[
            ("flex-shrink", "1"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "flex-shrink: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: ".chart",
        properties: &[
            ("width", "100%"),
            ("height", "315px"),
            ("background", "var(--stat-box-bg)"),
        ],
        leptos_css: "width: 100%; height: 315px; background: var(--stat-box-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: ".row",
        properties: &[("margin-right", "0")],
        leptos_css: "margin-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: ".full-width-row",
        properties: &[("padding-left", "15px"), ("flex-wrap", "nowrap")],
        leptos_css: "padding-left: 15px; flex-wrap: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
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
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: ".logo-wrapper img",
        properties: &[
            ("position", "absolute"),
            ("right", "0"),
            ("top", "0"),
            ("background", "var(--box-bg)"),
        ],
        leptos_css: "position: absolute; right: 0; top: 0; background: var(--box-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: ".logo-wrapper img.noimg",
        properties: &[("opacity", "0")],
        leptos_css: "opacity: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/pool/pool-preview.component.scss",
        selector_path: "::ng-deep .symbol",
        properties: &[("font-size", "24px")],
        leptos_css: "font-size: 24px;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
];
