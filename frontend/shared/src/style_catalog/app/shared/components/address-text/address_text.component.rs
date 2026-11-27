use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/shared/components/address-text/address-text.component.scss",
        selector_path: ".address-text",
        properties: &[
            ("text-overflow", "unset"),
            ("display", "flex"),
            ("flex-direction", "row"),
            ("align-items", "start"),
            ("position", "relative"),
            ("font-family", "monospace"),
        ],
        leptos_css: "text-overflow: unset; display: flex; flex-direction: row; align-items: start; position: relative; font-family: monospace;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/address-text/address-text.component.scss",
        selector_path: ".address-text .infix",
        properties: &[
            ("flex-grow", "0"),
            ("flex-shrink", "1"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("user-select", "none"),
            ("text-decoration", "underline 2px"),
        ],
        leptos_css: "flex-grow: 0; flex-shrink: 1; overflow: hidden; text-overflow: ellipsis; user-select: none; text-decoration: underline 2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/address-text/address-text.component.scss",
        selector_path: ".address-text .prefix, .address-text .postfix",
        properties: &[
            ("flex-shrink", "0"),
            ("flex-grow", "0"),
            ("user-select", "none"),
            ("text-decoration", "underline var(--red) 2px"),
        ],
        leptos_css: "flex-shrink: 0; flex-grow: 0; user-select: none; text-decoration: underline var(--red) 2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/address-text/address-text.component.scss",
        selector_path: ".poison-alert",
        properties: &[("margin-left", "0.5em"), ("color", "var(--yellow)")],
        leptos_css: "margin-left: 0.5em; color: var(--yellow);",
        gpui: GpuiOutcome::Paint,
    },
];
