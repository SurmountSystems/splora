use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
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
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".widget table",
        properties: &[("overflow-anchor", "none")],
        leptos_css: "overflow-anchor: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "tr, td, th",
        properties: &[
            ("border", "0px"),
            ("padding-top", "0.65rem"),
            ("padding-bottom", "0.6rem"),
            ("padding-right", "2rem"),
        ],
        leptos_css: "border: 0px; padding-top: 0.65rem; padding-bottom: 0.6rem; padding-right: 2rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".widget tr.widget, .widget td.widget, .widget th.widget",
        properties: &[("padding-right", "1rem")],
        leptos_css: "padding-right: 1rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@media (max-width: 510px) { .widget tr.widget, .widget td.widget, .widget th.widget }",

        properties: &[("padding-right", "0.5rem")],
        leptos_css: "padding-right: 0.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".clear-link",
        properties: &[("color", "white")],
        leptos_css: "color: white;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".disabled",
        properties: &[("pointer-events", "none"), ("opacity", "0.5")],
        leptos_css: "pointer-events: none; opacity: 0.5;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".progress",
        properties: &[("background-color", "var(--secondary)")],
        leptos_css: "background-color: var(--secondary);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".transaction",
        properties: &[
            ("width", "65%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "120px"),
        ],
        leptos_css: "width: 65%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 120px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".transaction.widget",
        properties: &[("width", "100%")],
        leptos_css: "width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".address",
        properties: &[
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "160px"),
        ],
        leptos_css: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 160px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@media (max-width: 527px) { .address }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".amount",
        properties: &[("width", "0%")],
        leptos_css: "width: 0%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".output",
        properties: &[
            ("width", "50%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "160px"),
        ],
        leptos_css: "width: 50%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 160px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@media (max-width: 800px) { .output }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".address",
        properties: &[
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "160px"),
        ],
        leptos_css: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 160px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@media (max-width: 960px) { .address }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".timestamp",
        properties: &[("width", "0%")],
        leptos_css: "width: 0%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@media (max-width: 650px) { .timestamp }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@media (max-width: 1000px) { .timestamp .relative-time }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@media (min-width: 768px) and (max-width: 1050px) { .timestamp.widget }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@media (max-width: 767px) { .timestamp.widget }",

        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@media (max-width: 510px) { .timestamp.widget }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".credit",
        properties: &[("color", "var(--green)")],
        leptos_css: "color: var(--green);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".debit",
        properties: &[("color", "var(--red)")],
        leptos_css: "color: var(--red);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: ".glow-effect",
        properties: &[(
            "animation",
            "color-oscillation 1s ease-in-out infinite alternate",
        )],
        leptos_css: "animation: color-oscillation 1s ease-in-out infinite alternate;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@keyframes color-oscillation 0%",
        properties: &[("color", "#777983")],
        leptos_css: "color: #777983;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-list/recent-pegs-list.component.scss",
        selector_path: "@keyframes color-oscillation 100%",
        properties: &[("color", "var(--red)")],
        leptos_css: "color: var(--red);",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
];
