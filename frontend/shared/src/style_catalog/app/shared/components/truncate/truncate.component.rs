use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/shared/components/truncate/truncate.component.scss",
        selector_path: ".truncate",
        properties: &[
            ("text-overflow", "unset"),
            ("display", "flex"),
            ("flex-direction", "row"),
            ("align-items", "start"),
            ("position", "relative"),
        ],
        leptos_css: "text-overflow: unset; display: flex; flex-direction: row; align-items: start; position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/truncate/truncate.component.scss",
        selector_path: ".truncate .truncate-link",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "row"),
            ("align-items", "baseline"),
            ("flex-shrink", "1"),
            ("overflow", "hidden"),
        ],
        leptos_css: "display: flex; flex-direction: row; align-items: baseline; flex-shrink: 1; overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/truncate/truncate.component.scss",
        selector_path: ".truncate .first",
        properties: &[
            ("flex-grow", "0"),
            ("flex-shrink", "1"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("user-select", "none"),
        ],
        leptos_css: "flex-grow: 0; flex-shrink: 1; overflow: hidden; text-overflow: ellipsis; user-select: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/truncate/truncate.component.scss",
        selector_path: ".truncate .last-four",
        properties: &[
            ("flex-shrink", "0"),
            ("flex-grow", "0"),
            ("user-select", "none"),
        ],
        leptos_css: "flex-shrink: 0; flex-grow: 0; user-select: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/truncate/truncate.component.scss",
        selector_path: ".truncate.inline",
        properties: &[("display", "inline-flex")],
        leptos_css: "display: inline-flex;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/truncate/truncate.component.scss",
        selector_path: ".truncate .hidden-content",
        properties: &[
            ("color", "transparent"),
            ("position", "absolute"),
            ("max-width", "300px"),
            ("overflow", "hidden"),
        ],
        leptos_css: "color: transparent; position: absolute; max-width: 300px; overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/truncate/truncate.component.scss",
        selector_path: ".truncate .disabled",
        properties: &[
            ("pointer-events", "none"),
            ("opacity", "0.8"),
            ("color", "var(--transparent-fg)"),
        ],
        leptos_css: "pointer-events: none; opacity: 0.8; color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/truncate/truncate.component.scss",
        selector_path: "@media (max-width: 567px) { .hidden-content }",
        properties: &[("max-width", "150px !important")],
        leptos_css: "max-width: 150px !important;",
        gpui: GpuiOutcome::Paint,
    },
];
