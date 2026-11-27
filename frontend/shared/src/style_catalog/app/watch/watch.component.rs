use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".wallet-brand",
        properties: &[
            ("display", "flex"),
            ("align-items", "flex-start"),
            ("margin", "0.5rem 0 1.25rem"),
        ],
        leptos_css: "display: flex; align-items: flex-start; margin: 0.5rem 0 1.25rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".wallet-logo",
        properties: &[
            ("display", "block"),
            ("height", "56px"),
            ("width", "auto"),
            ("min-width", "0"),
            ("max-width", "100%"),
        ],
        leptos_css: "display: block; height: 56px; width: auto; min-width: 0; max-width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".preview",
        properties: &[
            ("flex-shrink", "0"),
            ("font-size", "10px"),
            ("padding", "5px"),
            ("margin", "3px 0 0 10px"),
        ],
        leptos_css: "flex-shrink: 0; font-size: 10px; padding: 5px; margin: 3px 0 0 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: "@media (max-width: 575.98px) { .wallet-logo }",
        properties: &[("height", "44px")],
        leptos_css: "height: 44px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: "@media (max-width: 575.98px) { .preview }",
        properties: &[("margin-top", "0")],
        leptos_css: "margin-top: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".wallet-status",
        properties: &[
            ("display", "flex"),
            ("min-height", "2.25rem"),
            ("align-items", "center"),
            ("justify-content", "flex-end"),
            ("gap", "0.75rem"),
            ("margin-top", "0.75rem"),
        ],
        leptos_css: "display: flex; min-height: 2.25rem; align-items: center; justify-content: flex-end; gap: 0.75rem; margin-top: 0.75rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".status-line",
        properties: &[
            ("min-width", "0"),
            ("color", "var(--transparent-fg)"),
            ("text-align", "right"),
        ],
        leptos_css: "min-width: 0; color: var(--transparent-fg); text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".watch-tabs",
        properties: &[
            ("margin-top", "1rem"),
            ("overflow-x", "auto"),
            ("overflow-y", "hidden"),
            ("flex-wrap", "nowrap"),
            ("border-color", "var(--transparent-fg)"),
            ("scrollbar-width", "thin"),
        ],
        leptos_css: "margin-top: 1rem; overflow-x: auto; overflow-y: hidden; flex-wrap: nowrap; border-color: var(--transparent-fg); scrollbar-width: thin;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".watch-tabs .nav-link",
        properties: &[
            ("color", "var(--transparent-fg)"),
            ("background", "transparent"),
            ("border-color", "transparent"),
            ("white-space", "nowrap"),
            ("cursor", "pointer"),
        ],
        leptos_css: "color: var(--transparent-fg); background: transparent; border-color: transparent; white-space: nowrap; cursor: pointer;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".watch-tabs .nav-link:focus-visible",
        properties: &[("box-shadow", "inset 0 0 0 2px var(--info)")],
        leptos_css: "box-shadow: inset 0 0 0 2px var(--info);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".watch-tabs .nav-link.active",
        properties: &[
            ("color", "var(--fg)"),
            ("background", "var(--box-bg)"),
            (
                "border-color",
                "var(--transparent-fg) var(--transparent-fg) var(--box-bg)",
            ),
        ],
        leptos_css: "color: var(--fg); background: var(--box-bg); border-color: var(--transparent-fg) var(--transparent-fg) var(--box-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".watch-tab-content",
        properties: &[("padding", "1rem 0 2rem")],
        leptos_css: "padding: 1rem 0 2rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/watch.component.scss",
        selector_path: ".wallet-loading",
        properties: &[("color", "var(--transparent-fg)")],
        leptos_css: "color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
];
