//! Calls the style proof. The empty-failures assert stays so a missing Sass
//! compiler fails this test.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use splora_frontend_shared::style_proof::{
    GpuiVerdict, ProofReport, UpstreamCss, push_selector_mismatches,
};
use splora_frontend_shared::style_rule::{GpuiOutcome, StyleRule};
use splora_frontend_shared::style_scan::{ScanKind, ScannedRule};
use splora_native::style_paint::{Display, Overflow, paint_rule};

#[test]
fn style_proof_failures_are_empty() {
    let report = prove_style_catalog();
    assert!(report.failures.is_empty(), "{}", report.failures.join("\n"));
}

/// The helper must return CSS from the Sass compiler for one upstream file.
/// A missing-compiler string is not that CSS. This does not replace
/// `style_proof_failures_are_empty`.
#[test]
fn sass_compiler_emits_css_for_one_upstream_file() {
    let scss = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../ref/mempool/frontend/src/app/components/address-blocks/address-blocks.component.scss",
    );
    let css = compile_upstream_scss(&scss).unwrap_or_else(|error| error);
    assert!(
        !css.contains("missing Sass compiler"),
        "compiler helper returned the missing-compiler string: {css}"
    );
    assert!(
        css.contains(".activity-strip.dragging") && css.contains("display: block"),
        "compiled CSS was not the compiler output for address-blocks: {css}"
    );
    assert!(
        css.contains("@media (min-width: 768px) and (min-height: 700px)"),
        "compiled CSS dropped the upstream media query: {css}"
    );
}

