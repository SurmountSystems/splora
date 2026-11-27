use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: ".logo-container",
        properties: &[("width", "50px")],
        leptos_css: "width: 50px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: ".header",
        properties: &[("text-align", "center"), ("display", "flex")],
        leptos_css: "text-align: center; display: flex;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: "h1",
        properties: &[("margin-inline-start", "15px")],
        leptos_css: "margin-inline-start: 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: ".qr-wrapper",
        properties: &[
            ("background-color", "var(--fg)"),
            ("padding", "10px"),
            ("padding-bottom", "5px"),
            ("display", "inline-block"),
            ("position", "absolute"),
            ("bottom", "50px"),
            ("left", "-175px"),
            ("z-index", "100"),
        ],
        leptos_css: "background-color: var(--fg); padding: 10px; padding-bottom: 5px; display: inline-block; position: absolute; bottom: 50px; left: -175px; z-index: 100;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: ".dropdownLabel",
        properties: &[("min-width", "50px"), ("display", "inline-block")],
        leptos_css: "min-width: 50px; display: inline-block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: "#inputGroupFileAddon04",
        properties: &[
            ("position", "relative"),
            ("border-start-start-radius", "var(--bs-border-radius)"),
            ("border-start-end-radius", "var(--bs-border-radius)"),
            ("border-end-start-radius", "var(--bs-border-radius)"),
            ("border-end-end-radius", "var(--bs-border-radius)"),
        ],
        leptos_css: "position: relative; border-start-start-radius: var(--bs-border-radius); border-start-end-radius: var(--bs-border-radius); border-end-start-radius: var(--bs-border-radius); border-end-end-radius: var(--bs-border-radius);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: ".input-group > .form-control",
        properties: &[
            (
                "border-start-end-radius",
                "var(--bs-border-radius) !important",
            ),
            (
                "border-end-end-radius",
                "var(--bs-border-radius) !important",
            ),
        ],
        leptos_css: "border-start-end-radius: var(--bs-border-radius) !important; border-end-end-radius: var(--bs-border-radius) !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: ".toggle-holder",
        properties: &[
            ("display", "flex"),
            ("width", "100%"),
            ("justify-content", "flex-end"),
        ],
        leptos_css: "display: flex; width: 100%; justify-content: flex-end;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: "@media (max-width: 767.98px) { .text-truncate }",
        properties: &[("width", "120px")],
        leptos_css: "width: 120px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: "@media (max-width: 767.98px) { .btn }",
        properties: &[
            ("padding", "0.25rem 0.5rem"),
            ("font-size", "0.875rem"),
            ("line-height", "1.5"),
            ("border-radius", "0.2rem"),
        ],
        leptos_css: "padding: 0.25rem 0.5rem; font-size: 0.875rem; line-height: 1.5; border-radius: 0.2rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: ".second-line",
        properties: &[("font-size", "12px")],
        leptos_css: "font-size: 12px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: ".description-text",
        properties: &[("white-space", "break-spaces")],
        leptos_css: "white-space: break-spaces;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: ".timestamp-first .input-group input",
        properties: &[("margin-inline-end", "0.25rem")],
        leptos_css: "margin-inline-end: 0.25rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/group/group.component.scss",
        selector_path: ":host-context(.rtl-layout) .timestamp-first .input-group button",
        properties: &[("margin-inline-end", "0.25rem")],
        leptos_css: "margin-inline-end: 0.25rem;",
        gpui: GpuiOutcome::Paint,
    },
];
