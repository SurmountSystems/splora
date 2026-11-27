use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".address-title",
        properties: &[
            ("display", "flex"),
            ("align-items", "baseline"),
            ("justify-content", "space-between"),
            ("gap", "1rem"),
        ],
        leptos_css: "display: flex; align-items: baseline; justify-content: space-between; gap: 1rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".address-summary",
        properties: &[
            ("display", "flex"),
            ("flex-wrap", "wrap"),
            ("justify-content", "flex-end"),
            ("gap", "0.45rem 1rem"),
            ("color", "var(--transparent-fg)"),
            ("font-size", "0.8rem"),
        ],
        leptos_css: "display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 0.45rem 1rem; color: var(--transparent-fg); font-size: 0.8rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".address-summary strong",
        properties: &[
            ("color", "var(--fg)"),
            ("font-variant-numeric", "tabular-nums"),
        ],
        leptos_css: "color: var(--fg); font-variant-numeric: tabular-nums;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".explorer-controls",
        properties: &[
            ("display", "flex"),
            ("flex-wrap", "wrap"),
            ("align-items", "center"),
            ("gap", "0.75rem 1rem"),
            ("margin-bottom", "0.75rem"),
        ],
        leptos_css: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem 1rem; margin-bottom: 0.75rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".explorer-controls .form-select",
        properties: &[("width", "auto")],
        leptos_css: "width: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".address-cell",
        properties: &[("font-family", "monospace"), ("overflow-wrap", "anywhere")],
        leptos_css: "font-family: monospace; overflow-wrap: anywhere;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".used-address .address-cell > a",
        properties: &[("color", "var(--transparent-fg)")],
        leptos_css: "color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".path-cell",
        properties: &[
            ("color", "var(--transparent-fg)"),
            ("font-family", "monospace"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "color: var(--transparent-fg); font-family: monospace; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".balance-cell",
        properties: &[
            ("white-space", "nowrap"),
            ("font-variant-numeric", "tabular-nums"),
        ],
        leptos_css: "white-space: nowrap; font-variant-numeric: tabular-nums;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".zero-balance",
        properties: &[("color", "var(--transparent-fg)")],
        leptos_css: "color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".status-dot",
        properties: &[
            ("display", "inline-block"),
            ("width", "0.6rem"),
            ("height", "0.6rem"),
            ("margin-right", "0.4rem"),
            ("border", "1px solid var(--fg)"),
            ("border-radius", "50%"),
            ("vertical-align", "middle"),
        ],
        leptos_css: "display: inline-block; width: 0.6rem; height: 0.6rem; margin-right: 0.4rem; border: 1px solid var(--fg); border-radius: 50%; vertical-align: middle;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".status-dot.used",
        properties: &[
            ("border-color", "var(--transparent-fg)"),
            ("background", "var(--transparent-fg)"),
        ],
        leptos_css: "border-color: var(--transparent-fg); background: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: ".status-dot.unknown",
        properties: &[
            ("border-style", "dashed"),
            ("border-color", "var(--transparent-fg)"),
        ],
        leptos_css: "border-style: dashed; border-color: var(--transparent-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: "@media (max-width: 767.98px) { .address-title }",
        properties: &[("align-items", "flex-start"), ("flex-direction", "column")],
        leptos_css: "align-items: flex-start; flex-direction: column;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: "@media (max-width: 767.98px) { .address-summary }",
        properties: &[("justify-content", "flex-start")],
        leptos_css: "justify-content: flex-start;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-addresses/watch-addresses.component.scss",
        selector_path: "a:focus-visible",
        properties: &[
            ("border-radius", "0.2rem"),
            ("box-shadow", "0 0 0 2px var(--info)"),
        ],
        leptos_css: "border-radius: 0.2rem; box-shadow: 0 0 0 2px var(--info);",
        gpui: GpuiOutcome::Paint,
    },
];
