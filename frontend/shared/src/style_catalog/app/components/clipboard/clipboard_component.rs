use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/clipboard/clipboard.component.scss",
        selector_path: ".btn-link",
        properties: &[("padding", "0.25rem 0 0.1rem 0rem"), ("line-height", "0")],
        leptos_css: "padding: 0.25rem 0 0.1rem 0rem; line-height: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clipboard/clipboard.component.scss",
        selector_path: ".padding",
        properties: &[("padding-left", "0.4rem")],
        leptos_css: "padding-left: 0.4rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/clipboard/clipboard.component.scss",
        selector_path: ".copied-message",
        properties: &[
            ("background", "color-mix(in srgb, #11131f 95%, transparent)"),
            ("color", "var(--tooltip-grey)"),
            ("font-family", "sans-serif"),
            ("font-size", "0.8rem"),
            ("font-weight", "400"),
            ("text-decoration", "none"),
            ("text-align", "left"),
            ("padding", "0.6em 0.75rem"),
            ("border-radius", "4px"),
            ("position", "absolute"),
            ("white-space", "nowrap"),
            ("box-shadow", "0 0.5rem 1rem -0.5rem #000"),
            ("z-index", "1000"),
            ("opacity", "0.9"),
        ],
        leptos_css: "background: color-mix(in srgb, #11131f 95%, transparent); color: var(--tooltip-grey); font-family: sans-serif; font-size: 0.8rem; font-weight: 400; text-decoration: none; text-align: left; padding: 0.6em 0.75rem; border-radius: 4px; position: absolute; white-space: nowrap; box-shadow: 0 0.5rem 1rem -0.5rem #000; z-index: 1000; opacity: 0.9;",
        gpui: GpuiOutcome::Paint,
    },
];
