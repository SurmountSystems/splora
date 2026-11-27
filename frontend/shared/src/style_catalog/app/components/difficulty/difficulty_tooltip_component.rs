use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/difficulty/difficulty-tooltip.component.scss",
        selector_path: ".difficulty-tooltip",
        properties: &[
            ("position", "fixed"),
            ("background", "color-mix(in srgb, #11131f 95%, transparent)"),
            ("border-radius", "4px"),
            ("box-shadow", "1px 1px 10px rgba(0, 0, 0, 0.5)"),
            ("color", "var(--tooltip-grey)"),
            ("padding", "10px 15px"),
            ("text-align", "left"),
            ("pointer-events", "none"),
            ("max-width", "300px"),
            ("min-width", "200px"),
            ("text-align", "center"),
        ],
        leptos_css: "position: fixed; background: color-mix(in srgb, #11131f 95%, transparent); border-radius: 4px; box-shadow: 1px 1px 10px rgba(0, 0, 0, 0.5); color: var(--tooltip-grey); padding: 10px 15px; text-align: left; pointer-events: none; max-width: 300px; min-width: 200px; text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/difficulty/difficulty-tooltip.component.scss",
        selector_path: ".difficulty-tooltip p",
        properties: &[("margin", "0"), ("white-space", "nowrap")],
        leptos_css: "margin: 0; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/difficulty/difficulty-tooltip.component.scss",
        selector_path: ".next-block",
        properties: &[("text-transform", "lowercase")],
        leptos_css: "text-transform: lowercase;",
        gpui: GpuiOutcome::Paint,
    },
];
