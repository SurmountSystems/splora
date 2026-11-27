use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".import",
        properties: &[("max-width", "720px"), ("margin-bottom", "1.5rem")],
        leptos_css: "max-width: 720px; margin-bottom: 1.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".import-cancel",
        properties: &[
            ("display", "flex"),
            ("justify-content", "flex-end"),
            ("margin-bottom", "0.5rem"),
        ],
        leptos_css: "display: flex; justify-content: flex-end; margin-bottom: 0.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: "textarea.form-control",
        properties: &[
            ("font-family", "monospace"),
            ("font-size", "0.85rem"),
            ("word-break", "break-all"),
        ],
        leptos_css: "font-family: monospace; font-size: 0.85rem; word-break: break-all;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".detected-hint",
        properties: &[("color", "var(--primary)")],
        leptos_css: "color: var(--primary);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".script-type-picker",
        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".script-type-toggle",
        properties: &[
            ("display", "flex"),
            ("align-items", "center"),
            ("gap", "0.5rem"),
            ("width", "100%"),
            ("text-align", "left"),
        ],
        leptos_css: "display: flex; align-items: center; gap: 0.5rem; width: 100%; text-align: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".script-type-toggle::after",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".script-type-menu",
        properties: &[
            ("width", "100%"),
            ("background-color", "var(--secondary)"),
            ("border", "1px solid var(--transparent-fg)"),
            ("box-shadow", "0 0.5rem 1rem var(--box-bg)"),
        ],
        leptos_css: "width: 100%; background-color: var(--secondary); border: 1px solid var(--transparent-fg); box-shadow: 0 0.5rem 1rem var(--box-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".script-type-item",
        properties: &[
            ("display", "flex"),
            ("align-items", "center"),
            ("gap", "0.5rem"),
            ("cursor", "pointer"),
        ],
        leptos_css: "display: flex; align-items: center; gap: 0.5rem; cursor: pointer;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".script-type-item.active",
        properties: &[("color", "var(--fg)"), ("background", "var(--active-bg)")],
        leptos_css: "color: var(--fg); background: var(--active-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".script-type-item:focus-visible",
        properties: &[("box-shadow", "inset 0 0 0 2px var(--info)")],
        leptos_css: "box-shadow: inset 0 0 0 2px var(--info);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".script-type-placeholder",
        properties: &[("color", "var(--transparent-fg)")],
        leptos_css: "color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".script-type-path",
        properties: &[("color", "var(--transparent-fg)"), ("font-size", "0.85em")],
        leptos_css: "color: var(--transparent-fg); font-size: 0.85em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".history-badge",
        properties: &[
            ("margin-left", "auto"),
            ("background-color", "#fff"),
            ("color", "#11131f"),
            ("border", "1px solid rgba(17, 19, 31, 0.25)"),
            ("font-weight", "600"),
        ],
        leptos_css: "margin-left: auto; background-color: #fff; color: #11131f; border: 1px solid rgba(17, 19, 31, 0.25); font-weight: 600;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".script-type-note",
        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-import/watch-import.component.scss",
        selector_path: ".privacy-note",
        properties: &[
            ("margin", "1rem 0 0"),
            ("color", "var(--transparent-fg)"),
            ("font-size", "0.8rem"),
        ],
        leptos_css: "margin: 1rem 0 0; color: var(--transparent-fg); font-size: 0.8rem;",
        gpui: GpuiOutcome::Paint,
    },
];
