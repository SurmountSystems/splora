use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher",
        properties: &[("margin", "1.5rem 0 1rem")],
        leptos_css: "margin: 1.5rem 0 1rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-toggle",
        properties: &[
            ("display", "inline-flex"),
            ("align-items", "center"),
            ("gap", "0.5rem"),
            ("min-width", "16rem"),
            ("max-width", "100%"),
            ("text-align", "left"),
        ],
        leptos_css: "display: inline-flex; align-items: center; gap: 0.5rem; min-width: 16rem; max-width: 100%; text-align: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-toggle .wallet-switcher-label",
        properties: &[("margin-right", "auto")],
        leptos_css: "margin-right: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-menu",
        properties: &[
            ("min-width", "16rem"),
            ("background-color", "var(--secondary)"),
            ("border", "1px solid var(--transparent-fg)"),
            ("box-shadow", "0 0.5rem 1rem var(--box-bg)"),
        ],
        leptos_css: "min-width: 16rem; background-color: var(--secondary); border: 1px solid var(--transparent-fg); box-shadow: 0 0.5rem 1rem var(--box-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-menu.show",
        properties: &[
            ("display", "grid"),
            ("grid-template-columns", "minmax(0, 1fr) 2.5rem"),
        ],
        leptos_css: "display: grid; grid-template-columns: minmax(0, 1fr) 2.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-menu .dropdown-divider, .wallet-switcher-menu .wallet-switcher-add",
        properties: &[("grid-column", "1/-1")],
        leptos_css: "grid-column: 1/-1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-item.active, .wallet-settings-button.active",
        properties: &[("color", "var(--fg)"), ("background", "var(--active-bg)")],
        leptos_css: "color: var(--fg); background: var(--active-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-item:focus-visible, .wallet-switcher-add:focus-visible, .wallet-settings-button:focus-visible",
        properties: &[("box-shadow", "inset 0 0 0 2px var(--info)")],
        leptos_css: "box-shadow: inset 0 0 0 2px var(--info);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-item, .wallet-switcher-add",
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
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-item",
        properties: &[("min-width", "0")],
        leptos_css: "min-width: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-item .check",
        properties: &[
            ("flex", "0 0 1rem"),
            ("color", "var(--primary)"),
            ("font-size", "0.8rem"),
        ],
        leptos_css: "flex: 0 0 1rem; color: var(--primary); font-size: 0.8rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-item .wallet-switcher-label",
        properties: &[("margin-right", "auto")],
        leptos_css: "margin-right: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-settings-button",
        properties: &[
            ("padding", "0"),
            ("color", "var(--transparent-fg)"),
            ("text-align", "center"),
        ],
        leptos_css: "padding: 0; color: var(--transparent-fg); text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-settings-button:hover, .wallet-settings-button:focus-visible",
        properties: &[("color", "var(--fg)")],
        leptos_css: "color: var(--fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-add",
        properties: &[("color", "var(--transparent-fg)")],
        leptos_css: "color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-label",
        properties: &[
            ("overflow", "hidden"),
            ("font-weight", "600"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "overflow: hidden; font-weight: 600; text-overflow: ellipsis; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-switcher/watch-switcher.component.scss",
        selector_path: ".wallet-switcher-fingerprint",
        properties: &[
            ("color", "var(--transparent-fg)"),
            ("font-family", "monospace"),
            ("font-size", "0.72rem"),
            ("letter-spacing", "0.04em"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "color: var(--transparent-fg); font-family: monospace; font-size: 0.72rem; letter-spacing: 0.04em; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
];
