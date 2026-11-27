use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".title-block",
        properties: &[("flex-wrap", "wrap"), ("align-items", "baseline")],
        leptos_css: "flex-wrap: wrap; align-items: baseline;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (min-width: 650px) { .title-block }",

        properties: &[("flex-direction", "row")],
        leptos_css: "flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".title-block h1",
        properties: &[
            ("margin", "0rem"),
            ("margin-right", "15px"),
            ("line-height", "1"),
        ],
        leptos_css: "margin: 0rem; margin-right: 15px; line-height: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".td-width",
        properties: &[("width", "150px")],
        leptos_css: "width: 150px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 768px) { .td-width }",

        properties: &[("width", "175px")],
        leptos_css: "width: 175px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".badge",
        properties: &[("position", "relative"), ("top", "-1px")],
        leptos_css: "position: relative; top: -1px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".miner-name",
        properties: &[("margin-right", "4px"), ("vertical-align", "top")],
        leptos_css: "margin-right: 4px; vertical-align: top;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".badge.badge-accelerated",
        properties: &[("background-color", "var(--tertiary)"), ("color", "white")],
        leptos_css: "background-color: var(--tertiary); color: white;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".btn-small-height",
        properties: &[("line-height", "1")],
        leptos_css: "line-height: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".row",
        properties: &[("flex-direction", "column")],
        leptos_css: "flex-direction: column;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (min-width: 850px) { .row }",

        properties: &[("flex-direction", "row")],
        leptos_css: "flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".box.hidden",
        properties: &[
            ("visibility", "hidden"),
            ("height", "0px"),
            ("padding-top", "0px"),
            ("padding-bottom", "0px"),
            ("margin-top", "0px"),
            ("margin-bottom", "0px"),
        ],
        leptos_css: "visibility: hidden; height: 0px; padding-top: 0px; padding-bottom: 0px; margin-top: 0px; margin-bottom: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 767.98px) { .mobile-bottomcol }",

        properties: &[("margin-top", "15px")],
        leptos_css: "margin-top: 15px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 767.98px) { .details-table td:first-child }",

        properties: &[("white-space", "pre-wrap")],
        leptos_css: "white-space: pre-wrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".fiat",
        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (min-width: 768px) { .fiat }",

        properties: &[
            ("display", "inline-block"),
            ("margin-left", "15px"),
            ("text-align", "left"),
        ],
        leptos_css: "display: inline-block; margin-left: 15px; text-align: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".table tr td",
        properties: &[("padding", "0.75rem 0.5rem")],
        leptos_css: "padding: 0.75rem 0.5rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (min-width: 576px) { .table tr td }",

        properties: &[("padding", "0.75rem 0.75rem")],
        leptos_css: "padding: 0.75rem 0.75rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".table tr td:last-child",
        properties: &[("text-align", "right")],
        leptos_css: "text-align: right;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (min-width: 850px) { .table tr td:last-child }",

        properties: &[("text-align", "left")],
        leptos_css: "text-align: left;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".table tr td .btn",
        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".table tr td.wrap-cell",
        properties: &[("white-space", "normal")],
        leptos_css: "white-space: normal;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".effective-fee-container",
        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (min-width: 768px) { .effective-fee-container }",

        properties: &[("display", "inline-block")],
        leptos_css: "display: inline-block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 425px) { .effective-fee-container }",

        properties: &[("display", "flex"), ("flex-direction", "column")],
        leptos_css: "display: flex; flex-direction: column;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 767px) { .hide-on-mobile }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 767px) { .effective-fee-rating }",

        properties: &[("margin-right", "0px !important")],
        leptos_css: "margin-right: 0px !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".btn-outline-info",
        properties: &[("margin-top", "5px")],
        leptos_css: "margin-top: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (min-width: 768px) { .btn-outline-info }",

        properties: &[("margin-top", "0px")],
        leptos_css: "margin-top: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".eta",
        properties: &[
            ("display", "flex"),
            ("flex-wrap", "wrap"),
            ("align-content", "center"),
        ],
        leptos_css: "display: flex; flex-wrap: wrap; align-content: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (min-width: 850px) { .eta }",

        properties: &[("justify-content", "left !important")],
        leptos_css: "justify-content: left !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (min-width: 850px) { .accelerate }",

        properties: &[("margin-left", "auto")],
        leptos_css: "margin-left: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".etaDeepMempool",
        properties: &[("flex-wrap", "wrap")],
        leptos_css: "flex-wrap: wrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 849px) { .etaDeepMempool }",

        properties: &[("justify-content", "right !important")],
        leptos_css: "justify-content: right !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".accelerateDeepMempool",
        properties: &[("margin-left", "5px")],
        leptos_css: "margin-left: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".goggles-icon",
        properties: &[("display", "block"), ("width", "2.7em")],
        leptos_css: "display: block; width: 2.7em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".oobFees",
        properties: &[("color", "#905cf4")],
        leptos_css: "color: #905cf4;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".disabled",
        properties: &[("opacity", "0.5"), ("pointer-events", "none")],
        leptos_css: "opacity: 0.5; pointer-events: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".effective-fee-row.has-cluster-preview",
        properties: &[
            ("display", "flex"),
            ("align-items", "center"),
            ("gap", "0.5rem"),
            ("flex-wrap", "wrap"),
        ],
        leptos_css: "display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 849.98px) { .effective-fee-row.has-cluster-preview }",

        properties: &[("justify-content", "flex-end")],
        leptos_css: "justify-content: flex-end;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".effective-fee-row.has-cluster-preview .effective-fee-container",
        properties: &[("flex", "0 0 auto")],
        leptos_css: "flex: 0 0 auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 425px) { .effective-fee-row.has-cluster-preview .effective-fee-container }",

        properties: &[("align-items", "flex-end")],
        leptos_css: "align-items: flex-end;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".cluster-preview-inline",
        properties: &[
            ("position", "relative"),
            ("flex", "1 1 120px"),
            ("min-width", "120px"),
            ("max-width", "300px"),
            ("display", "flex"),
            ("align-items", "center"),
            ("overflow", "visible"),
            ("margin", "-0.75rem -0.75rem -0.75rem auto"),
            ("cursor", "pointer"),
        ],
        leptos_css: "position: relative; flex: 1 1 120px; min-width: 120px; max-width: 300px; display: flex; align-items: center; overflow: visible; margin: -0.75rem -0.75rem -0.75rem auto; cursor: pointer;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".cluster-preview-inline .cluster-preview-diagram",
        properties: &[
            ("display", "block"),
            ("width", "100%"),
            ("--cluster-preview-cursor", "pointer"),
        ],
        leptos_css: "display: block; width: 100%; --cluster-preview-cursor: pointer;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".cluster-preview-inline .cluster-preview-expand",
        properties: &[
            ("position", "absolute"),
            ("top", "4px"),
            ("right", "4px"),
            ("color", "var(--info)"),
            ("font-size", "0.85em"),
            ("opacity", "0"),
            ("transition", "opacity 150ms ease"),
            ("pointer-events", "none"),
            ("z-index", "1"),
        ],
        leptos_css: "position: absolute; top: 4px; right: 4px; color: var(--info); font-size: 0.85em; opacity: 0; transition: opacity 150ms ease; pointer-events: none; z-index: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".cluster-preview-inline:focus-visible",
        properties: &[("outline", "none")],
        leptos_css: "outline: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".cluster-preview-inline:hover .cluster-preview-expand, .cluster-preview-inline:focus-visible .cluster-preview-expand",
        properties: &[("opacity", "1")],
        leptos_css: "opacity: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".cluster-preview-inline:hover .cluster-preview-diagram, .cluster-preview-inline:focus-visible .cluster-preview-diagram",
        properties: &[(
            "--cluster-preview-shadow",
            "0 0 0 1px var(--info), 0 0 8px color-mix(in srgb, var(--info) 35%, transparent)",
        )],
        leptos_css: "--cluster-preview-shadow: 0 0 0 1px var(--info), 0 0 8px color-mix(in srgb, var(--info) 35%, transparent);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: ".cluster-preview-table-row",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 849.98px) and (min-width: 500px) { .cluster-preview-inline }",

        properties: &[("order", "-1"), ("margin-right", "0")],
        leptos_css: "order: -1; margin-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 499.98px) { .effective-fee-row.has-cluster-preview .cluster-preview-inline }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 499.98px) { .cluster-preview-table-row }",

        properties: &[("display", "table-row")],
        leptos_css: "display: table-row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 499.98px) { .cluster-preview-table-row td }",

        properties: &[("padding-top", "0")],
        leptos_css: "padding-top: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "@media (max-width: 499.98px) { .cluster-preview-table-row .cluster-preview-inline }",

        properties: &[
            ("width", "100%"),
            ("flex-basis", "100%"),
            ("max-width", "100%"),
            ("margin", "-0.25rem 0 -0.75rem"),
        ],
        leptos_css: "width: 100%; flex-basis: 100%; max-width: 100%; margin: -0.25rem 0 -0.75rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "::ng-deep .cluster-preview-ngb-tooltip .tooltip-inner",
        properties: &[
            ("text-align", "left"),
            ("max-width", "320px"),
            ("padding", "10px 12px"),
        ],
        leptos_css: "text-align: left; max-width: 320px; padding: 10px 12px;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "::ng-deep .cluster-preview-ngb-tooltip .tooltip-inner p",
        properties: &[("margin", "0 0 6px"), ("line-height", "1.35")],
        leptos_css: "margin: 0 0 6px; line-height: 1.35;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "::ng-deep .cluster-preview-ngb-tooltip .tooltip-inner p:last-child",
        properties: &[("margin-bottom", "0")],
        leptos_css: "margin-bottom: 0;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/transaction/transaction-details/transaction-details.component.scss",
        selector_path: "::ng-deep .cluster-preview-ngb-tooltip .tooltip-inner p.hint",
        properties: &[
            ("margin-top", "8px"),
            ("opacity", "0.8"),
            ("font-style", "italic"),
        ],
        leptos_css: "margin-top: 8px; opacity: 0.8; font-style: italic;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
];
