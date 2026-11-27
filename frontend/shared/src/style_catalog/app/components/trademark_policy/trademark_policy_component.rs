use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/trademark-policy/trademark-policy.component.scss",
        selector_path: ".trademark-policy .logo",
        properties: &[("height", "62.5px"), ("width", "250px"), ("margin", "10px")],
        leptos_css: "height: 62.5px; width: 250px; margin: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/trademark-policy/trademark-policy.component.scss",
        selector_path: ".trademark-policy .circle",
        properties: &[("margin-top", "-50px"), ("margin-bottom", "-30px")],
        leptos_css: "margin-top: -50px; margin-bottom: -30px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/trademark-policy/trademark-policy.component.scss",
        selector_path: ".trademark-policy .circle:before",
        properties: &[("content", "\" ●\""), ("font-size", "250px")],
        leptos_css: "content: \" ●\"; font-size: 250px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/trademark-policy/trademark-policy.component.scss",
        selector_path: ".trademark-policy ol",
        properties: &[("list-style-type", "upper-roman")],
        leptos_css: "list-style-type: upper-roman;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/trademark-policy/trademark-policy.component.scss",
        selector_path: ".trademark-policy ol li",
        properties: &[("font-size", "1.75rem")],
        leptos_css: "font-size: 1.75rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/trademark-policy/trademark-policy.component.scss",
        selector_path: ".trademark-policy ol ol",
        properties: &[("list-style-type", "lower-alpha")],
        leptos_css: "list-style-type: lower-alpha;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/trademark-policy/trademark-policy.component.scss",
        selector_path: ".trademark-policy ol ol li",
        properties: &[("font-size", "1.25rem")],
        leptos_css: "font-size: 1.25rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/trademark-policy/trademark-policy.component.scss",
        selector_path: ".trademark-policy ol ol ol",
        properties: &[("list-style-type", "lower-roman")],
        leptos_css: "list-style-type: lower-roman;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/trademark-policy/trademark-policy.component.scss",
        selector_path: ".trademark-policy ol ol ol li",
        properties: &[("font-size", "1.15rem")],
        leptos_css: "font-size: 1.15rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/trademark-policy/trademark-policy.component.scss",
        selector_path: ".trademark-policy .table",
        properties: &[("--bs-table-border-color", "var(--fg)")],
        leptos_css: "--bs-table-border-color: var(--fg);",
        gpui: GpuiOutcome::Paint,
    },
];
