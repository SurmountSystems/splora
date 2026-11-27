use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: ".block-overview-tooltip",
        properties: &[
            ("position", "absolute"),
            ("background", "color-mix(in srgb, #11131f 95%, transparent)"),
            ("border-radius", "4px"),
            ("box-shadow", "1px 1px 10px rgba(0, 0, 0, 0.5)"),
            ("color", "var(--tooltip-grey)"),
            ("display", "flex"),
            ("flex-direction", "column"),
            ("justify-content", "space-between"),
            ("padding", "10px 15px"),
            ("text-align", "left"),
            ("min-width", "340px"),
            ("max-width", "400px"),
            ("pointer-events", "none"),
            ("z-index", "11"),
            ("--link-color", "#1bd8f4"),
            ("--link-hover-color", "#09a3ba"),
        ],
        leptos_css: "position: absolute; background: color-mix(in srgb, #11131f 95%, transparent); border-radius: 4px; box-shadow: 1px 1px 10px rgba(0, 0, 0, 0.5); color: var(--tooltip-grey); display: flex; flex-direction: column; justify-content: space-between; padding: 10px 15px; text-align: left; min-width: 340px; max-width: 400px; pointer-events: none; z-index: 11; --link-color: #1bd8f4; --link-hover-color: #09a3ba;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: ".block-overview-tooltip.clickable",
        properties: &[("pointer-events", "all")],
        leptos_css: "pointer-events: all;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: ".block-overview-tooltip ::ng-deep .fixed-unit-color .symbol",
        properties: &[("color", "rgba(255, 255, 255, 0.4)")],
        leptos_css: "color: rgba(255, 255, 255, 0.4);",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: "th.label, td.label",
        properties: &[("width", "30%")],
        leptos_css: "width: 30%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: "th.value, td.value",
        properties: &[("width", "70%"), ("text-align", "end")],
        leptos_css: "width: 70%; text-align: end;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: "th.oobFees, td.oobFees",
        properties: &[("color", "#905cf4")],
        leptos_css: "color: #905cf4;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: ".badge.badge-accelerated",
        properties: &[
            ("background-color", "#653b9c"),
            (
                "box-shadow",
                "rgba(173, 125, 229, 0.4980392157) 0px 0px 12px -2px",
            ),
            ("color", "white"),
            ("animation", "acceleratePulse 1s infinite"),
        ],
        leptos_css: "background-color: #653b9c; box-shadow: rgba(173, 125, 229, 0.4980392157) 0px 0px 12px -2px; color: white; animation: acceleratePulse 1s infinite;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: ".tags",
        properties: &[
            ("display", "flex"),
            ("flex-wrap", "wrap"),
            ("row-gap", "0.25em"),
            ("margin-top", "0.2em"),
            ("max-width", "310px"),
        ],
        leptos_css: "display: flex; flex-wrap: wrap; row-gap: 0.25em; margin-top: 0.2em; max-width: 310px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: ".tags .badge",
        properties: &[
            ("border-radius", "0.2rem"),
            ("padding", "0.2em 0.5em"),
            ("margin-right", "0.25em"),
        ],
        leptos_css: "border-radius: 0.2rem; padding: 0.2em 0.5em; margin-right: 0.25em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: ".tags .filter-tag",
        properties: &[
            ("background", "rgba(24, 27, 45, 0.6862745098)"),
            ("border", "solid 1px var(--primary)"),
            ("color", "white"),
            ("transition", "background-color 300ms"),
        ],
        leptos_css: "background: rgba(24, 27, 45, 0.6862745098); border: solid 1px var(--primary); color: white; transition: background-color 300ms;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: ".tags .filter-tag.matching",
        properties: &[("background-color", "var(--primary)")],
        leptos_css: "background-color: var(--primary);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: ".tags.any-mode .filter-tag",
        properties: &[("border", "solid 1px var(--success)")],
        leptos_css: "border: solid 1px var(--success);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: ".tags.any-mode .filter-tag.matching",
        properties: &[("background-color", "var(--success)")],
        leptos_css: "background-color: var(--success);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: "@keyframes acceleratePulse 0%",
        properties: &[
            ("background-color", "#653b9c"),
            (
                "box-shadow",
                "rgba(173, 125, 229, 0.4980392157) 0px 0px 12px -2px",
            ),
        ],
        leptos_css: "background-color: #653b9c; box-shadow: rgba(173, 125, 229, 0.4980392157) 0px 0px 12px -2px;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: "@keyframes acceleratePulse 50%",
        properties: &[
            ("background-color", "#8457bb"),
            ("box-shadow", "#ad7de5 0px 0px 18px -2px"),
        ],
        leptos_css: "background-color: #8457bb; box-shadow: #ad7de5 0px 0px 18px -2px;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/block-overview-tooltip/block-overview-tooltip.component.scss",
        selector_path: "@keyframes acceleratePulse 100%",
        properties: &[
            ("background-color", "#653b9c"),
            (
                "box-shadow",
                "rgba(173, 125, 229, 0.4980392157) 0px 0px 12px -2px",
            ),
        ],
        leptos_css: "background-color: #653b9c; box-shadow: rgba(173, 125, 229, 0.4980392157) 0px 0px 12px -2px;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
];
