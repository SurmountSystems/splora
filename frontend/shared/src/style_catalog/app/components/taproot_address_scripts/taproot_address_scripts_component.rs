use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/taproot-address-scripts/taproot-address-scripts.component.scss",
        selector_path: ".fade-out",
        properties: &[("position", "relative")],
        leptos_css: "position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/taproot-address-scripts/taproot-address-scripts.component.scss",
        selector_path: ".fade-out::before",
        properties: &[
            ("content", "''"),
            ("position", "absolute"),
            ("width", "100%"),
            ("height", "40px"),
            ("top", "-40px"),
            (
                "background",
                "linear-gradient(to top,\n        var(--fade-out-box-bg-end) 0%,\n        var(--fade-out-box-bg-end) 30%,\n        var(--fade-out-box-bg-start) 100%)",
            ),
            ("z-index", "10000000"),
        ],
        leptos_css: "content: ''; position: absolute; width: 100%; height: 40px; top: -40px; background: linear-gradient(to top,\n        var(--fade-out-box-bg-end) 0%,\n        var(--fade-out-box-bg-end) 30%,\n        var(--fade-out-box-bg-start) 100%); z-index: 10000000;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/taproot-address-scripts/taproot-address-scripts.component.scss",
        selector_path: ".toggle-wrapper",
        properties: &[
            ("display", "flex"),
            ("justify-content", "center"),
            ("width", "100%"),
        ],
        leptos_css: "display: flex; justify-content: center; width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/taproot-address-scripts/taproot-address-scripts.component.scss",
        selector_path: ".button-container",
        properties: &[("display", "flex"), ("justify-content", "center")],
        leptos_css: "display: flex; justify-content: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/taproot-address-scripts/taproot-address-scripts.component.scss",
        selector_path: ".graph-toggle",
        properties: &[("margin-top", "10px")],
        leptos_css: "margin-top: 10px;",
        gpui: GpuiOutcome::Paint,
    },
];
