use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/rbf-list/rbf-list.component.scss",
        selector_path: ".spinner-border",
        properties: &[
            ("height", "25px"),
            ("width", "25px"),
            ("margin-top", "13px"),
        ],
        leptos_css: "height: 25px; width: 25px; margin-top: 13px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/rbf-list/rbf-list.component.scss",
        selector_path: ".rbf-trees .info",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "row"),
            ("justify-content", "space-between"),
            ("align-items", "baseline"),
            ("margin", "0"),
            ("margin-bottom", "0.5em"),
        ],
        leptos_css: "display: flex; flex-direction: row; justify-content: space-between; align-items: baseline; margin: 0; margin-bottom: 0.5em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/rbf-list/rbf-list.component.scss",
        selector_path: ".rbf-trees .info .type .badge",
        properties: &[("margin-left", "0.5em")],
        leptos_css: "margin-left: 0.5em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/rbf-list/rbf-list.component.scss",
        selector_path: ".rbf-trees .tree",
        properties: &[("margin-bottom", "1em")],
        leptos_css: "margin-bottom: 1em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/rbf-list/rbf-list.component.scss",
        selector_path: ".rbf-trees .timeline-wrapper.mined",
        properties: &[("border", "solid 4px var(--success)")],
        leptos_css: "border: solid 4px var(--success);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/rbf-list/rbf-list.component.scss",
        selector_path: ".rbf-trees .no-replacements",
        properties: &[("margin", "1em"), ("text-align", "center")],
        leptos_css: "margin: 1em; text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
];
