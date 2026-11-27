use crate::style_rule::{GpuiOutcome, StyleRule};

pub fn rules() -> &'static [StyleRule] {
    RULES
}

static RULES: &[StyleRule] = &[
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".bitcoin-block",
        properties: &[
            ("width", "var(--block-size)"),
            ("height", "var(--block-size)"),
            (
                "transition",
                "background 2s, right 2s, transform 1s, opacity 1s",
            ),
        ],
        leptos_css: "width: var(--block-size); height: var(--block-size); transition: background 2s, right 2s, transform 1s, opacity 1s;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".block-size",
        properties: &[("font-size", "16px"), ("font-weight", "bold")],
        leptos_css: "font-size: 16px; font-weight: bold;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".mempool-blocks-container",
        properties: &[
            ("position", "absolute"),
            ("top", "0px"),
            ("right", "0px"),
            ("left", "0px"),
            ("--block-size", "125px"),
        ],
        leptos_css: "position: absolute; top: 0px; right: 0px; left: 0px; --block-size: 125px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".flashing",
        properties: &[
            ("will-change", "opacity"),
            ("transform", "translateZ(0)"),
            ("animation", "opacityPulse 2s steps(30, end)"),
            ("animation-iteration-count", "infinite"),
            ("opacity", "1"),
        ],
        leptos_css: "will-change: opacity; transform: translateZ(0); animation: opacityPulse 2s steps(30, end); animation-iteration-count: infinite; opacity: 1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".mempool-block",
        properties: &[("position", "absolute"), ("top", "0")],
        leptos_css: "position: absolute; top: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".block-body",
        properties: &[("text-align", "center"), ("transition", "transform 1s")],
        leptos_css: "text-align: center; transition: transform 1s;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: "@keyframes opacityPulse 0%",
        properties: &[("opacity", "0.7")],
        leptos_css: "opacity: 0.7;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: "@keyframes opacityPulse 50%",
        properties: &[("opacity", "1")],
        leptos_css: "opacity: 1;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: "@keyframes opacityPulse 100%",
        properties: &[("opacity", "0.7")],
        leptos_css: "opacity: 0.7;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".time-difference",
        properties: &[("font-size", "13px")],
        leptos_css: "font-size: 13px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".fees",
        properties: &[
            ("font-size", "12px"),
            ("margin-top", "10px"),
            ("margin-bottom", "2px"),
        ],
        leptos_css: "font-size: 12px; margin-top: 10px; margin-bottom: 2px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".fee-span",
        properties: &[
            ("font-size", "11px"),
            ("margin-bottom", "5px"),
            ("color", "var(--yellow)"),
        ],
        leptos_css: "font-size: 11px; margin-bottom: 5px; color: var(--yellow);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".transaction-count",
        properties: &[
            ("font-size", "10px"),
            ("margin-top", "3px"),
            ("margin-bottom", "4px"),
        ],
        leptos_css: "font-size: 10px; margin-top: 3px; margin-bottom: 4px;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".bitcoin-block::after",
        properties: &[
            ("content", "''"),
            ("width", "var(--block-size)"),
            ("height", "calc(0.192 * var(--block-size))"),
            ("position", "absolute"),
            ("top", "calc(-0.192 * var(--block-size))"),
            ("left", "calc(-0.16 * var(--block-size))"),
            ("background-color", "var(--block-top)"),
            ("transform", "skew(40deg)"),
            ("transform-origin", "top"),
            ("transition", "transform 1s, left 1s"),
        ],
        leptos_css: "content: ''; width: var(--block-size); height: calc(0.192 * var(--block-size)); position: absolute; top: calc(-0.192 * var(--block-size)); left: calc(-0.16 * var(--block-size)); background-color: var(--block-top); transform: skew(40deg); transform-origin: top; transition: transform 1s, left 1s;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".bitcoin-block::before",
        properties: &[
            ("content", "''"),
            ("width", "calc(0.16 * var(--block-size))"),
            ("height", "var(--block-size)"),
            ("position", "absolute"),
            ("top", "calc(-0.096 * var(--block-size))"),
            ("left", "calc(-0.16 * var(--block-size))"),
            ("background-color", "var(--block-side)"),
            ("z-index", "-1"),
            ("transform", "skewY(50deg)"),
            ("transform-origin", "top"),
            ("transition", "transform 1s, left 1s"),
        ],
        leptos_css: "content: ''; width: calc(0.16 * var(--block-size)); height: var(--block-size); position: absolute; top: calc(-0.096 * var(--block-size)); left: calc(-0.16 * var(--block-size)); background-color: var(--block-side); z-index: -1; transform: skewY(50deg); transform-origin: top; transition: transform 1s, left 1s;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".mempool-block.bitcoin-block::after",
        properties: &[("background-color", "var(--mempool-block-top)")],
        leptos_css: "background-color: var(--mempool-block-top);",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".mempool-block.bitcoin-block::before",
        properties: &[("background-color", "var(--mempool-block-side)")],
        leptos_css: "background-color: var(--mempool-block-side);",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".mempool-block.hide-block",
        properties: &[("opacity", "0")],
        leptos_css: "opacity: 0;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".black-background",
        properties: &[
            ("background-color", "var(--active-bg)"),
            ("z-index", "100"),
            ("position", "relative"),
        ],
        leptos_css: "background-color: var(--active-bg); z-index: 100; position: relative;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: "#arrow-up",
        properties: &[
            ("position", "relative"),
            ("right", "calc(var(--block-size) * 0.6)"),
            ("top", "calc(var(--block-size) * 1.2)"),
            ("width", "0"),
            ("height", "0"),
            (
                "border-left",
                "calc(var(--block-size) * 0.3) solid transparent",
            ),
            (
                "border-right",
                "calc(var(--block-size) * 0.3) solid transparent",
            ),
            (
                "border-bottom",
                "calc(var(--block-size) * 0.3) solid var(--nav-icon)",
            ),
        ],
        leptos_css: "position: relative; right: calc(var(--block-size) * 0.6); top: calc(var(--block-size) * 1.2); width: 0; height: 0; border-left: calc(var(--block-size) * 0.3) solid transparent; border-right: calc(var(--block-size) * 0.3) solid transparent; border-bottom: calc(var(--block-size) * 0.3) solid var(--nav-icon);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".blockLink",
        properties: &[
            ("width", "100%"),
            ("height", "100%"),
            ("position", "absolute"),
            ("left", "0"),
            ("z-index", "10"),
        ],
        leptos_css: "width: 100%; height: 100%; position: absolute; left: 0; z-index: 10;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".blockLink.disabled",
        properties: &[("pointer-events", "none")],
        leptos_css: "pointer-events: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".blockLink:hover",
        properties: &[("text-decoration", "none")],
        leptos_css: "text-decoration: none;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".time-ltr .bitcoin-block::after",
        properties: &[("transform", "skew(-40deg)"), ("left", "20px")],
        leptos_css: "transform: skew(-40deg); left: 20px;",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".time-ltr .bitcoin-block::before",
        properties: &[
            ("transform", "skewY(-50deg)"),
            ("left", "var(--block-size)"),
        ],
        leptos_css: "transform: skewY(-50deg); left: var(--block-size);",
        gpui: GpuiOutcome::NotAWidget { feature: "pseudo" },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".time-ltr .block-body",
        properties: &[("transform", "scaleX(-1)")],
        leptos_css: "transform: scaleX(-1);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ":host-context(.rtl-layout) #arrow-up",
        properties: &[("transform", "translateX(70px)")],
        leptos_css: "transform: translateX(70px);",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".spotlight-bottom",
        properties: &[
            ("position", "absolute"),
            ("width", "calc(0.6 * var(--block-size))"),
            ("height", "calc(0.25 * var(--block-size))"),
            (
                "border-left",
                "solid calc(0.3 * var(--block-size)) transparent",
            ),
            (
                "border-bottom",
                "solid calc(0.3 * var(--block-size)) var(--fg)",
            ),
            (
                "border-right",
                "solid calc(0.3 * var(--block-size)) transparent",
            ),
            (
                "transform",
                "translate(calc(-0.2 * var(--block-size)), calc(1.1 * var(--block-size)))",
            ),
            ("border-radius", "2px"),
            ("z-index", "-1"),
        ],
        leptos_css: "position: absolute; width: calc(0.6 * var(--block-size)); height: calc(0.25 * var(--block-size)); border-left: solid calc(0.3 * var(--block-size)) transparent; border-bottom: solid calc(0.3 * var(--block-size)) var(--fg); border-right: solid calc(0.3 * var(--block-size)) transparent; transform: translate(calc(-0.2 * var(--block-size)), calc(1.1 * var(--block-size))); border-radius: 2px; z-index: -1;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: ".blink",
        properties: &[
            ("width", "400px"),
            ("height", "400px"),
            (
                "border-bottom",
                "solid calc(0.3 * var(--block-size)) var(--fg)",
            ),
            ("animation", "blink 0.2s infinite"),
        ],
        leptos_css: "width: 400px; height: 400px; border-bottom: solid calc(0.3 * var(--block-size)) var(--fg); animation: blink 0.2s infinite;",
        gpui: GpuiOutcome::Paint,
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: "@keyframes blink 0%",
        properties: &[("border-bottom", "solid calc(0.3 * var(--block-size)) green")],
        leptos_css: "border-bottom: solid calc(0.3 * var(--block-size)) green;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: "@keyframes blink 50%",
        properties: &[(
            "border-bottom",
            "solid calc(0.3 * var(--block-size)) yellow",
        )],
        leptos_css: "border-bottom: solid calc(0.3 * var(--block-size)) yellow;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: "@keyframes blink 100%",
        properties: &[(
            "border-bottom",
            "solid calc(0.3 * var(--block-size)) orange",
        )],
        leptos_css: "border-bottom: solid calc(0.3 * var(--block-size)) orange;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: "@-webkit-keyframes blink { 0% }",
        properties: &[("border-bottom", "solid calc(0.3 * var(--block-size)) green")],
        leptos_css: "border-bottom: solid calc(0.3 * var(--block-size)) green;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: "@-webkit-keyframes blink { 50% }",
        properties: &[(
            "border-bottom",
            "solid calc(0.3 * var(--block-size)) yellow",
        )],
        leptos_css: "border-bottom: solid calc(0.3 * var(--block-size)) yellow;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
    StyleRule {
        source_path: "app/components/mempool-blocks/mempool-blocks.component.scss",
        selector_path: "@-webkit-keyframes blink { 100% }",
        properties: &[(
            "border-bottom",
            "solid calc(0.3 * var(--block-size)) orange",
        )],
        leptos_css: "border-bottom: solid calc(0.3 * var(--block-size)) orange;",
        gpui: GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
    },
];
