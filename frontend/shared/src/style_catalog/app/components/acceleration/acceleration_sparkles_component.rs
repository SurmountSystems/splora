use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/acceleration/sparkles/acceleration-sparkles.component.scss",
        selector_path: ".sparkles",
        properties: &[
            ("position", "absolute"),
            ("top", "var(--block-size)"),
            ("height", "50px"),
            ("right", "0"),
        ],
        leptos_css: "position: absolute; top: var(--block-size); height: 50px; right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/sparkles/acceleration-sparkles.component.scss",
        selector_path: ".sparkle",
        properties: &[
            ("position", "absolute"),
            ("color", "rgba(152, 88, 255, 0.75)"),
            ("opacity", "0"),
            ("transform", "scale(0.8) rotate(0deg)"),
            (
                "animation",
                "pop ease 2000ms forwards, sparkle ease 500ms infinite",
            ),
        ],
        leptos_css: "position: absolute; color: rgba(152, 88, 255, 0.75); opacity: 0; transform: scale(0.8) rotate(0deg); animation: pop ease 2000ms forwards, sparkle ease 500ms infinite;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/sparkles/acceleration-sparkles.component.scss",
        selector_path: ".inner-sparkle",
        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/acceleration/sparkles/acceleration-sparkles.component.scss",
        selector_path: "@keyframes pop 0%",
        properties: &[("transform", "scale(0.8) rotate(0deg)"), ("opacity", "0")],
        leptos_css: "transform: scale(0.8) rotate(0deg); opacity: 0;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/acceleration/sparkles/acceleration-sparkles.component.scss",
        selector_path: "@keyframes pop 20%",
        properties: &[("transform", "scale(1) rotate(72deg)"), ("opacity", "1")],
        leptos_css: "transform: scale(1) rotate(72deg); opacity: 1;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/acceleration/sparkles/acceleration-sparkles.component.scss",
        selector_path: "@keyframes pop 100%",
        properties: &[("transform", "scale(0) rotate(360deg)"), ("opacity", "0")],
        leptos_css: "transform: scale(0) rotate(360deg); opacity: 0;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/acceleration/sparkles/acceleration-sparkles.component.scss",
        selector_path: "@keyframes sparkle 0%",
        properties: &[("color", "rgba(152, 88, 255, 0.75)")],
        leptos_css: "color: rgba(152, 88, 255, 0.75);",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/acceleration/sparkles/acceleration-sparkles.component.scss",
        selector_path: "@keyframes sparkle 50%",
        properties: &[("color", "rgba(198, 162, 255, 0.75)")],
        leptos_css: "color: rgba(198, 162, 255, 0.75);",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/acceleration/sparkles/acceleration-sparkles.component.scss",
        selector_path: "@keyframes sparkle 100%",
        properties: &[("color", "rgba(152, 88, 255, 0.75)")],
        leptos_css: "color: rgba(152, 88, 255, 0.75);",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
];
