use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/assets/assets.component.scss",
        selector_path: ".td-name",
        properties: &[
            ("max-width", "200px"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
        ],
        leptos_css:
            "max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/assets/assets.component.scss",
        selector_path: ".title-asset h1",
        properties: &[
            ("line-height", "1"),
            ("margin", "0px"),
            ("padding-bottom", "10px"),
        ],
        leptos_css: "line-height: 1; margin: 0px; padding-bottom: 10px;",
        gpui: GpuiOutcome::Paint,
    },
];
