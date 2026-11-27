use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container",
        properties: &[
            ("display", "flex"),
            ("justify-content", "space-between"),
            ("margin-bottom", "10px"),
        ],
        leptos_css: "display: flex; justify-content: space-between; margin-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: "@media (min-width: 376px) { .fee-estimation-container }",

        properties: &[("flex-direction", "row")],
        leptos_css: "flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container .item",
        properties: &[
            ("max-width", "300px"),
            ("margin", "0"),
            ("width", "-webkit-fill-available"),
        ],
        leptos_css: "max-width: 300px; margin: 0; width: -webkit-fill-available;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: "@media (min-width: 376px) { .fee-estimation-container .item }",

        properties: &[("margin", "0 auto 0px")],
        leptos_css: "margin: 0 auto 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container .item .card-title",
        properties: &[
            ("margin", "0"),
            ("color", "var(--title-fg)"),
            ("font-size", "10px"),
            ("font-size", "1rem"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "margin: 0; color: var(--title-fg); font-size: 10px; font-size: 1rem; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container .item .card-text",
        properties: &[("font-size", "22px"), ("text-align", "center")],
        leptos_css: "font-size: 22px; text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container .item .card-text span",
        properties: &[
            ("font-size", "11px"),
            ("position", "relative"),
            ("top", "-2px"),
        ],
        leptos_css: "font-size: 11px; position: relative; top: -2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container .item .card-text span",
        properties: &[
            ("color", "var(--transparent-fg)"),
            ("font-size", "12px"),
            ("top", "0px"),
        ],
        leptos_css: "color: var(--transparent-fg); font-size: 12px; top: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container .item .fee-text",
        properties: &[
            ("border-bottom", "1px solid var(--border-subtle)"),
            ("width", "fit-content"),
            ("margin", "auto"),
            ("line-height", "1.45"),
            ("padding", "0px 2px"),
        ],
        leptos_css: "border-bottom: 1px solid var(--border-subtle); width: fit-content; margin: auto; line-height: 1.45; padding: 0px 2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container .item .fiat",
        properties: &[("display", "block"), ("font-size", "14px !important")],
        leptos_css: "display: block; font-size: 14px !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container.volume-header",
        properties: &[("justify-content", "center")],
        leptos_css: "justify-content: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container.volume-header .item",
        properties: &[("margin", "0"), ("max-width", "none"), ("width", "auto")],
        leptos_css: "margin: 0; max-width: none; width: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".fee-estimation-container.volume-header h5",
        properties: &[("margin", "0")],
        leptos_css: "margin: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".card-text .skeleton-loader",
        properties: &[("width", "100%"), ("display", "block")],
        leptos_css: "width: 100%; display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".card-text .skeleton-loader:first-child",
        properties: &[("max-width", "90px"), ("margin", "15px auto 3px")],
        leptos_css: "max-width: 90px; margin: 15px auto 3px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".card-text .skeleton-loader:last-child",
        properties: &[("margin", "10px auto 3px"), ("max-width", "55px")],
        leptos_css: "margin: 10px auto 3px; max-width: 55px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".title-link, .title-link:hover, .title-link:focus, .title-link:active",
        properties: &[
            ("display", "block"),
            ("margin-bottom", "4px"),
            ("text-decoration", "none"),
            ("color", "inherit"),
        ],
        leptos_css: "display: block; margin-bottom: 4px; text-decoration: none; color: inherit;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".credit",
        properties: &[("color", "var(--green)")],
        leptos_css: "color: var(--green);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/liquid-reserves-audit/recent-pegs-stats/recent-pegs-stats.component.scss",
        selector_path: ".debit",
        properties: &[("color", "var(--red)")],
        leptos_css: "color: var(--red);",
        gpui: GpuiOutcome::Paint,
    },
];
