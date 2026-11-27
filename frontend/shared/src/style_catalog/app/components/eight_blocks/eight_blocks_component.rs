use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [crate::style_rule::StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/eight-blocks/eight-blocks.component.scss",
        selector_path: ".blocks",
        properties: &[
            ("width", "100%"),
            ("height", "100%"),
            ("min-width", "100vw"),
            ("display", "flex"),
            ("flex-direction", "row"),
            ("flex-wrap", "nowrap"),
            ("justify-content", "flex-start"),
            ("align-items", "flex-start"),
            ("align-content", "flex-start"),
        ],
        leptos_css: "width: 100%; height: 100%; min-width: 100vw; display: flex; flex-direction: row; flex-wrap: nowrap; justify-content: flex-start; align-items: flex-start; align-content: flex-start;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/eight-blocks/eight-blocks.component.scss",
        selector_path: ".blocks.wrap",
        properties: &[("flex-wrap", "wrap")],
        leptos_css: "flex-wrap: wrap;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/eight-blocks/eight-blocks.component.scss",
        selector_path: ".blocks .block-wrapper",
        properties: &[
            ("flex-grow", "0"),
            ("flex-shrink", "0"),
            ("position", "relative"),
            ("--block-width", "1080px"),
        ],
        leptos_css: "flex-grow: 0; flex-shrink: 0; position: relative; --block-width: 1080px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/eight-blocks/eight-blocks.component.scss",
        selector_path: ".blocks .block-wrapper .info",
        properties: &[
            ("position", "absolute"),
            ("left", "8%"),
            ("top", "8%"),
            ("right", "8%"),
            ("bottom", "8%"),
            ("height", "84%"),
            ("width", "84%"),
            ("display", "flex"),
            ("flex-direction", "column"),
            ("align-items", "center"),
            ("justify-content", "center"),
            ("font-weight", "700"),
            ("font-size", "calc(var(--block-width) * 0.03)"),
            ("text-shadow", "0 0 calc(var(--block-width) * 0.05) black"),
        ],
        leptos_css: "position: absolute; left: 8%; top: 8%; right: 8%; bottom: 8%; height: 84%; width: 84%; display: flex; flex-direction: column; align-items: center; justify-content: center; font-weight: 700; font-size: calc(var(--block-width) * 0.03); text-shadow: 0 0 calc(var(--block-width) * 0.05) black;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/eight-blocks/eight-blocks.component.scss",
        selector_path: ".blocks .block-wrapper .info h1",
        properties: &[
            ("font-size", "6em"),
            ("line-height", "1"),
            ("margin-bottom", "calc(var(--block-width) * 0.03)"),
        ],
        leptos_css: "font-size: 6em; line-height: 1; margin-bottom: calc(var(--block-width) * 0.03);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/eight-blocks/eight-blocks.component.scss",
        selector_path: ".blocks .block-wrapper .info h2",
        properties: &[
            ("font-size", "1.8em"),
            ("line-height", "1"),
            ("margin-bottom", "calc(var(--block-width) * 0.03)"),
        ],
        leptos_css: "font-size: 1.8em; line-height: 1; margin-bottom: calc(var(--block-width) * 0.03);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/eight-blocks/eight-blocks.component.scss",
        selector_path: ".blocks .block-wrapper .info .hash",
        properties: &[
            ("font-family", "monospace"),
            ("word-wrap", "break-word"),
            ("font-size", "1.4em"),
            ("line-height", "1"),
            ("margin-bottom", "calc(var(--block-width) * 0.03)"),
        ],
        leptos_css: "font-family: monospace; word-wrap: break-word; font-size: 1.4em; line-height: 1; margin-bottom: calc(var(--block-width) * 0.03);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/eight-blocks/eight-blocks.component.scss",
        selector_path: ".blocks .block-wrapper .info .mined-by",
        properties: &[
            ("position", "absolute"),
            ("bottom", "0"),
            ("margin", "auto"),
            ("text-align", "center"),
        ],
        leptos_css: "position: absolute; bottom: 0; margin: auto; text-align: center;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/eight-blocks/eight-blocks.component.scss",
        selector_path: ".blocks .block-container",
        properties: &[("overflow", "hidden")],
        leptos_css: "overflow: hidden;",
        gpui: GpuiOutcome::Paint,
    },
];
