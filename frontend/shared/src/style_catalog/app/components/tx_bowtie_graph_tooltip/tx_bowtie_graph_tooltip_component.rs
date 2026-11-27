use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/tx-bowtie-graph-tooltip/tx-bowtie-graph-tooltip.component.scss",
        selector_path: ".bowtie-graph-tooltip",
        properties: &[
            ("position", "absolute"),
            ("background", "color-mix(in srgb, #11131f 95%, transparent)"),
            ("border-radius", "4px"),
            ("box-shadow", "1px 1px 10px rgba(0, 0, 0, 0.5)"),
            ("color", "var(--tooltip-grey)"),
            ("padding", "10px 15px"),
            ("text-align", "left"),
            ("pointer-events", "none"),
            ("max-width", "350px"),
        ],
        leptos_css: "position: absolute; background: color-mix(in srgb, #11131f 95%, transparent); border-radius: 4px; box-shadow: 1px 1px 10px rgba(0, 0, 0, 0.5); color: var(--tooltip-grey); padding: 10px 15px; text-align: left; pointer-events: none; max-width: 350px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/tx-bowtie-graph-tooltip/tx-bowtie-graph-tooltip.component.scss",
        selector_path: ".bowtie-graph-tooltip p",
        properties: &[("margin", "0"), ("white-space", "nowrap")],
        leptos_css: "margin: 0; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/tx-bowtie-graph-tooltip/tx-bowtie-graph-tooltip.component.scss",
        selector_path: ".bowtie-graph-tooltip .address",
        properties: &[("width", "100%"), ("max-width", "100%")],
        leptos_css: "width: 100%; max-width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
];
