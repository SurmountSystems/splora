use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: ".second-line",
        properties: &[("font-size", "12px")],
        leptos_css: "font-size: 12px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: ".sats",
        properties: &[
            ("color", "var(--transparent-fg)"),
            ("font-size", "12px"),
            ("top", "0px"),
        ],
        leptos_css: "color: var(--transparent-fg); font-size: 12px; top: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: "@media (min-width: 435px) { .formRadioGroup }",
        properties: &[("position", "absolute"), ("right", "0"), ("top", "-46px")],
        leptos_css: "position: absolute; right: 0; top: -46px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: "@media (max-width: 435px) { .formRadioGroup }",
        properties: &[("display", "flex")],
        leptos_css: "display: flex;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: ":host-context(.rtl-layout) .formRadioGroup",
        properties: &[("direction", "ltr")],
        leptos_css: "direction: ltr;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: "@media (min-width: 435px) { :host-context(.rtl-layout) .formRadioGroup }",
        properties: &[("right", "unset"), ("left", "0")],
        leptos_css: "right: unset; left: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: "@media (max-width: 435px) { .btn-group }",
        properties: &[("flex-grow", "1")],
        leptos_css: "flex-grow: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: ".alias",
        properties: &[("padding-left", "0")],
        leptos_css: "padding-left: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: "@media (max-width: 815px) { .feerate }",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: "@media (max-width: 710px) { .status }",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: "@media (max-width: 600px) { .nodedetails }",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: "@media (max-width: 500px) { .liquidity }",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channels-list/channels-list.component.scss",
        selector_path: ".channelid",
        properties: &[("padding-right", "0")],
        leptos_css: "padding-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
];
