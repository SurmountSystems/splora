use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig",
        properties: &[("cursor", "pointer")],
        leptos_css: "cursor: pointer;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig.sighash-0",
        properties: &[("--sigColor", "var(--green)")],
        leptos_css: "--sigColor: var(--green);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig.sighash-1",
        properties: &[("--sigColor", "var(--green)")],
        leptos_css: "--sigColor: var(--green);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig.sighash-2",
        properties: &[("--sigColor", "gold")],
        leptos_css: "--sigColor: gold;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig.sighash-3",
        properties: &[("--sigColor", "cornflowerblue")],
        leptos_css: "--sigColor: cornflowerblue;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig.sighash-129",
        properties: &[("--sigColor", "darkviolet")],
        leptos_css: "--sigColor: darkviolet;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig.sighash-130",
        properties: &[("--sigColor", "darkorange")],
        leptos_css: "--sigColor: darkorange;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig.sighash-131",
        properties: &[("--sigColor", "var(--pink)")],
        leptos_css: "--sigColor: var(--pink);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig.sig-no-lock",
        properties: &[("--sigColor", "var(--red)")],
        leptos_css: "--sigColor: var(--red);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig",
        properties: &[("color", "var(--sigColor)")],
        leptos_css: "color: var(--sigColor);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig:hover, .sig.hovered",
        properties: &[("color", "color-mix(in srgb, var(--sigColor) 60%, white)")],
        leptos_css: "color: color-mix(in srgb, var(--sigColor) 60%, white);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig ::ng-deep svg path",
        properties: &[("pointer-events", "auto")],
        leptos_css: "pointer-events: auto;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig-key",
        properties: &[
            ("position", "absolute"),
            ("top", "0"),
            ("left", "0"),
            ("transition", "transform 0.2s ease"),
            ("pointer-events", "none"),
        ],
        leptos_css: "position: absolute; top: 0; left: 0; transition: transform 0.2s ease; pointer-events: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".sig-key.sig-inline",
        properties: &[
            ("position", "relative"),
            ("pointer-events", "auto"),
            ("margin-left", "4px"),
            ("margin-right", "-4px"),
        ],
        leptos_css: "position: relative; pointer-events: auto; margin-left: 4px; margin-right: -4px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".script",
        properties: &[("margin-left", "4px"), ("margin-right", "-4px")],
        leptos_css: "margin-left: 4px; margin-right: -4px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/asm/asm.component.scss",
        selector_path: ".timelock",
        properties: &[("margin-left", "4px"), ("margin-right", "-4px")],
        leptos_css: "margin-left: 4px; margin-right: -4px;",
        gpui: GpuiOutcome::Paint,
    },
];
