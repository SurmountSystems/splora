use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".frame",
        properties: &[
            ("position", "relative"),
            ("background", "var(--box-bg)"),
            ("padding", "0.5rem"),
            ("height", "calc(100% + 60px)"),
        ],
        leptos_css: "position: relative; background: var(--box-bg); padding: 0.5rem; height: calc(100% + 60px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".heading",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "row"),
            ("justify-content", "space-between"),
            ("align-items", "flex-start"),
        ],
        leptos_css: "display: flex; flex-direction: row; justify-content: space-between; align-items: flex-start;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".heading > *",
        properties: &[("flex-basis", "0"), ("flex-grow", "1")],
        leptos_css: "flex-basis: 0; flex-grow: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".heading h3",
        properties: &[("text-align", "center"), ("margin", "0 1em")],
        leptos_css: "text-align: center; margin: 0 1em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".pagination",
        properties: &[
            ("position", "absolute"),
            ("bottom", "0.5rem"),
            ("right", "0.5rem"),
        ],
        leptos_css: "position: absolute; bottom: 0.5rem; right: 0.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".table",
        properties: &[("margin-top", "0.5em")],
        leptos_css: "margin-top: 0.5em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".table td, .table th",
        properties: &[("padding", "0.15rem 0.5rem")],
        leptos_css: "padding: 0.15rem 0.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".table td.address, .table th.address",
        properties: &[("width", "auto")],
        leptos_css: "width: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".table td.btc, .table th.btc",
        properties: &[("width", "140px"), ("text-align", "right")],
        leptos_css: "width: 140px; text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".table td.fiat, .table th.fiat",
        properties: &[("width", "142px"), ("text-align", "right")],
        leptos_css: "width: 142px; text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".table tr",
        properties: &[("border-collapse", "collapse")],
        leptos_css: "border-collapse: collapse;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".table tr:first-child",
        properties: &[("border-bottom", "solid 1px white")],
        leptos_css: "border-bottom: solid 1px white;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".table tr:first-child td, .table tr:first-child th",
        properties: &[("padding-bottom", "0.3rem")],
        leptos_css: "padding-bottom: 0.3rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".table tr:nth-child(2) td, .table tr:nth-child(2) th",
        properties: &[("padding-top", "0.3rem")],
        leptos_css: "padding-top: 0.3rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: ".table tr:nth-child(even)",
        properties: &[("background", "var(--stat-box-bg)")],
        leptos_css: "background: var(--stat-box-bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: "@media (min-width: 528px) { .table td.btc, .table th.btc }",

        properties: &[("width", "160px")],
        leptos_css: "width: 160px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: "@media (min-width: 528px) { .table td.fiat, .table th.fiat }",

        properties: &[("width", "140px")],
        leptos_css: "width: 140px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: "@media (min-width: 576px) { .table td.btc, .table th.btc }",

        properties: &[("width", "170px")],
        leptos_css: "width: 170px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: "@media (min-width: 576px) { .table td.fiat, .table th.fiat }",

        properties: &[("width", "140px")],
        leptos_css: "width: 140px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: "@media (min-width: 992px) { .table td.btc, .table th.btc }",

        properties: &[("width", "210px")],
        leptos_css: "width: 210px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/address-group/address-group.component.scss",
        selector_path: "@media (min-width: 992px) { .table td.fiat, .table th.fiat }",

        properties: &[("width", "140px")],
        leptos_css: "width: 140px;",
        gpui: GpuiOutcome::Paint,
    },
];
