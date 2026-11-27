use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/nodes-channels/node-channels.component.scss",
        selector_path: ".node-channels-container",
        properties: &[("position", "relative")],
        leptos_css: "position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-channels/node-channels.component.scss",
        selector_path: ".loading-spinner",
        properties: &[
            ("position", "absolute"),
            ("top", "0"),
            ("left", "0"),
            ("right", "0"),
            ("width", "100%"),
            ("z-index", "99"),
        ],
        leptos_css: "position: absolute; top: 0; left: 0; right: 0; width: 100%; z-index: 99;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/nodes-channels/node-channels.component.scss",
        selector_path: ".spinner-border",
        properties: &[("position", "relative"), ("top", "225px")],
        leptos_css: "position: relative; top: 225px;",
        gpui: GpuiOutcome::Paint,
    },
];
