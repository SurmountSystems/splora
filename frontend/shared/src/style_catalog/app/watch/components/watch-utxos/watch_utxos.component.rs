use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/watch/components/watch-utxos/watch-utxos.component.scss",
        selector_path: ".value-cell",
        properties: &[("min-width", "9rem"), ("white-space", "nowrap")],
        leptos_css: "min-width: 9rem; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-utxos/watch-utxos.component.scss",
        selector_path: ".address-cell",
        properties: &[
            ("min-width", "16rem"),
            ("font-family", "monospace"),
            ("overflow-wrap", "anywhere"),
        ],
        leptos_css: "min-width: 16rem; font-family: monospace; overflow-wrap: anywhere;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-utxos/watch-utxos.component.scss",
        selector_path: ".output-cell",
        properties: &[
            ("min-width", "8rem"),
            ("font-family", "monospace"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "min-width: 8rem; font-family: monospace; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/watch/components/watch-utxos/watch-utxos.component.scss",
        selector_path: "a:focus-visible",
        properties: &[
            ("border-radius", "0.2rem"),
            ("box-shadow", "0 0 0 2px var(--info)"),
        ],
        leptos_css: "border-radius: 0.2rem; box-shadow: 0 0 0 2px var(--info);",
        gpui: GpuiOutcome::Paint,
    },
];