#[test]
fn style_proof_consumer_prints_rules_checked_and_failures() {
    let report = prove_style_catalog();
    println!("rules_checked={}", report.rules_checked);
    println!("fail_count={}", report.failures.len());
    for failure in &report.failures {
        println!("failure={failure}");
    }

    let mut problems = Vec::new();
    let address = "address-blocks.component.scss";
    let autos: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| {
            verdict.source_path.contains(address)
                && verdict.selector_path == ".activity-strip"
                && verdict.specified == "auto"
        })
        .collect();
    if autos.is_empty()
        || autos.iter().any(|verdict| !verdict.field.is_empty())
        || autos
            .iter()
            .any(|verdict| verdict.feature != "overflow-x:auto")
    {
        let shown = autos
            .first()
            .map(|verdict| verdict_text(verdict))
            .unwrap_or_else(|| "none".to_string());
        problems.push(format!(
            "overflow-x:auto must be a named unhostable feature, not a field match: {shown}"
        ));
    }

    let grids: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| {
            verdict.source_path.contains(address)
                && verdict.selector_path == ".block-face"
                && verdict.specified == "grid"
        })
        .collect();
    let face_rule = splora_frontend_shared::all_rules()
        .into_iter()
        .find(|rule| rule.source_path.contains(address) && rule.selector_path == ".block-face");
    let face_paint = face_rule.map(paint_rule);
    if grids.len() != 1
        || grids[0].field != "display"
        || !grids[0].feature.is_empty()
        || face_paint
            .as_ref()
            .is_none_or(|painted| !painted.wrote_display || painted.style.display != Display::Grid)
    {
        let shown = grids
            .first()
            .map(|verdict| verdict_text(verdict))
            .unwrap_or_else(|| "none".to_string());
        let display = face_paint
            .as_ref()
            .map(|painted| format!("{:?}", painted.style.display))
            .unwrap_or_else(|| "no style".to_string());
        problems.push(format!(
            ".block-face display:grid must write Style.display = Display::Grid, not painter=={:?}: {shown} style.display={display}",
            grids.first().map(|verdict| verdict.painter)
        ));
    }

    let visibles: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| {
            verdict.source_path.contains(address)
                && verdict.selector_path == ".block-face"
                && verdict.specified == "visible"
        })
        .collect();
    if visibles.len() != 1
        || visibles[0].field != "overflow"
        || !visibles[0].feature.is_empty()
        || face_paint.as_ref().is_none_or(|painted| {
            !painted.wrote_overflow
                || painted.style.overflow.x != Overflow::Visible
                || painted.style.overflow.y != Overflow::Visible
        })
    {
        let shown = visibles
            .first()
            .map(|verdict| verdict_text(verdict))
            .unwrap_or_else(|| "none".to_string());
        let overflow = face_paint
            .as_ref()
            .map(|painted| format!("{:?}", painted.style.overflow))
            .unwrap_or_else(|| "no style".to_string());
        problems.push(format!(
            ".block-face overflow:visible must write Style.overflow = Overflow::Visible, not painter=={:?}: {shown} style.overflow={overflow}",
            visibles.first().map(|verdict| verdict.painter)
        ));
    }

    for (selector, feature) in [
        (".block-face::before", "::before"),
        (".block-face::after", "::after"),
    ] {
        let named: Vec<_> = report
            .gpui_verdicts
            .iter()
            .filter(|verdict| {
                verdict.source_path.contains(address) && verdict.selector_path == selector
            })
            .collect();
        if named.is_empty()
            || named.iter().any(|verdict| verdict.feature != feature)
            || named.iter().any(|verdict| verdict.feature == "pseudo")
        {
            let shown = named
                .first()
                .map(|verdict| verdict_text(verdict))
                .unwrap_or_else(|| "none".to_string());
            problems.push(format!(
                "{selector} verdict must name {feature}, not only pseudo: {shown}"
            ));
        }
    }

    let placeholders: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| verdict.selector_path == ".form-control::placeholder")
        .collect();
    if placeholders.is_empty()
        || placeholders
            .iter()
            .any(|verdict| verdict.feature != "::placeholder")
        || placeholders
            .iter()
            .any(|verdict| verdict.feature == "pseudo")
    {
        let shown = placeholders
            .first()
            .map(|verdict| verdict_text(verdict))
            .unwrap_or_else(|| "none".to_string());
        problems.push(format!(
            ".form-control::placeholder verdict must name ::placeholder, not pseudo: {shown}"
        ));
    }

    let ng_deep: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| verdict.selector_path == ":host ::ng-deep .dropdown-item")
        .collect();
    if ng_deep.is_empty()
        || !ng_deep.iter().any(|verdict| verdict.feature == "::ng-deep")
        || !ng_deep.iter().any(|verdict| verdict.feature == ":host")
        || ng_deep.iter().any(|verdict| verdict.feature == "pseudo")
        || ng_deep.iter().any(|verdict| !verdict.field.is_empty())
    {
        let shown = ng_deep
            .first()
            .map(|verdict| verdict_text(verdict))
            .unwrap_or_else(|| "none".to_string());
        problems.push(format!(
            ":host ::ng-deep .dropdown-item verdict must name ::ng-deep, not pseudo: {shown}"
        ));
    }

    let filter_only = report
        .leptos_verdict
        .lines()
        .any(|line| line.trim() == "filter: brightness(0.7);");
    let face_without_filter = report.leptos_verdict.lines().any(|line| {
        let line = line.trim();
        line.contains("background-color: var(--block-side)")
            && line.contains("transform: skewY(50deg)")
            && !line.contains("filter")
    });
    let before_merged: Vec<_> = report
        .failures
        .iter()
        .filter(|failure| {
            failure.contains(address)
                && failure.contains("selector_path=.block-face::before")
                && (failure.contains("do not match compiled")
                    || failure.contains("not the compiled block")
                    || failure.contains("compiled block does not match"))
        })
        .cloned()
        .collect();
    let scrollbar: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| verdict.selector_path == "::-webkit-scrollbar")
        .collect();
    if scrollbar.is_empty()
        || scrollbar
            .iter()
            .any(|verdict| verdict.feature != "::-webkit-scrollbar")
        || scrollbar.iter().any(|verdict| verdict.feature == "pseudo")
        || scrollbar.iter().any(|verdict| !verdict.field.is_empty())
    {
        let shown = scrollbar
            .first()
            .map(|verdict| verdict_text(verdict))
            .unwrap_or_else(|| "none".to_string());
        problems.push(format!(
            "::-webkit-scrollbar must name that pseudo, not pseudo or a resting field: {shown}"
        ));
    }

    let focus: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| verdict.selector_path == ":focus")
        .collect();
    if focus.is_empty()
        || focus.iter().any(|verdict| verdict.feature != ":focus")
        || focus.iter().any(|verdict| !verdict.field.is_empty())
    {
        let shown = focus
            .first()
            .map(|verdict| verdict_text(verdict))
            .unwrap_or_else(|| "none".to_string());
        problems.push(format!(
            ":focus must name :focus and must not paint outline as resting style: {shown}"
        ));
    }

    let progress: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| verdict.selector_path == "@keyframes progress 0%")
        .collect();
    let resting = report
        .failures
        .iter()
        .any(|failure| failure.contains("keyframes treated as resting paint"));
    if progress.is_empty()
        || progress
            .iter()
            .any(|verdict| verdict.feature != "keyframes" || !verdict.field.is_empty())
        || resting
    {
        let shown = progress
            .first()
            .map(|verdict| verdict_text(verdict))
            .unwrap_or_else(|| "none".to_string());
        problems.push(format!(
            "@keyframes progress 0% must stay a keyframes sample, not resting paint: {shown} resting={resting}"
        ));
    }

    let pseudo_word = report
        .gpui_verdicts
        .iter()
        .find(|verdict| verdict.feature == "pseudo");
    if let Some(verdict) = pseudo_word {
        problems.push(format!(
            "gpui verdict uses the catalog word pseudo source_path={} selector_path={}",
            verdict.source_path, verdict.selector_path
        ));
    }

    let host_header: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| {
            verdict.source_path.contains("address-block-group-header")
                && verdict.selector_path == ":host"
        })
        .collect();
    let header_display = host_header.iter().any(|verdict| {
        verdict.field == "display" && verdict.specified == "flex" && verdict.feature.is_empty()
    });
    let header_wrap = host_header.iter().any(|verdict| {
        verdict.field == "flex_wrap" && verdict.specified == "wrap" && verdict.feature.is_empty()
    });
    let header_gap = host_header.iter().any(|verdict| {
        verdict.field == "gap" && verdict.specified == "0.5rem 1rem" && verdict.feature.is_empty()
    });
    if !header_display
        || !header_wrap
        || !header_gap
        || host_header
            .iter()
            .any(|verdict| verdict.feature == ":host" && verdict.field.is_empty())
    {
        let shown = host_header
            .iter()
            .map(|verdict| verdict_text(verdict))
            .collect::<Vec<_>>()
            .join(" | ");
        problems.push(format!(
            ":host on address-block-group-header must paint display flex, flex-wrap wrap, and gap, not feature :host: {shown}"
        ));
    }

    let host_blocks: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| {
            verdict.source_path.contains("address-blocks.component")
                && verdict.selector_path == ":host"
        })
        .collect();
    let blocks_display = host_blocks.iter().any(|verdict| {
        verdict.field == "display" && verdict.specified == "block" && verdict.feature.is_empty()
    });
    if !blocks_display
        || host_blocks
            .iter()
            .any(|verdict| verdict.feature == ":host" && verdict.field.is_empty())
    {
        let shown = host_blocks
            .iter()
            .map(|verdict| verdict_text(verdict))
            .collect::<Vec<_>>()
            .join(" | ");
        problems.push(format!(
            ":host on address-blocks must paint display block, not feature :host: {shown}"
        ));
    }

    let not_label: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| verdict.selector_path == "form label:not(.form-check-label):not(.btn)")
        .collect();
    let label_margin = not_label.iter().any(|verdict| {
        verdict.field == "margin" && verdict.specified == "0.5rem" && verdict.feature.is_empty()
    });
    if !label_margin || not_label.iter().any(|verdict| verdict.feature == ":not") {
        let shown = not_label
            .iter()
            .map(|verdict| verdict_text(verdict))
            .collect::<Vec<_>>()
            .join(" | ");
        problems.push(format!(
            "form label:not must paint margin-bottom 0.5rem, not feature :not: {shown}"
        ));
    }

    let not_cell: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| verdict.selector_path == ".table > :not(caption) > * > *")
        .collect();
    let cell_border = not_cell.iter().any(|verdict| {
        verdict.field == "border_widths" && verdict.specified == "0px" && verdict.feature.is_empty()
    });
    if !cell_border || not_cell.iter().any(|verdict| verdict.feature == ":not") {
        let shown = not_cell
            .iter()
            .map(|verdict| verdict_text(verdict))
            .collect::<Vec<_>>()
            .join(" | ");
        problems.push(format!(
            ".table > :not(caption) must paint border-bottom-width 0px, not feature :not: {shown}"
        ));
    }

    let focus_visible: Vec<_> = report
        .gpui_verdicts
        .iter()
        .filter(|verdict| verdict.selector_path == ":host:focus-visible")
        .collect();
    if focus_visible.is_empty()
        || !focus_visible
            .iter()
            .any(|verdict| verdict.feature == ":focus-visible")
        || !focus_visible
            .iter()
            .any(|verdict| verdict.feature == ":host")
        || focus_visible
            .iter()
            .any(|verdict| !verdict.field.is_empty())
    {
        let shown = focus_visible
            .first()
            .map(|verdict| verdict_text(verdict))
            .unwrap_or_else(|| "none".to_string());
        problems.push(format!(
            ":host:focus-visible must stay a named feature with an empty field: {shown}"
        ));
    }

    if !filter_only || !face_without_filter || !before_merged.is_empty() {
        problems.push(format!(
            ".block-face::before split was not checked (filter_only={filter_only} face_without_filter={face_without_filter} mismatches={})",
            before_merged.join(" | ")
        ));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));

    let rules = splora_frontend_shared::all_rules();
    assert_eq!(
        report.leptos_verdict,
        splora_frontend_shared::style_proof::leptos_stylesheet_css(),
        "leptos verdict must be the stylesheet the page emits"
    );
    assert!(!report.leptos_verdict.trim().is_empty());
    assert!(!rules.is_empty());
    for rule in rules {
        let gpui = report.gpui_verdicts.iter().any(|verdict| {
            verdict.source_path == rule.source_path && verdict.selector_path == rule.selector_path
        });
        assert!(
            gpui,
            "missing gpui verdict source_path={} selector_path={}",
            rule.source_path, rule.selector_path
        );
        let leptos = rule.leptos_css.trim();
        if !leptos.is_empty() {
            assert!(
                report.leptos_verdict.contains(leptos),
                "missing leptos verdict source_path={} selector_path={}",
                rule.source_path,
                rule.selector_path
            );
        }
    }
}

