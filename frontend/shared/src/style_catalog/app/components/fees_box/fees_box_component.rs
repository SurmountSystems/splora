use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".card-title",
        properties: &[
            ("color", "var(--title-fg)"),
            ("font-size", "10px"),
            ("margin-bottom", "4px"),
            ("font-size", "1rem"),
        ],
        leptos_css: "color: var(--title-fg); font-size: 10px; margin-bottom: 4px; font-size: 1rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".card-text",
        properties: &[("font-size", "22px")],
        leptos_css: "font-size: 22px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".card-text span",
        properties: &[
            ("font-size", "11px"),
            ("position", "relative"),
            ("top", "-2px"),
            ("display", "inline-flex"),
        ],
        leptos_css: "font-size: 11px; position: relative; top: -2px; display: inline-flex;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".card-text .green-color",
        properties: &[("display", "block")],
        leptos_css: "display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".fee-estimation-container",
        properties: &[
            ("display", "flex"),
            ("justify-content", "space-between"),
            ("flex-direction", "row"),
        ],
        leptos_css: "display: flex; justify-content: space-between; flex-direction: row;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".fee-estimation-container .item",
        properties: &[
            ("width", "100px"),
            ("margin", "0"),
            ("width", "-webkit-fill-available"),
            ("margin", "0 auto 0px"),
        ],
        leptos_css: "width: 100px; margin: 0; width: -webkit-fill-available; margin: 0 auto 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: "@media (767px < width < 992px), (width < 576px) { .fee-estimation-container .item:first-child }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".fee-estimation-container .item:last-child",
        properties: &[("margin-bottom", "0")],
        leptos_css: "margin-bottom: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
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
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".fee-estimation-container .item .fee-text",
        properties: &[
            ("border-bottom", "1px solid var(--border-subtle)"),
            ("width", "fit-content"),
            ("margin", "auto"),
            ("font-size", "20px"),
        ],
        leptos_css: "border-bottom: 1px solid var(--border-subtle); width: fit-content; margin: auto; font-size: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".fee-estimation-container .item .fiat",
        properties: &[("display", "block"), ("font-size", "13px !important")],
        leptos_css: "display: block; font-size: 13px !important;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".loading-container",
        properties: &[("height", "50px")],
        leptos_css: "height: 50px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".card-text .skeleton-loader",
        properties: &[("width", "100%"), ("display", "block")],
        leptos_css: "width: 100%; display: block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".card-text .skeleton-loader:first-child",
        properties: &[("max-width", "70px"), ("margin", "10px auto 3px")],
        leptos_css: "max-width: 70px; margin: 10px auto 3px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".card-text .skeleton-loader:last-child",
        properties: &[("margin", "10px auto 3px"), ("max-width", "55px")],
        leptos_css: "margin: 10px auto 3px; max-width: 55px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".fee-progress-bar",
        properties: &[
            ("width", "25%"),
            ("height", "22px"),
            ("margin-bottom", "12px"),
            ("display", "flex"),
            ("flex-direction", "row"),
            ("transition", "background-color 1s"),
            ("color", "var(--fg)"),
        ],
        leptos_css: "width: 25%; height: 22px; margin-bottom: 12px; display: flex; flex-direction: row; transition: background-color 1s; color: var(--fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".fee-progress-bar.priority",
        properties: &[("width", "75%"), ("border-radius", "0px 10px 10px 0px")],
        leptos_css: "width: 75%; border-radius: 0px 10px 10px 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: "@media (767px < width < 992px), (width < 576px) { .fee-progress-bar.priority }",

        properties: &[("width", "100%")],
        leptos_css: "width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: "@media (767px < width < 992px), (width < 576px) { .fee-progress-bar:first-child }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".band-separator",
        properties: &[("width", "5%")],
        leptos_css: "width: 5%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: "@media (767px < width < 992px), (width < 576px) { .band-separator }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".band-separator.fill",
        properties: &[
            ("height", "22px"),
            (
                "background",
                "repeating-linear-gradient(90deg, var(--secondary), var(--secondary) 2px, var(--bg) 2px, var(--bg) 4px)",
            ),
        ],
        leptos_css: "height: 22px; background: repeating-linear-gradient(90deg, var(--secondary), var(--secondary) 2px, var(--bg) 2px, var(--bg) 4px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".fee-label",
        properties: &[
            ("padding-top", "2px"),
            ("font-size", "12px"),
            ("width", "100%"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("padding-left", "5px"),
            ("padding-right", "5px"),
        ],
        leptos_css: "padding-top: 2px; font-size: 12px; width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; padding-left: 5px; padding-right: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: "@media (767px < width < 992px), (width < 576px) { .fee-label }",

        properties: &[("width", "33%")],
        leptos_css: "width: 33%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/fees-box/fees-box.component.scss",
        selector_path: ".fee-label.prority",
        properties: &[("width", "33%")],
        leptos_css: "width: 33%;",
        gpui: GpuiOutcome::Paint,
    },
];
