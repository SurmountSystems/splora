use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/channel/channel-box/channel-box.component.scss",
        selector_path: ".box-top",
        properties: &[("display", "flex")],
        leptos_css: "display: flex;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-box/channel-box.component.scss",
        selector_path: ".box-left",
        properties: &[("width", "100%")],
        leptos_css: "width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-box/channel-box.component.scss",
        selector_path: ".box-right",
        properties: &[
            ("text-align", "right"),
            ("margin", "auto"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "text-align: right; margin: auto; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-box/channel-box.component.scss",
        selector_path: ".shared-block",
        properties: &[("color", "var(--transparent-fg)"), ("font-size", "12px")],
        leptos_css: "color: var(--transparent-fg); font-size: 12px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-box/channel-box.component.scss",
        selector_path: "@media (max-width: 768px) { .box }",

        properties: &[("margin-bottom", "20px")],
        leptos_css: "margin-bottom: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-box/channel-box.component.scss",
        selector_path: "@media (max-width: 768px) { td:first-child }",

        properties: &[("white-space", "normal")],
        leptos_css: "white-space: normal;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-box/channel-box.component.scss",
        selector_path: ".base-fee",
        properties: &[("max-width", "0")],
        leptos_css: "max-width: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-box/channel-box.component.scss",
        selector_path: "@media (max-width: 768px) { .base-fee }",

        properties: &[("padding-right", "0px")],
        leptos_css: "padding-right: 0px;",
        gpui: GpuiOutcome::Paint,
    },
];
