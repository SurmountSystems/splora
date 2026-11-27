use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".box",
        properties: &[("margin-top", "20px")],
        leptos_css: "margin-top: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: "@media (max-width: 768px) { .box }",
        properties: &[("margin-bottom", "20px")],
        leptos_css: "margin-bottom: 20px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".starting-balance, .closing-balance",
        properties: &[("width", "100%")],
        leptos_css: "width: 100%;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".starting-balance h5, .closing-balance h5",
        properties: &[("text-align", "center")],
        leptos_css: "text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".nodes",
        properties: &[
            ("display", "none"),
            ("flex-direction", "row"),
            ("align-items", "baseline"),
            ("justify-content", "space-between"),
        ],
        leptos_css: "display: none; flex-direction: row; align-items: baseline; justify-content: space-between;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: "@media (max-width: 768px) { .nodes }",
        properties: &[("display", "flex")],
        leptos_css: "display: flex;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".balances",
        properties: &[
            ("display", "flex"),
            ("flex-direction", "row"),
            ("align-items", "baseline"),
            ("justify-content", "space-between"),
            ("margin-bottom", "8px"),
        ],
        leptos_css: "display: flex; flex-direction: row; align-items: baseline; justify-content: space-between; margin-bottom: 8px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".balances .balance.left",
        properties: &[("text-align", "start")],
        leptos_css: "text-align: start;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".balances .balance.right",
        properties: &[("text-align", "end")],
        leptos_css: "text-align: end;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".balance-bar",
        properties: &[
            ("width", "100%"),
            ("height", "2em"),
            ("position", "relative"),
        ],
        leptos_css: "width: 100%; height: 2em; position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".balance-bar .bar",
        properties: &[
            ("position", "absolute"),
            ("top", "0"),
            ("bottom", "0"),
            ("height", "100%"),
            ("display", "flex"),
            ("flex-direction", "row"),
            ("align-items", "center"),
            ("justify-content", "center"),
        ],
        leptos_css: "position: absolute; top: 0; bottom: 0; height: 100%; display: flex; flex-direction: row; align-items: center; justify-content: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".balance-bar .bar.left",
        properties: &[("background", "var(--primary)")],
        leptos_css: "background: var(--primary);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".balance-bar .bar.center",
        properties: &[(
            "background",
            "repeating-linear-gradient(60deg, var(--primary) 0, var(--primary) 12px, var(--success) 12px, var(--success) 24px)",
        )],
        leptos_css: "background: repeating-linear-gradient(60deg, var(--primary) 0, var(--primary) 12px, var(--success) 12px, var(--success) 24px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".balance-bar .bar.right",
        properties: &[("background", "var(--success)")],
        leptos_css: "background: var(--success);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".balance-bar .bar .value",
        properties: &[("flex", "0"), ("white-space", "nowrap")],
        leptos_css: "flex: 0; white-space: nowrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: ".balance-bar .bar.hide-value .value",
        properties: &[("display", "none")],
        leptos_css: "display: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: "@media (max-width: 768px) { .balance-bar }",
        properties: &[("height", "1em")],
        leptos_css: "height: 1em;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/lightning/channel/channel-close-box/channel-close-box.component.scss",
        selector_path: "@media (max-width: 768px) { .balance-bar .bar.center }",
        properties: &[(
            "background",
            "repeating-linear-gradient(60deg, var(--primary) 0, var(--primary) 8px, var(--success) 8px, var(--success) 16px)",
        )],
        leptos_css: "background: repeating-linear-gradient(60deg, var(--primary) 0, var(--primary) 8px, var(--success) 8px, var(--success) 16px);",
        gpui: GpuiOutcome::Paint,
    },
];
