use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/server-health/server-status.component.scss",
        selector_path: ".tomahawk .links",
        properties: &[("text-align", "right"), ("margin-inline-end", "1em")],
        leptos_css: "text-align: right; margin-inline-end: 1em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/server-health/server-status.component.scss",
        selector_path: ".tomahawk .links a, .tomahawk .links span",
        properties: &[("margin-left", "1em")],
        leptos_css: "margin-left: 1em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/server-health/server-status.component.scss",
        selector_path: ".tomahawk .mempoolStatus",
        properties: &[("width", "100%"), ("height", "270px"), ("border", "none")],
        leptos_css: "width: 100%; height: 270px; border: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/server-health/server-status.component.scss",
        selector_path: ".tomahawk .hostLink",
        properties: &[
            ("text-align", "center"),
            ("margin", "auto"),
            ("margin-top", "1em"),
        ],
        leptos_css: "text-align: center; margin: auto; margin-top: 1em;",
        gpui: GpuiOutcome::Paint,
    },
];
