use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/shared/components/toggle/toggle.component.scss",
        selector_path: ".switch",
        properties: &[
            ("position", "relative"),
            ("display", "inline-block"),
            ("width", "30px"),
            ("height", "17px"),
        ],
        leptos_css: "position: relative; display: inline-block; width: 30px; height: 17px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/toggle/toggle.component.scss",
        selector_path: ".switch input",
        properties: &[("opacity", "0"), ("width", "0"), ("height", "0")],
        leptos_css: "opacity: 0; width: 0; height: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/toggle/toggle.component.scss",
        selector_path: ".slider",
        properties: &[
            ("position", "absolute"),
            ("cursor", "pointer"),
            ("top", "0"),
            ("left", "0"),
            ("right", "0"),
            ("bottom", "0"),
            ("background-color", "#ccc"),
        ],
        leptos_css: "position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: #ccc;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/toggle/toggle.component.scss",
        selector_path: ".slider:before",
        properties: &[
            ("position", "absolute"),
            ("content", "\"\""),
            ("height", "13px"),
            ("width", "13px"),
            ("left", "2px"),
            ("bottom", "2px"),
            ("background-color", "white"),
        ],
        leptos_css: "position: absolute; content: \"\"; height: 13px; width: 13px; left: 2px; bottom: 2px; background-color: white;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/toggle/toggle.component.scss",
        selector_path: ".slider.animate, .slider.animate:before",
        properties: &[("-webkit-transition", "0.4s"), ("transition", "0.4s")],
        leptos_css: "-webkit-transition: 0.4s; transition: 0.4s;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/toggle/toggle.component.scss",
        selector_path: "input:checked + .slider",
        properties: &[("background-color", "#2196F3")],
        leptos_css: "background-color: #2196F3;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/toggle/toggle.component.scss",
        selector_path: "input:focus + .slider",
        properties: &[("box-shadow", "0 0 1px #2196F3")],
        leptos_css: "box-shadow: 0 0 1px #2196F3;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/toggle/toggle.component.scss",
        selector_path: "input:checked + .slider:before",
        properties: &[
            ("-webkit-transform", "translateX(13px)"),
            ("-ms-transform", "translateX(13px)"),
            ("transform", "translateX(13px)"),
        ],
        leptos_css: "-webkit-transform: translateX(13px); -ms-transform: translateX(13px); transform: translateX(13px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/toggle/toggle.component.scss",
        selector_path: ".slider.round",
        properties: &[("border-radius", "17px")],
        leptos_css: "border-radius: 17px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/toggle/toggle.component.scss",
        selector_path: ".slider.round:before",
        properties: &[("border-radius", "50%")],
        leptos_css: "border-radius: 50%;",
        gpui: GpuiOutcome::Paint,
    },
];
