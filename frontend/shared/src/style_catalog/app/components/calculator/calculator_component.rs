use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".input-group-text",
        properties: &[("width", "75px")],
        leptos_css: "width: 75px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".bitcoin-satoshis-text",
        properties: &[("font-size", "40px")],
        leptos_css: "font-size: 40px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".fiat-text",
        properties: &[("font-size", "24px")],
        leptos_css: "font-size: 24px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".symbol",
        properties: &[("font-style", "italic")],
        leptos_css: "font-style: italic;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: "@media (max-width: 767.98px) { .bitcoin-satoshis-text }",

        properties: &[("font-size", "30px")],
        leptos_css: "font-size: 30px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".sats",
        properties: &[("font-size", "20px"), ("margin-left", "5px")],
        leptos_css: "font-size: 20px; margin-left: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".row",
        properties: &[("margin", "auto")],
        leptos_css: "margin: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".form-control.disabled",
        properties: &[
            ("pointer-events", "none"),
            ("background-color", "var(--secondary)"),
            ("opacity", "0.75"),
        ],
        leptos_css: "pointer-events: none; background-color: var(--secondary); opacity: 0.75;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".calculator-date-group",
        properties: &[("width", "172px")],
        leptos_css: "width: 172px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".calculator-form",
        properties: &[("width", "100%"), ("max-width", "373px")],
        leptos_css: "width: 100%; max-width: 373px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".calculator-row > *",
        properties: &[("padding-left", "0"), ("padding-right", "0")],
        leptos_css: "padding-left: 0; padding-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ".calculator-inline-row > *",
        properties: &[
            ("flex", "0 0 auto"),
            ("width", "auto"),
            ("max-width", "100%"),
            ("padding-left", "0"),
            ("padding-right", "0"),
        ],
        leptos_css: "flex: 0 0 auto; width: auto; max-width: 100%; padding-left: 0; padding-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/calculator/calculator.component.scss",
        selector_path: ":host ::ng-deep ngb-datepicker .ngb-dp-weekdays, :host ::ng-deep ngb-datepicker .ngb-dp-header",
        properties: &[("background-color", "var(--bg) !important")],
        leptos_css: "background-color: var(--bg) !important;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
];
