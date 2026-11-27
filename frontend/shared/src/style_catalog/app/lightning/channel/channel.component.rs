use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: ".title-container",
        properties: &[("display", "flex"), ("flex-direction", "row")],
        leptos_css: "display: flex; flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: "@media (max-width: 767.98px) { .title-container }",

        properties: &[("flex-direction", "column")],
        leptos_css: "flex-direction: column;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: ".tx-link",
        properties: &[("display", "flex"), ("flex-grow", "1")],
        leptos_css: "display: flex; flex-grow: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: "@media (min-width: 768px) { .tx-link }",

        properties: &[
            ("top", "1px"),
            ("position", "relative"),
            ("align-self", "end"),
            ("margin-left", "15px"),
            ("margin-top", "0px"),
            ("margin-bottom", "4px"),
        ],
        leptos_css: "top: 1px; position: relative; align-self: end; margin-left: 15px; margin-top: 0px; margin-bottom: 4px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: "@media (max-width: 768px) { .tx-link }",

        properties: &[("order", "2")],
        leptos_css: "order: 2;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: ".badges",
        properties: &[("font-size", "20px")],
        leptos_css: "font-size: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: ".badge",
        properties: &[("margin-right", "5px")],
        leptos_css: "margin-right: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: "app-fiat",
        properties: &[("display", "block"), ("font-size", "13px")],
        leptos_css: "display: block; font-size: 13px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: "@media (min-width: 768px) { app-fiat }",

        properties: &[
            ("font-size", "14px"),
            ("display", "inline-block"),
            ("margin-left", "10px"),
        ],
        leptos_css: "font-size: 14px; display: inline-block; margin-left: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: ".closing-header",
        properties: &[
            ("flex-direction", "row"),
            ("margin-bottom", "1rem"),
            ("align-items", "center"),
        ],
        leptos_css: "flex-direction: row; margin-bottom: 1rem; align-items: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: "h3",
        properties: &[("margin-bottom", "0rem")],
        leptos_css: "margin-bottom: 0rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: "@media (max-width: 768px) { h3 }",

        properties: &[("font-size", "1.4rem"), ("white-space", "nowrap")],
        leptos_css: "font-size: 1.4rem; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: "@media (max-width: 768px) { .closing-header }",

        properties: &[("flex-wrap", "wrap")],
        leptos_css: "flex-wrap: wrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: "@media (max-width: 768px) { app-closing-type }",

        properties: &[("flex-basis", "100%"), ("order", "1")],
        leptos_css: "flex-basis: 100%; order: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: ".loading-spinner",
        properties: &[
            ("position", "absolute"),
            ("top", "400px"),
            ("z-index", "99"),
            ("width", "100%"),
            ("left", "0"),
        ],
        leptos_css: "position: absolute; top: 400px; z-index: 99; width: 100%; left: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: "@media (max-width: 767.98px) { .loading-spinner }",

        properties: &[("top", "450px")],
        leptos_css: "top: 450px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel.component.scss",
        selector_path: ".details-button",
        properties: &[("align-self", "center"), ("margin-inline-start", "auto")],
        leptos_css: "align-self: center; margin-inline-start: auto;",
        gpui: GpuiOutcome::Paint,
    },
];
