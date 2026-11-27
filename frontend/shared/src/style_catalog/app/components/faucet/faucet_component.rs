use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/faucet/faucet.component.scss",
        selector_path: ".formGroup",
        properties: &[("width", "100%")],
        leptos_css: "width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/faucet/faucet.component.scss",
        selector_path: ".input-group",
        properties: &[
            ("display", "flex"),
            ("flex-wrap", "wrap"),
            ("align-items", "stretch"),
            ("justify-content", "flex-end"),
            ("row-gap", "0.5rem"),
            ("gap", "0.5rem"),
        ],
        leptos_css: "display: flex; flex-wrap: wrap; align-items: stretch; justify-content: flex-end; row-gap: 0.5rem; gap: 0.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/faucet/faucet.component.scss",
        selector_path: ".input-group .form-control",
        properties: &[("min-width", "160px"), ("flex-grow", "100")],
        leptos_css: "min-width: 160px; flex-grow: 100;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/faucet/faucet.component.scss",
        selector_path: ".input-group .button-group",
        properties: &[("display", "flex"), ("align-items", "stretch")],
        leptos_css: "display: flex; align-items: stretch;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/faucet/faucet.component.scss",
        selector_path: ".input-group .submit-button, .input-group .button-group, .input-group .button-group .btn",
        properties: &[("flex-grow", "1")],
        leptos_css: "flex-grow: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/faucet/faucet.component.scss",
        selector_path: ".input-group .submit-button:disabled",
        properties: &[("cursor", "not-allowed")],
        leptos_css: "cursor: not-allowed;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/faucet/faucet.component.scss",
        selector_path: ".input-group #satoshis::after",
        properties: &[
            ("content", "'sats'"),
            ("position", "absolute"),
            ("right", "0.5em"),
            ("top", "0"),
            ("bottom", "0"),
        ],
        leptos_css: "content: 'sats'; position: absolute; right: 0.5em; top: 0; bottom: 0;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/faucet/faucet.component.scss",
        selector_path: ".faucet-container",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "column"),
            ("align-items", "center"),
            ("justify-content", "flex-start"),
            ("width", "100%"),
            ("max-width", "800px"),
            ("margin", "auto"),
        ],
        leptos_css: "display: flex; flex-direction: column; align-items: center; justify-content: flex-start; width: 100%; max-width: 800px; margin: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/faucet/faucet.component.scss",
        selector_path: ".invalid",
        properties: &[("border-width", "1px"), ("border-color", "var(--red)")],
        leptos_css: "border-width: 1px; border-color: var(--red);",
        gpui: GpuiOutcome::Paint,
    },
];
