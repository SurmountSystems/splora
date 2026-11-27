use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: ":host ::ng-deep .dropdown-item",
        properties: &[("white-space", "nowrap")],
        leptos_css: "white-space: nowrap;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: ":host ::ng-deep .dropdown-menu",
        properties: &[("width", "calc(100% - 34px)")],
        leptos_css: "width: calc(100% - 34px);",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: "@media (min-width: 768px) { :host ::ng-deep .dropdown-menu }",
        properties: &[("width", "410px")],
        leptos_css: "width: 410px;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: "form",
        properties: &[("margin-top", "5px")],
        leptos_css: "margin-top: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: "@media (min-width: 564px) { form }",
        properties: &[("margin-right", "-5px")],
        leptos_css: "margin-right: -5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: "@media (min-width: 992px) { form }",
        properties: &[("width", "100%")],
        leptos_css: "width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: "@media (max-width: 613px) { form.hamburgerOpen }",
        properties: &[("margin-left", "0px"), ("margin-right", "0px")],
        leptos_css: "margin-left: 0px; margin-right: 0px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: ".search-submit",
        properties: &[("width", "62px"), ("color", "#fff")],
        leptos_css: "width: 62px; color: #fff;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: ".search-box-container",
        properties: &[("position", "relative"), ("width", "100%")],
        leptos_css: "position: relative; width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: "@media (min-width: 768px) { .search-box-container }",
        properties: &[("min-width", "300px")],
        leptos_css: "min-width: 300px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: "@media (min-width: 992px) { .search-box-container }",
        properties: &[("min-width", "142px")],
        leptos_css: "min-width: 142px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: "@media (min-width: 1200px) { .search-box-container }",
        properties: &[("min-width", "300px")],
        leptos_css: "min-width: 300px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: ".search-box-container .btn",
        properties: &[("width", "100px")],
        leptos_css: "width: 100px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: ".spinner-border",
        properties: &[
            ("vertical-align", "text-top"),
            ("margin-top", "1px"),
            ("margin-right", "2px"),
        ],
        leptos_css: "vertical-align: text-top; margin-top: 1px; margin-right: 2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/search-form/search-form.component.scss",
        selector_path: "input:focus",
        properties: &[("box-shadow", "none"), ("border-color", "transparent")],
        leptos_css: "box-shadow: none; border-color: transparent;",
        gpui: GpuiOutcome::Paint,
    },
];
