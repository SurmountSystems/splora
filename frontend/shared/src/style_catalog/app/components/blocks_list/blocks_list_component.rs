use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".spinner-border",
        properties: &[
            ("height", "25px"),
            ("width", "25px"),
            ("margin-top", "-10px"),
            ("margin-left", "-13px"),
            ("flex-shrink", "0"),
        ],
        leptos_css: "height: 25px; width: 25px; margin-top: -10px; margin-left: -13px; flex-shrink: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".container-xl",
        properties: &[("max-width", "1400px")],
        leptos_css: "max-width: 1400px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".container-xl.widget",
        properties: &[("padding-left", "0px"), ("padding-bottom", "0px")],
        leptos_css: "padding-left: 0px; padding-bottom: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".container-xl.legacy",
        properties: &[("max-width", "1140px")],
        leptos_css: "max-width: 1140px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".container",
        properties: &[("max-width", "100%")],
        leptos_css: "max-width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "tr, td, th",
        properties: &[
            ("border", "0px"),
            ("padding-top", "0.65rem !important"),
            ("padding-bottom", "0.7rem !important"),
        ],
        leptos_css: "border: 0px; padding-top: 0.65rem !important; padding-bottom: 0.7rem !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "tr .difference, td .difference, th .difference",
        properties: &[("margin-left", "0.5em")],
        leptos_css: "margin-left: 0.5em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "tr .difference.positive, td .difference.positive, th .difference.positive",
        properties: &[("color", "rgb(66, 183, 71)")],
        leptos_css: "color: rgb(66, 183, 71);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "tr .difference.negative, td .difference.negative, th .difference.negative",
        properties: &[("color", "rgb(183, 66, 66)")],
        leptos_css: "color: rgb(183, 66, 66);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".clear-link",
        properties: &[("color", "var(--fg)")],
        leptos_css: "color: var(--fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".disabled",
        properties: &[("pointer-events", "none"), ("opacity", "0.5")],
        leptos_css: "pointer-events: none; opacity: 0.5;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".progress",
        properties: &[("background-color", "var(--secondary)")],
        leptos_css: "background-color: var(--secondary);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".pool",
        properties: &[
            ("width", "17%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "160px"),
        ],
        leptos_css: "width: 17%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 160px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 576px) { .pool }",

        properties: &[("width", "34%")],
        leptos_css: "width: 34%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".pool.widget",
        properties: &[("width", "40%"), ("padding-left", "24px")],
        leptos_css: "width: 40%; padding-left: 24px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (min-width: 768px) and (max-width: 926px) { .pool.widget }",

        properties: &[("padding-left", "0px"), ("width", "60%")],
        leptos_css: "padding-left: 0px; width: 60%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 430px) { .pool.widget }",

        properties: &[("padding-left", "0px"), ("width", "60%")],
        leptos_css: "padding-left: 0px; width: 60%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".height",
        properties: &[("width", "8%")],
        leptos_css: "width: 8%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".height.widget",
        properties: &[("width", "15%")],
        leptos_css: "width: 15%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 576px) { .height.widget }",

        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (min-width: 768px) and (max-width: 926px) { .height.widget }",

        properties: &[("width", "30%")],
        leptos_css: "width: 30%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 430px) { .height.widget }",

        properties: &[("width", "30%")],
        leptos_css: "width: 30%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".height.legacy",
        properties: &[("width", "15%")],
        leptos_css: "width: 15%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".timestamp",
        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 1100px) { .timestamp }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".timestamp.legacy",
        properties: &[("width", "20%")],
        leptos_css: "width: 20%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 1100px) { .timestamp.legacy }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 850px) { .timestamp.legacy }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".mined",
        properties: &[("width", "13%")],
        leptos_css: "width: 13%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 730px) { .mined }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".mined.legacy",
        properties: &[("width", "15%")],
        leptos_css: "width: 15%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 1000px) { .mined.legacy }",

        properties: &[("padding-right", "20px"), ("width", "20%")],
        leptos_css: "padding-right: 20px; width: 20%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 576px) { .mined.legacy }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".txs",
        properties: &[
            ("padding-right", "20px"),
            ("width", "6%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "100px"),
        ],
        leptos_css: "padding-right: 20px; width: 6%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 1100px) { .txs }",

        properties: &[("padding-right", "10px")],
        leptos_css: "padding-right: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 875px) { .txs }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".txs.widget",
        properties: &[("padding-right", "0"), ("display", "none")],
        leptos_css: "padding-right: 0; display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 650px) { .txs.widget }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".txs.legacy",
        properties: &[("width", "18%"), ("display", "table-cell")],
        leptos_css: "width: 18%; display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 1000px) { .txs.legacy }",

        properties: &[("padding-right", "20px")],
        leptos_css: "padding-right: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".fees",
        properties: &[("width", "8%")],
        leptos_css: "width: 8%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 820px) { .fees }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".fees.widget",
        properties: &[("width", "20%")],
        leptos_css: "width: 20%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".fee-delta",
        properties: &[("width", "6%"), ("padding-left", "0")],
        leptos_css: "width: 6%; padding-left: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 991px) { .fee-delta }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".fee-delta.widget",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".reward",
        properties: &[
            ("width", "8%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "130px"),
        ],
        leptos_css: "width: 8%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 130px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 576px) { .reward }",

        properties: &[("width", "7%"), ("padding-right", "30px")],
        leptos_css: "width: 7%; padding-right: 30px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".reward.widget",
        properties: &[("width", "20%")],
        leptos_css: "width: 20%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 576px) { .reward.widget }",

        properties: &[("width", "30%"), ("padding-right", "0")],
        leptos_css: "width: 30%; padding-right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (min-width: 768px) and (max-width: 926px) { .reward.widget }",

        properties: &[
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("max-width", "90px"),
        ],
        leptos_css: "overflow: hidden; text-overflow: ellipsis; max-width: 90px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 430px) { .reward.widget }",

        properties: &[
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("max-width", "90px"),
        ],
        leptos_css: "overflow: hidden; text-overflow: ellipsis; max-width: 90px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".size",
        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 1000px) { .size }",

        properties: &[("width", "13%")],
        leptos_css: "width: 13%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 950px) { .size }",

        properties: &[("width", "15%")],
        leptos_css: "width: 15%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 650px) { .size }",

        properties: &[("width", "20%")],
        leptos_css: "width: 20%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 450px) { .size }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".size.legacy",
        properties: &[("width", "30%")],
        leptos_css: "width: 30%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 576px) { .size.legacy }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".health",
        properties: &[("width", "10%")],
        leptos_css: "width: 10%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 1100px) { .health }",

        properties: &[("width", "13%")],
        leptos_css: "width: 13%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 560px) { .health }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".health .progress-text .skeleton-loader",
        properties: &[("top", "-8.5px")],
        leptos_css: "top: -8.5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".health.widget",
        properties: &[("width", "25%")],
        leptos_css: "width: 25%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 1105px) { .health.widget }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 767px) { .health.widget }",

        properties: &[("display", "table-cell")],
        leptos_css: "display: table-cell;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: "@media (max-width: 500px) { .health.widget }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".tooltip-custom",
        properties: &[("position", "relative")],
        leptos_css: "position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".tooltip-custom .tooltiptext",
        properties: &[
            ("visibility", "hidden"),
            ("color", "var(--fg)"),
            ("text-align", "center"),
            ("padding", "5px 0"),
            ("border-radius", "6px"),
            ("position", "absolute"),
            ("z-index", "1"),
            ("top", "-40px"),
            ("left", "0"),
        ],
        leptos_css: "visibility: hidden; color: var(--fg); text-align: center; padding: 5px 0; border-radius: 6px; position: absolute; z-index: 1; top: -40px; left: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".tooltip-custom:hover .tooltiptext",
        properties: &[("visibility", "visible")],
        leptos_css: "visibility: visible;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/blocks-list/blocks-list.component.scss",
        selector_path: ".scriptmessage",
        properties: &[
            ("overflow", "hidden"),
            ("display", "inline-block"),
            ("text-overflow", "ellipsis"),
            ("vertical-align", "middle"),
            ("max-width", "50vw"),
            ("text-align", "left"),
        ],
        leptos_css: "overflow: hidden; display: inline-block; text-overflow: ellipsis; vertical-align: middle; max-width: 50vw; text-align: left;",
        gpui: GpuiOutcome::Paint,
    },
];
