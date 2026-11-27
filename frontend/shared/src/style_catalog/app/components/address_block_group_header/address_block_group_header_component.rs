use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: ":host",
        properties: &[
            ("display", "flex"),
            ("flex-wrap", "wrap"),
            ("align-items", "center"),
            ("gap", "0.5rem 1rem"),
            ("margin-top", "1rem"),
            ("padding-bottom", "0.5rem"),
            ("margin-bottom", "0.25rem"),
            ("scroll-margin-top", "90px"),
        ],
        leptos_css: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 1rem; margin-top: 1rem; padding-bottom: 0.5rem; margin-bottom: 0.25rem; scroll-margin-top: 90px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: "@media (max-width: 575px) { :host }",

        properties: &[("scroll-margin-top", "120px")],
        leptos_css: "scroll-margin-top: 120px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: ":host:focus-visible",
        properties: &[("outline", "1px solid var(--info)")],
        leptos_css: "outline: 1px solid var(--info);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: ":host.first-group",
        properties: &[("margin-top", "1.5rem")],
        leptos_css: "margin-top: 1.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: ".block-cube",
        properties: &[
            ("--block-size", "18px"),
            ("position", "relative"),
            ("flex", "0 0 auto"),
            ("width", "var(--block-size)"),
            ("height", "var(--block-size)"),
            ("margin-left", "3px"),
        ],
        leptos_css: "--block-size: 18px; position: relative; flex: 0 0 auto; width: var(--block-size); height: var(--block-size); margin-left: 3px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: ".block-cube::after",
        properties: &[
            ("content", "''"),
            ("width", "var(--block-size)"),
            ("height", "calc(0.192 * var(--block-size))"),
            ("position", "absolute"),
            ("top", "calc(-0.192 * var(--block-size))"),
            ("left", "calc(-0.16 * var(--block-size))"),
            ("background-color", "var(--block-top)"),
            ("transform", "skew(40deg)"),
            ("transform-origin", "top"),
        ],
        leptos_css: "content: ''; width: var(--block-size); height: calc(0.192 * var(--block-size)); position: absolute; top: calc(-0.192 * var(--block-size)); left: calc(-0.16 * var(--block-size)); background-color: var(--block-top); transform: skew(40deg); transform-origin: top;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: ".block-cube::before",
        properties: &[
            ("content", "''"),
            ("width", "calc(0.16 * var(--block-size))"),
            ("height", "var(--block-size)"),
            ("position", "absolute"),
            ("top", "calc(-0.096 * var(--block-size))"),
            ("left", "calc(-0.16 * var(--block-size))"),
            ("background-color", "var(--block-side)"),
            ("transform", "skewY(50deg)"),
            ("transform-origin", "top"),
        ],
        leptos_css: "content: ''; width: calc(0.16 * var(--block-size)); height: var(--block-size); position: absolute; top: calc(-0.096 * var(--block-size)); left: calc(-0.16 * var(--block-size)); background-color: var(--block-side); transform: skewY(50deg); transform-origin: top;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: ".block-cube::before",
        properties: &[("filter", "brightness(0.7)")],
        leptos_css: "filter: brightness(0.7);",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: ".block-cube.pending::after",
        properties: &[("background", "var(--mempool-block-top)")],
        leptos_css: "background: var(--mempool-block-top);",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: ".block-cube.pending::before",
        properties: &[("background", "var(--mempool-block-side)")],
        leptos_css: "background: var(--mempool-block-side);",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: ".group-count, .group-partial",
        properties: &[("font-size", "0.85rem"), ("color", "var(--transparent-fg)")],
        leptos_css: "font-size: 0.85rem; color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-block-group-header/address-block-group-header.component.scss",
        selector_path: "h3",
        properties: &[("font-size", "1.1rem"), ("margin-bottom", "0")],
        leptos_css: "font-size: 1.1rem; margin-bottom: 0;",
        gpui: GpuiOutcome::Paint,
    },
];