fn verdict_text(verdict: &GpuiVerdict) -> String {
    format!(
        "field={} specified={} painter={} feature={} selector={}",
        verdict.field, verdict.specified, verdict.painter, verdict.feature, verdict.selector_path
    )
}

/// Bootstrap is loaded so `styles.scss` can compile. Bootstrap library
/// selectors stay out of the planned-rule failure list.
#[test]
fn bootstrap_library_rules_are_not_planned_mempool_rules() {
    let report = prove_style_catalog();
    let accordion: Vec<_> = report
        .failures
        .iter()
        .filter(|failure| failure.contains("selector_path=.accordion"))
        .cloned()
        .collect();
    assert!(
        accordion.is_empty(),
        "bootstrap accordion rules are not planned mempool rules:\n{}",
        accordion.join("\n")
    );
    let badge_padding: Vec<_> = report
        .failures
        .iter()
        .filter(|failure| failure.contains("--bs-badge-padding-x"))
        .cloned()
        .collect();
    assert!(
        badge_padding.is_empty(),
        "bootstrap badge padding is not a planned mempool declaration:\n{}",
        badge_padding.join("\n")
    );
}

#[test]
fn duplicate_selector_paths_match_by_membership() {
    let source_path = "app/components/block/block.component.scss";
    let shared = "h1 a:hover, h1 a:focus";
    let other = ".block-cube::before";
    let extra = ".menu-button";
    let first = catalog_rule(source_path, shared);
    let second = catalog_rule(source_path, shared);
    let third = catalog_rule(source_path, other);
    let scanned = [scanned_rule(shared), scanned_rule(extra)];
    let mut failures = Vec::new();
    push_selector_mismatches(
        source_path,
        &scanned,
        &[&first, &second, &third],
        &mut failures,
    );

    let shared_suffix = format!("selector_path={shared}");
    let shared_failures: Vec<_> = failures
        .iter()
        .filter(|failure| failure.ends_with(shared_suffix.as_str()))
        .cloned()
        .collect();
    assert!(
        shared_failures.is_empty(),
        "both catalog rows with selector_path {shared:?} must pass when one scanned rule has that path: {shared_failures:?}"
    );

    let catalog_other = format!(
        "catalog selector_path is not the scanner outside-media form source_path={source_path} selector_path={other}"
    );
    assert!(
        failures.iter().any(|failure| failure == &catalog_other),
        "a catalog selector the scan does not have must fail: {failures:?}"
    );

    let scanned_extra = format!(
        "scanned rule has no catalog selector_path in the scanner outside-media form source_path={source_path} selector_path={extra}"
    );
    assert!(
        failures.iter().any(|failure| failure == &scanned_extra),
        "a scanned selector no catalog row has must fail: {failures:?}"
    );
    assert_eq!(failures.len(), 2, "{failures:?}");
}

