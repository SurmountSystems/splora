use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/shared/components/mining-pool/mining-pool.component.scss",
        selector_path: ".mining-pool-logo",
        properties: &[
            ("width", "15px"),
            ("height", "15px"),
            ("position", "relative"),
            ("top", "-1px"),
            ("margin-right", "5px"),
        ],
        leptos_css: "width: 15px; height: 15px; position: relative; top: -1px; margin-right: 5px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/mining-pool/mining-pool.component.scss",
        selector_path: ".mining-pool-name",
        properties: &[("display", "inline"), ("color", "var(--fg)")],
        leptos_css: "display: inline; color: var(--fg);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/shared/components/mining-pool/mining-pool.component.scss",
        selector_path: ".mining-pool-name.truncate",
        properties: &[
            ("display", "inline-block"),
            ("vertical-align", "text-top"),
            ("padding-left", "10px"),
            ("overflow", "hidden"),
            ("text-overflow", "ellipsis"),
            ("white-space", "nowrap"),
            ("max-width", "160px"),
        ],
        leptos_css: "display: inline-block; vertical-align: text-top; padding-left: 10px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 160px;",
        gpui: GpuiOutcome::Paint,
    },
];
