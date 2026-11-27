use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/shared/components/testnet-alert/testnet-alert.component.scss",
        selector_path: ".alert-danger",
        properties: &[
            ("color", "#fff"),
            ("background-color", "#b71c1c"),
            ("border-color", "#b71c1c"),
            ("padding", "0.5rem 1.25rem"),
            ("margin", "0px 10px 0px 10px"),
            ("display", "flex"),
            ("justify-content", "center"),
            ("align-items", "center"),
            ("position", "relative"),
        ],
        leptos_css: "color: #fff; background-color: #b71c1c; border-color: #b71c1c; padding: 0.5rem 1.25rem; margin: 0px 10px 0px 10px; display: flex; justify-content: center; align-items: center; position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/testnet-alert/testnet-alert.component.scss",
        selector_path: ".message-container",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "column"),
            ("margin", "0 auto"),
        ],
        leptos_css: "display: flex; flex-direction: column; margin: 0 auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/testnet-alert/testnet-alert.component.scss",
        selector_path: ".btn-close",
        properties: &[("top", "50%"), ("transform", "translateY(-50%)")],
        leptos_css: "top: 50%; transform: translateY(-50%);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/testnet-alert/testnet-alert.component.scss",
        selector_path: ".btn-close:hover, .btn-close:focus",
        properties: &[("opacity", "1")],
        leptos_css: "opacity: 1;",
        gpui: GpuiOutcome::Paint,
    },
];