#[test]
fn empty_catalog_is_rejected() {
    let report = prove_style_rules(&[]);
    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure == "empty catalog"),
        "an empty catalog must fail: {:?}",
        report.failures
    );
    assert!(report.gpui_verdicts.is_empty());
    assert!(report.leptos_verdict.is_empty());
}

#[test]
fn empty_leptos_css_is_not_filled_from_properties() {
    let rule = StyleRule {
        source_path: "app/components/address-blocks/address-blocks.component.scss",
        selector_path: ".activity-strip",
        properties: &[("overflow-x", "auto")],
        leptos_css: "",
        gpui: GpuiOutcome::Paint,
    };
    let report = prove_style_rules(&[&rule]);
    assert!(
        report.failures.iter().any(|failure| {
            failure.contains("empty leptos_css")
                && failure.contains("selector_path=.activity-strip")
        }),
        "empty leptos_css was filled from properties: {:?}",
        report
            .failures
            .iter()
            .filter(|failure| failure.contains(".activity-strip"))
            .collect::<Vec<_>>()
    );
    assert!(
        !report.leptos_verdict.contains("overflow-x"),
        "the page stylesheet invented a block from an empty leptos_css: {}",
        report.leptos_verdict
    );
}

#[test]
fn invented_empty_block_is_not_compiled_css() {
    let rule = StyleRule {
        source_path: "app/components/address-blocks/address-blocks.component.scss",
        selector_path: ".invented-empty-block",
        properties: &[],
        leptos_css: ".invented-empty-block {}",
        gpui: GpuiOutcome::Paint,
    };
    let report = prove_style_rules(&[&rule]);
    assert!(
        report.failures.iter().any(|failure| {
            failure.contains("invented empty block")
                && failure.contains("selector_path=.invented-empty-block")
        }),
        "a selector plus {{}} the compiler did not emit must fail: {:?}",
        report
            .failures
            .iter()
            .filter(|failure| failure.contains("invented-empty-block"))
            .collect::<Vec<_>>()
    );
}

