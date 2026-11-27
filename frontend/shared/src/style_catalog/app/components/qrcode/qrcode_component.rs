use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/qrcode/qrcode.component.scss",
        selector_path: "img",
        properties: &[
            ("position", "absolute"),
            ("top", "50%"),
            ("left", "50%"),
            ("width", "42px"),
            ("height", "42px"),
            ("transform", "translate(-50%, -50%)"),
        ],
        leptos_css: "position: absolute; top: 50%; left: 50%; width: 42px; height: 42px; transform: translate(-50%, -50%);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/qrcode/qrcode.component.scss",
        selector_path: ".holder",
        properties: &[("position", "relative")],
        leptos_css: "position: relative;",
        gpui: GpuiOutcome::Paint,
    },
];
