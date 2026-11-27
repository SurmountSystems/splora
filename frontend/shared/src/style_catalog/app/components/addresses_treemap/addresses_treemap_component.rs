use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/addresses-treemap/addresses-treemap.component.scss",
        selector_path: ".node-channels-container",
        properties: &[("position", "relative")],
        leptos_css: "position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/addresses-treemap/addresses-treemap.component.scss",
        selector_path: ".loading-spinner",
        properties: &[
            ("position", "absolute"),
            ("top", "0"),
            ("left", "0"),
            ("right", "0"),
            ("width", "100%"),
            ("z-index", "100"),
        ],
        leptos_css: "position: absolute; top: 0; left: 0; right: 0; width: 100%; z-index: 100;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/addresses-treemap/addresses-treemap.component.scss",
        selector_path: ".spinner-border",
        properties: &[("position", "relative"), ("top", "225px")],
        leptos_css: "position: relative; top: 225px;",
        gpui: GpuiOutcome::Paint,
    },
];
