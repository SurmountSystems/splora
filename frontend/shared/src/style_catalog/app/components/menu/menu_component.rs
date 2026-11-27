use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".sidenav",
        properties: &[
            ("z-index", "10"),
            ("background-color", "transparent"),
            ("width", "225px"),
            ("height", "calc(100vh - 65px)"),
            ("position", "absolute"),
            ("top", "65px"),
            ("transition", "0.25s"),
            ("box-shadow", "5px 0px 30px 0px #000"),
            ("padding-bottom", "20px"),
        ],
        leptos_css: "z-index: 10; background-color: transparent; width: 225px; height: calc(100vh - 65px); position: absolute; top: 65px; transition: 0.25s; box-shadow: 5px 0px 30px 0px #000; padding-bottom: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: "@media (max-width: 613px) { .sidenav }",

        properties: &[("top", "105px")],
        leptos_css: "top: 105px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ":host-context(.ltr-layout) .sidenav",
        properties: &[("left", "-250px")],
        leptos_css: "left: -250px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ":host-context(.rtl-layout) .sidenav",
        properties: &[("right", "-250px")],
        leptos_css: "right: -250px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".ellipsis",
        properties: &[
            ("display", "block"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
        ],
        leptos_css: "display: block; overflow: hidden; text-overflow: ellipsis;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".scrollable",
        properties: &[("overflow-x", "hidden"), ("overflow-y", "auto")],
        leptos_css: "overflow-x: hidden; overflow-y: auto;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".sidenav.open",
        properties: &[("display", "block"), ("background-color", "var(--bg)")],
        leptos_css: "display: block; background-color: var(--bg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ":host-context(.ltr-layout) .sidenav.open",
        properties: &[("left", "0")],
        leptos_css: "left: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ":host-context(.rtl-layout) .sidenav.open",
        properties: &[("right", "0")],
        leptos_css: "right: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".sidenav a, .sidenav button",
        properties: &[
            ("text-decoration", "none"),
            ("color", "var(--fg)"),
            ("margin-left", "20px"),
        ],
        leptos_css: "text-decoration: none; color: var(--fg); margin-left: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".sidenav a:hover, .sidenav button:hover, .sidenav button:focus-visible",
        properties: &[("color", "var(--fg)")],
        leptos_css: "color: var(--fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".sidenav nav",
        properties: &[
            ("width", "100%"),
            ("height", "calc(100vh - 65px)"),
            ("background-color", "var(--bg)"),
            ("padding-left", "20px"),
            ("padding-right", "20px"),
            ("padding-top", "20px"),
            ("padding-bottom", "20px"),
        ],
        leptos_css: "width: 100%; height: calc(100vh - 65px); background-color: var(--bg); padding-left: 20px; padding-right: 20px; padding-top: 20px; padding-bottom: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: "@media (max-width: 991px) { .sidenav nav }",

        properties: &[("padding-bottom", "200px")],
        leptos_css: "padding-bottom: 200px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: "@media screen and (max-height: 450px) { .sidenav a }",

        properties: &[("font-size", "18px")],
        leptos_css: "font-size: 18px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".badge-default",
        properties: &[
            ("background-color", "var(--secondary)"),
            ("color", "var(--fg)"),
        ],
        leptos_css: "background-color: var(--secondary); color: var(--fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".badge-og",
        properties: &[("background-color", "var(--title-fg)")],
        leptos_css: "background-color: var(--title-fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".badge-pleb",
        properties: &[("background-color", "#3ccbe3")],
        leptos_css: "background-color: #3ccbe3;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".badge-chad",
        properties: &[("background-color", "#957d0b")],
        leptos_css: "background-color: #957d0b;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".badge-whale",
        properties: &[("background-color", "var(--tertiary)")],
        leptos_css: "background-color: var(--tertiary);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".badge-silver",
        properties: &[("background-color", "#95a5a6")],
        leptos_css: "background-color: #95a5a6;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".badge-gold",
        properties: &[("background-color", "#f1c40f")],
        leptos_css: "background-color: #f1c40f;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/menu/menu.component.scss",
        selector_path: ".badge-platinum",
        properties: &[("background-color", "var(--tertiary)")],
        leptos_css: "background-color: var(--tertiary);",
        gpui: GpuiOutcome::Paint,
    },
];
