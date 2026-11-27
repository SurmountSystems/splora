use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/rbf-timeline/rbf-timeline-tooltip.component.scss",
        selector_path: ".rbf-tooltip",
        properties: &[
            ("position", "fixed"),
            ("z-index", "3"),
            ("background", "color-mix(in srgb, #11131f 95%, transparent)"),
            ("border-radius", "4px"),
            ("box-shadow", "1px 1px 10px rgba(0, 0, 0, 0.5)"),
            ("color", "var(--tooltip-grey)"),
            ("display", "flex"),
            ("flex-direction", "column"),
            ("justify-content", "space-between"),
            ("padding", "10px 15px"),
            ("text-align", "left"),
            ("pointer-events", "none"),
            ("--link-color", "#1bd8f4"),
            ("--link-hover-color", "#09a3ba"),
        ],
        leptos_css: "position: fixed; z-index: 3; background: color-mix(in srgb, #11131f 95%, transparent); border-radius: 4px; box-shadow: 1px 1px 10px rgba(0, 0, 0, 0.5); color: var(--tooltip-grey); display: flex; flex-direction: column; justify-content: space-between; padding: 10px 15px; text-align: left; pointer-events: none; --link-color: #1bd8f4; --link-hover-color: #09a3ba;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/rbf-timeline/rbf-timeline-tooltip.component.scss",
        selector_path: ".rbf-tooltip .badge",
        properties: &[("margin-right", "1em")],
        leptos_css: "margin-right: 1em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/rbf-timeline/rbf-timeline-tooltip.component.scss",
        selector_path: ".rbf-tooltip .badge:last-child",
        properties: &[("margin-right", "0")],
        leptos_css: "margin-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/rbf-timeline/rbf-timeline-tooltip.component.scss",
        selector_path: ".rbf-tooltip ::ng-deep .symbol",
        properties: &[("color", "rgba(255, 255, 255, 0.4)")],
        leptos_css: "color: rgba(255, 255, 255, 0.4);",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/rbf-timeline/rbf-timeline-tooltip.component.scss",
        selector_path: ".td-width",
        properties: &[("padding-right", "10px")],
        leptos_css: "padding-right: 10px;",
        gpui: GpuiOutcome::Paint,
    },
];
