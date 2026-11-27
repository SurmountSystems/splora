use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/acceleration-timeline/acceleration-timeline-tooltip.component.scss",
        selector_path: ".acceleration-tooltip",
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
        ],
        leptos_css: "position: fixed; z-index: 3; background: color-mix(in srgb, #11131f 95%, transparent); border-radius: 4px; box-shadow: 1px 1px 10px rgba(0, 0, 0, 0.5); color: var(--tooltip-grey); display: flex; flex-direction: column; justify-content: space-between; padding: 10px 15px; text-align: left; pointer-events: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration-timeline/acceleration-timeline-tooltip.component.scss",
        selector_path: ".acceleration-tooltip .badge.badge-accelerated",
        properties: &[("background-color", "var(--tertiary)"), ("color", "white")],
        leptos_css: "background-color: var(--tertiary); color: white;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration-timeline/acceleration-timeline-tooltip.component.scss",
        selector_path: ".acceleration-tooltip .value",
        properties: &[("text-align", "end")],
        leptos_css: "text-align: end;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration-timeline/acceleration-timeline-tooltip.component.scss",
        selector_path: ".acceleration-tooltip .label",
        properties: &[("padding-right", "30px"), ("vertical-align", "top")],
        leptos_css: "padding-right: 30px; vertical-align: top;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration-timeline/acceleration-timeline-tooltip.component.scss",
        selector_path: ".acceleration-tooltip .oobFees",
        properties: &[("color", "#905cf4")],
        leptos_css: "color: #905cf4;",
        gpui: GpuiOutcome::Paint,
    },
];
