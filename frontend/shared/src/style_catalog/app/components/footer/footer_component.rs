use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: ".footer",
        properties: &[
            ("position", "fixed"),
            ("bottom", "0"),
            ("width", "100%"),
            ("height", "60px"),
            ("background-color", "var(--bg)"),
            ("box-shadow", "15px 15px 15px 15px #000"),
            ("z-index", "10"),
        ],
        leptos_css: "position: fixed; bottom: 0; width: 100%; height: 60px; background-color: var(--bg); box-shadow: 15px 15px 15px 15px #000; z-index: 10;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: ".footer.inline-footer",
        properties: &[
            ("position", "relative"),
            ("bottom", "unset"),
            ("top", "-44px"),
        ],
        leptos_css: "position: relative; bottom: unset; top: -44px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: ".sub-text",
        properties: &[
            ("display", "block"),
            ("font-weight", "bold"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "display: block; font-weight: bold; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: ".footer > div",
        properties: &[("margin-top", "3px")],
        leptos_css: "margin-top: 3px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: "@media (min-width: 1200px) { .footer > div }",

        properties: &[("margin-top", "17px")],
        leptos_css: "margin-top: 17px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: "@media (min-width: 1200px) { .sub-text }",

        properties: &[("margin-left", "5px"), ("display", "inline-block")],
        leptos_css: "margin-left: 5px; display: inline-block;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: "@media (max-width: 767.98px) { .extra-text }",

        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: ".mempoolSize",
        properties: &[("color", "var(--title-fg)")],
        leptos_css: "color: var(--title-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: ".info-block",
        properties: &[
            ("float", "left"),
            ("width", "350px"),
            ("line-height", "25px"),
        ],
        leptos_css: "float: left; width: 350px; line-height: 25px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: ".progress",
        properties: &[
            ("display", "inline-flex"),
            ("width", "160px"),
            ("background-color", "var(--secondary)"),
            ("height", "1.1rem"),
        ],
        leptos_css: "display: inline-flex; width: 160px; background-color: var(--secondary); height: 1.1rem;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: ".progress-bar",
        properties: &[("padding", "4px")],
        leptos_css: "padding: 4px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/footer/footer.component.scss",
        selector_path: ".bg-warning",
        properties: &[("background-color", "#b58800 !important")],
        leptos_css: "background-color: #b58800 !important;",
        gpui: GpuiOutcome::Paint,
    },
];