fn catalog_rule(source_path: &'static str, selector_path: &'static str) -> StyleRule {
    StyleRule {
        source_path,
        selector_path,
        properties: &[],
        leptos_css: "",
        gpui: GpuiOutcome::NotAWidget {
            feature: "no-declarations",
        },
    }
}

fn scanned_rule(selector_path: &str) -> ScannedRule {
    ScannedRule {
        selector_path: selector_path.to_string(),
        properties: Vec::new(),
        kind: ScanKind::Rule,
    }
}

/// `font-weight` and `grid-template-columns: repeat(2, minmax(0, 1fr))` must
/// write a fresh `gpui::Style` field. A canonical string is not that field.
#[test]
fn font_weight_writes_style_font_weight_field() {
    use splora_frontend_shared::style_proof::{SpecifiedStyle, write_specified_style};
    use splora_native::style_paint::Style;

    let style = match write_specified_style("font-weight", "700") {
        SpecifiedStyle::Written(style) => style,
        SpecifiedStyle::Feature(feature) => panic!(
            "font-weight is still not a Style field write (feature {feature}); a canonical string match is not Style.text.font_weight"
        ),
    };
    let fresh = Style::default();
    assert_ne!(
        style.text.font_weight, fresh.text.font_weight,
        "font-weight did not change Style.text.font_weight from Style::default()"
    );
    let shown = format!("{:?}", style.text.font_weight);
    assert!(
        shown.contains("700"),
        "Style.text.font_weight was {shown}, not FontWeight(700); a canonical string is not the field"
    );
    assert!(
        style.text.color.is_none(),
        "font-weight must not be accepted through the color arm: {shown}"
    );

    let grid = match write_specified_style("grid-template-columns", "repeat(2, minmax(0, 1fr))") {
        SpecifiedStyle::Written(style) => style,
        SpecifiedStyle::Feature(feature) => panic!(
            "grid-template-columns repeat(2, minmax(0, 1fr)) was feature {feature}, not Style.grid_cols"
        ),
    };
    let cols = grid
        .grid_cols
        .expect("grid-template-columns did not write Style.grid_cols");
    assert_eq!(cols.repeat, 2, "grid repeat was {}, not 2", cols.repeat);
    let min_size = format!("{:?}", cols.min_size);
    assert!(
        min_size.contains("Zero"),
        "grid min_size was {min_size}, not Zero; repeat(12, minmax(min-content, 1fr)) is the theme sample"
    );
    assert!(
        grid.grid_location.is_none(),
        "grid-template-columns must not become GridLocation::default()"
    );

    let background = match write_specified_style("background-color", "#000000") {
        SpecifiedStyle::Written(style) => style,
        SpecifiedStyle::Feature(feature) => panic!(
            "background-color is still not a Style field write (feature {feature}); a canonical string match is not Style.background"
        ),
    };
    assert!(
        background.background.is_some(),
        "background-color did not write Style.background"
    );
    assert_ne!(
        background.background, fresh.background,
        "background-color left Style.background at Style::default()"
    );

    let report = prove_style_catalog();
    let compared = report.gpui_verdicts.iter().any(|verdict| {
        verdict.field == "text.font_weight"
            && verdict.feature.is_empty()
            && verdict.specified.trim().eq_ignore_ascii_case("700")
    });
    assert!(
        compared,
        "prove_style_catalog did not compare Style.text.font_weight for font-weight: 700"
    );
}

