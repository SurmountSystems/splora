//! Direct `scan_scss` contracts for empty declaration blocks and
//! `@include block-cube.faces`.
//!
//! A parent with no declarations of its own is still emitted. A media
//! condition stays outside the composed selector.

use std::fs;
use std::path::PathBuf;

use splora_frontend_shared::style_scan::{ScannedRule, scan_scss};

#[test]
fn scan_scss_emits_empty_parents_and_expands_block_cube_faces() {
    let mut failures = Vec::new();

    expect_selectors(
        ".foo { .bar { color: red; } }",
        &[(".foo", &[]), (".foo .bar", &[("color", "red")])],
        &[],
        &mut failures,
    );
    expect_selectors(
        "@media (max-width: 10px) { .foo { color: red; } }",
        &[
            ("@media (max-width: 10px)", &[]),
            ("@media (max-width: 10px) { .foo }", &[("color", "red")]),
        ],
        &[],
        &mut failures,
    );
    expect_selectors(
        ".foo { color: red; .bar { width: 1px; } }",
        &[
            (".foo", &[("color", "red")]),
            (".foo .bar", &[("width", "1px")]),
        ],
        &[],
        &mut failures,
    );
    expect_selectors(
        "@supports (display: grid) { .foo { color: red; } }",
        &[
            ("@supports (display: grid)", &[]),
            ("@supports (display: grid) { .foo }", &[("color", "red")]),
        ],
        &[],
        &mut failures,
    );
    expect_selectors(
        "@keyframes spin { }",
        &[("@keyframes spin", &[])],
        &[],
        &mut failures,
    );
    expect_selectors(
        "@keyframes spin { 0% { opacity: 0; } }",
        &[
            ("@keyframes spin", &[]),
            ("@keyframes spin 0%", &[("opacity", "0")]),
        ],
        &[],
        &mut failures,
    );
    expect_selectors(
        "@include keyframes(pop) { 0% { opacity: 0; } 85%, 100% { opacity: 1; } }",
        &[
            ("@include keyframes(pop)", &[]),
            ("@include keyframes(pop) 0%", &[("opacity", "0")]),
            (
                "@include keyframes(pop) 85%, @include keyframes(pop) 100%",
                &[("opacity", "1")],
            ),
        ],
        &["@keyframes pop"],
        &mut failures,
    );
    expect_selectors(
        "@mixin not-a-rule { .hidden { color: red; } } .visible { color: blue; }",
        &[(".visible", &[("color", "blue")])],
        &[".hidden", "@mixin not-a-rule"],
        &mut failures,
    );
    expect_selectors(
        ".table tr { &.group-last, &:last-child { padding-bottom: 0; } }",
        &[
            (".table tr", &[]),
            (
                ".table tr.group-last, .table tr:last-child",
                &[("padding-bottom", "0")],
            ),
        ],
        &[],
        &mut failures,
    );

    let partial_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../ref/mempool/frontend/src/app/components/address-blocks/_block-cube.scss");
    let partial = fs::read_to_string(&partial_path).unwrap_or_else(|error| {
        panic!("read {}: {error}", partial_path.display());
    });
    assert!(
        partial.contains("&::after") && partial.contains("&::before"),
        "partial mixin must declare &::after and &::before"
    );

    let faces = "\
@mixin faces { &::after { color: orange; } }
.cube { @include block-cube.faces; }
";
    expect_selectors(
        faces,
        &[
            (".cube", &[]),
            (".cube::after", BLOCK_CUBE_AFTER),
            (".cube::before", BLOCK_CUBE_BEFORE),
        ],
        &["&::after", "&::before"],
        &mut failures,
    );
    expect_selectors(
        ".cube { width: 10px; @include block-cube.faces; }",
        &[
            (".cube", &[("width", "10px")]),
            (".cube::after", BLOCK_CUBE_AFTER),
            (".cube::before", BLOCK_CUBE_BEFORE),
        ],
        &[],
        &mut failures,
    );
    expect_selectors(
        "@media (max-width: 10px) { .cube { @include block-cube.faces; } }",
        &[
            ("@media (max-width: 10px)", &[]),
            ("@media (max-width: 10px) { .cube }", &[]),
            (
                "@media (max-width: 10px) { .cube::after }",
                BLOCK_CUBE_AFTER,
            ),
            (
                "@media (max-width: 10px) { .cube::before }",
                BLOCK_CUBE_BEFORE,
            ),
        ],
        &[".cube"],
        &mut failures,
    );

    if let Some(scanned) = scan_scss(faces)
        .into_iter()
        .find(|rule| rule.selector_path == ".cube::after")
    {
        for (name, value) in &scanned.properties {
            let needle = format!("{name}: {value}");
            if !partial.contains(&needle) {
                failures.push(format!(
                    ".cube::after property {needle:?} is not in {}",
                    partial_path.display()
                ));
            }
        }
        if scanned
            .properties
            .iter()
            .any(|(name, value)| name == "color" && value == "orange")
        {
            failures.push(".cube::after used the in-file mixin, not _block-cube.scss".to_string());
        }
    }

    assert!(
        failures.is_empty(),
        "scan_scss contract failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn scan_scss_matches_compiler_selector_shapes() {
    let mut failures = Vec::new();
    expect_selectors(
        ".block-filters { :host-context(.block-overview-graph:hover) &, &:hover { .menu-toggle { opacity: 1; } } }",
        &[(
            ":host-context(.block-overview-graph:hover) .block-filters .menu-toggle, .block-filters:hover .menu-toggle",
            &[("opacity", "1")],
        )],
        &[],
        &mut failures,
    );
    expect_selectors(
        ".pagination-container { display: inline-block; } .rtl-layout .pagination-container { @extend .pagination-container; }",
        &[(
            ".pagination-container, .rtl-layout .pagination-container",
            &[("display", "inline-block")],
        )],
        &[],
        &mut failures,
    );
    expect_selectors(
        ".tab-pane .subtitle { font-weight: bold; } .rtl-layout .code .subtitle { @extend .subtitle; }",
        &[(
            ".tab-pane .subtitle, .tab-pane .rtl-layout .code .subtitle, .rtl-layout .code .tab-pane .subtitle",
            &[("font-weight", "bold")],
        )],
        &[],
        &mut failures,
    );
    expect_selectors(
        ".sig { @for $i from 1 through 2 { &:nth-child(#{$i}) { color: red; } } }",
        &[
            (".sig:nth-child(1)", &[("color", "red")]),
            (".sig:nth-child(2)", &[("color", "red")]),
        ],
        &["@for"],
        &mut failures,
    );
    expect_selectors(
        "@mixin keyframes($name) { @-webkit-keyframes #{$name} { @content; } @keyframes #{$name} { @content; } } @include keyframes(pop) { 0%, 15% { opacity: 0; } }",
        &[
            ("@-webkit-keyframes pop { 0%, 15% }", &[("opacity", "0")]),
            ("@keyframes pop 0%, 15%", &[("opacity", "0")]),
        ],
        &["@include keyframes(pop)"],
        &mut failures,
    );
    expect_selectors(
        "@container (max-width: 10px) { .a { color: red; } }",
        &[
            ("@container (max-width: 10px)", &[]),
            ("@container (max-width: 10px) { .a }", &[("color", "red")]),
        ],
        &[],
        &mut failures,
    );
    assert!(
        failures.is_empty(),
        "compiler selector shapes failed:\n{}",
        failures.join("\n")
    );
}

const BLOCK_CUBE_AFTER: &[(&str, &str)] = &[
    ("content", "''"),
    ("width", "var(--block-size)"),
    ("height", "calc(0.192 * var(--block-size))"),
    ("position", "absolute"),
    ("top", "calc(-0.192 * var(--block-size))"),
    ("left", "calc(-0.16 * var(--block-size))"),
    ("background-color", "var(--block-top)"),
    ("transform", "skew(40deg)"),
    ("transform-origin", "top"),
];

const BLOCK_CUBE_BEFORE: &[(&str, &str)] = &[
    ("content", "''"),
    ("width", "calc(0.16 * var(--block-size))"),
    ("height", "var(--block-size)"),
    ("position", "absolute"),
    ("top", "calc(-0.096 * var(--block-size))"),
    ("left", "calc(-0.16 * var(--block-size))"),
    ("background-color", "var(--block-side)"),
    ("transform", "skewY(50deg)"),
    ("transform-origin", "top"),
];

fn expect_selectors(
    source: &str,
    present: &[(&str, &[(&str, &str)])],
    absent: &[&str],
    failures: &mut Vec<String>,
) {
    let scanned = scan_scss(source);
    let paths: Vec<&str> = scanned
        .iter()
        .map(|rule| rule.selector_path.as_str())
        .collect();
    for (selector, properties) in present {
        match scanned.iter().find(|rule| rule.selector_path == *selector) {
            None => failures.push(format!(
                "missing {selector:?} in {paths:?}\nsource: {source}"
            )),
            Some(rule) => {
                if !properties_match(rule, properties) {
                    failures.push(format!(
                        "properties for {selector:?} were {:?}, expected {properties:?}\nsource: {source}",
                        rule.properties
                    ));
                }
            }
        }
    }
    for selector in absent {
        if paths.contains(selector) {
            failures.push(format!(
                "emitted empty or forbidden selector {selector:?} in {paths:?}\nsource: {source}"
            ));
        }
    }
}

fn properties_match(rule: &ScannedRule, expected: &[(&str, &str)]) -> bool {
    if rule.properties.len() != expected.len() {
        return false;
    }
    rule.properties
        .iter()
        .zip(expected)
        .all(|((name, value), (expect_name, expect_value))| {
            name == expect_name && value == expect_value
        })
}
