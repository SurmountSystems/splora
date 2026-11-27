use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/about/about-sponsors.component.scss",
        selector_path: "#become-sponsor-container",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "row"),
            ("flex-wrap", "nowrap"),
            ("justify-content", "center"),
            ("align-items", "center"),
            ("gap", "20px"),
            ("margin", "68px auto"),
            ("text-align", "center"),
        ],
        leptos_css: "display: flex; flex-direction: row; flex-wrap: nowrap; justify-content: center; align-items: center; gap: 20px; margin: 68px auto; text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/about/about-sponsors.component.scss",
        selector_path: "#become-sponsor-container.account",
        properties: &[("margin", "20px auto")],
        leptos_css: "margin: 20px auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/about/about-sponsors.component.scss",
        selector_path: ".become-sponsor",
        properties: &[
            ("background-color", "var(--bg)"),
            ("border-radius", "16px"),
            ("padding", "12px 20px"),
            ("width", "400px"),
            ("padding", "40px 20px"),
        ],
        leptos_css: "background-color: var(--bg); border-radius: 16px; padding: 12px 20px; width: 400px; padding: 40px 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/about/about-sponsors.component.scss",
        selector_path: ".become-sponsor a",
        properties: &[("margin-top", "10px")],
        leptos_css: "margin-top: 10px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/about/about-sponsors.component.scss",
        selector_path: "#become-sponsor-container .btn",
        properties: &[("margin-bottom", "24px")],
        leptos_css: "margin-bottom: 24px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/about/about-sponsors.component.scss",
        selector_path: "#become-sponsor-container .ng-fa-icon",
        properties: &[("color", "#2ecc71"), ("margin-right", "5px")],
        leptos_css: "color: #2ecc71; margin-right: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/about/about-sponsors.component.scss",
        selector_path: "#become-sponsor-container .sponsor-feature",
        properties: &[
            ("text-align", "left"),
            ("width", "250px"),
            ("margin", "12px auto"),
            ("white-space", "nowrap"),
        ],
        leptos_css: "text-align: left; width: 250px; margin: 12px auto; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/about/about-sponsors.component.scss",
        selector_path: "@media (max-width: 992px) { #become-sponsor-container }",

        properties: &[("flex-wrap", "wrap")],
        leptos_css: "flex-wrap: wrap;",
        gpui: GpuiOutcome::Paint,
    },
];