fn prove_style_catalog() -> ProofReport {
    splora_frontend_shared::style_proof::prove_style_catalog(&upstream_css())
}

fn prove_style_rules(rules: &[&StyleRule]) -> ProofReport {
    if rules.is_empty() {
        return splora_frontend_shared::style_proof::prove_style_rules(
            rules,
            &UpstreamCss::NotCompared(String::new()),
        );
    }
    splora_frontend_shared::style_proof::prove_style_rules(rules, &upstream_css())
}

fn upstream_css() -> UpstreamCss {
    let Some(compiler) = find_sass_compiler() else {
        return UpstreamCss::NotCompared(missing_compiler_failure());
    };
    UpstreamCss::Compared(compile_tree(&compiler))
}

const MISSING_SASS_COMPILER: &str =
    "missing Sass compiler: sass and sassc are not on PATH; the Sass compiler did not run";

enum CompilerLookup {
    Missing,
    Ran,
    Failed,
}

fn missing_compiler_failure() -> String {
    match (lookup_compiler("sass"), lookup_compiler("sassc")) {
        (CompilerLookup::Missing, CompilerLookup::Missing) => MISSING_SASS_COMPILER.to_string(),
        (CompilerLookup::Ran, _) | (_, CompilerLookup::Ran) => {
            "Sass compiler was found but compiled CSS was not compared to leptos_css".to_string()
        }
        _ => "Sass compiler did not run: sass and sassc could not be executed".to_string(),
    }
}

fn lookup_compiler(name: &str) -> CompilerLookup {
    match Command::new(name)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Err(error) if error.kind() == ErrorKind::NotFound => CompilerLookup::Missing,
        Ok(status) if status.success() => CompilerLookup::Ran,
        Ok(_) | Err(_) => CompilerLookup::Failed,
    }
}

/// Development-time Sass binary. Not an appliance package.
fn compiler_candidates() -> Vec<PathBuf> {
    let mut candidates = vec![PathBuf::from("sass"), PathBuf::from("sassc")];
    if let Ok(home) = std::env::var("HOME") {
        let home = PathBuf::from(home);
        candidates.push(home.join(".local/bin/sass"));
        candidates.push(home.join(".local/dart-sass/sass"));
    }
    candidates
}

fn compiler_runs(path: &Path) -> bool {
    Command::new(path)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn find_sass_compiler() -> Option<PathBuf> {
    compiler_candidates()
        .into_iter()
        .find(|path| compiler_runs(path))
}

/// CSS from the development-time compiler for one upstream file.
/// `Err` is the missing-compiler sentence when no binary runs.
fn compile_upstream_scss(scss_file: &Path) -> Result<String, String> {
    let compiler = find_sass_compiler().ok_or_else(|| MISSING_SASS_COMPILER.to_string())?;
    let root = upstream_scss_root()?;
    compile_scss_with(&compiler, scss_file, &root)
}

fn compile_tree(compiler: &Path) -> BTreeMap<String, Result<String, String>> {
    let mut map = BTreeMap::new();
    let Ok(root) = upstream_scss_root() else {
        return map;
    };
    let mut files = Vec::new();
    collect_scss(&root, &mut files);
    files.sort();
    for file in files {
        let source_path = match relative_source(&root, &file) {
            Ok(source_path) => source_path,
            Err(_) => file.display().to_string(),
        };
        map.insert(source_path, compile_scss_with(compiler, &file, &root));
    }
    map
}

fn upstream_scss_root() -> Result<PathBuf, String> {
    let root_rel = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ref/mempool/frontend/src");
    root_rel.canonicalize().map_err(|error| {
        format!(
            "upstream scss root unavailable {}: {error}",
            root_rel.display()
        )
    })
}

fn compile_scss_with(
    compiler: &Path,
    scss_file: &Path,
    load_root: &Path,
) -> Result<String, String> {
    if !scss_file.is_file() {
        return Err(format!("upstream scss missing: {}", scss_file.display()));
    }
    let mut cmd = Command::new(compiler);
    if compiler
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "sassc")
    {
        cmd.arg("-t").arg("expanded");
        cmd.arg("-I").arg(load_root);
        if let Some(parent) = scss_file.parent() {
            cmd.arg("-I").arg(parent);
        }
        cmd.arg(scss_file);
    } else {
        cmd.arg("--style=expanded");
        cmd.arg("--embed-source-map");
        cmd.arg("--source-map-urls=absolute");
        cmd.arg("--no-charset");
        cmd.arg(format!("--load-path={}", load_root.display()));
        cmd.arg("--load-path=/home/hunter/.local/sass-load");
        if let Some(parent) = scss_file.parent() {
            cmd.arg(format!("--load-path={}", parent.display()));
        }
        cmd.arg(scss_file);
    }
    let output = cmd
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("sass compiler did not run: {error}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        return Err(format!("exit {code}: {}", one_line(&err)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn one_line(text: &str) -> String {
    text.replace(['\n', '\r'], " ")
}

fn collect_scss(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            collect_scss(&path, out);
            continue;
        }
        if is_scss(&path) {
            out.push(path);
        }
    }
}

fn is_scss(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".scss"))
        && path.is_file()
}

fn relative_source(root: &Path, file: &Path) -> Result<String, String> {
    let relative = file.strip_prefix(root).map_err(|error| {
        format!(
            "strip {} from upstream scss root {}: {error}",
            file.display(),
            root.display()
        )
    })?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/"))
}
