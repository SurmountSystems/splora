//! Fail-closed check of catalogued Leptos CSS and of GPUI paint.
//!
//! Compiled CSS is data from the caller. This module does not start a
//! process. When that CSS is absent, the failure names that fact. An empty
//! catalog fails. An empty leptos CSS string fails when that rule also has
//! no properties. A leptos CSS value that is only a selector plus `{}` fails
//! in that same case. When the rule has properties, the emitted block
//! contains those `name: value;` declarations.
//!
//! When compiled CSS is supplied, each catalog rule is compared to that CSS
//! for the upstream file. A compiled rule is planned only when
//! its source map names a file under `ref/mempool/frontend`. A rule with no
//! named origin stays planned. `@media` and `@supports` stay outside
//! the selector. Nested `@media` queries merge with ` and `. Specified
//! declarations are the compiler's declarations. Custom properties stay
//! unresolved. `@keyframes` steps are timeline samples, not resting paint.
//! An empty string, or a selector plus `{}` that is not that compiled block,
//! still fails. A missing compiler is still a failure. `scan_scss` text is
//! not compiled CSS. The checked count is every catalog row plus every
//! scanned rule plus every compiled rule. Nothing stops the pass at a fixed
//! count.
//!
//! That pass also returns the leptos stylesheet text and a GPUI verdict for
//! every rule. Every paintable longhand is written onto a fresh
//! `Style::default()`, not the theme sample, and the check reads that field.
//! A painter name or a canonical string does not set the match. Display is
//! Block, Flex, Grid, or None. Overflow is Visible, Clip, Hidden, or Scroll.
//! There is no Auto, so overflow auto stays the named feature and does not
//! write `Overflow`. Cursor strips a trailing ` !important`. `grab` writes
//! `CursorStyle::OpenHand`. `font-weight`, `font-style`, `line-height`,
//! `text-align`, and `white-space` write `text` fields and do not write
//! `text.color`. `GridTemplate` is only `repeat` plus `min_size`. A parsed
//! `repeat(N, minmax(MIN, 1fr))` writes that template. MIN `0` is Zero.
//! A template that does not parse is `unhostable-grid-template` and is not
//! `GridLocation::default()`. A value the field cannot represent is a named
//! unhostable feature, not a field match.
//! Duplicate compiled selectors stay separate blocks. A catalog row matches
//! only when its declarations equal one of those blocks.
//! A grid placement paints only the named axis (`column line 2 / auto`,
//! `column line 1 / line -1`). The other axis stays Auto. A value that does
//! not parse stays a mismatch with a non-empty explanation.
//! An unhostable feature names that feature and does not name a Style field.
//! A rule with no properties and no feature is the unhostable feature
//! `no-declarations`. That name does not invent properties and does not
//! skip the empty leptos CSS checks.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use gpui::{CursorStyle, Display, Overflow, Style};

use crate::style_rule::{GpuiOutcome, StyleRule};
use crate::style_scan::{ScannedRule, scan_scss};

#[path = "style_proof_origin.rs"]
mod style_proof_origin;

pub struct ProofReport {
    pub rules_checked: usize,
    pub failures: Vec<String>,
    /// Stylesheet text `leptos_stylesheet_css` emits. Not an element style attribute.
    pub leptos_verdict: String,
    pub gpui_verdicts: Vec<GpuiVerdict>,
}

pub struct GpuiVerdict {
    pub source_path: &'static str,
    pub selector_path: &'static str,
    /// Style field name. Empty when `feature` names an unhostable feature.
    pub field: &'static str,
    pub specified: &'static str,
    /// gpui variant or canonical paint for this longhand. Empty when `feature` is set.
    pub painter: &'static str,
    /// Unhostable feature name. Empty when `field` is set.
    pub feature: &'static str,
}

/// Compiled CSS for upstream files. The caller runs the compiler.
/// This crate does not start a process.
pub enum UpstreamCss {
    /// No compiler output. `message` is the failure the caller recorded.
    NotCompared(String),
    /// `source_path` to compiler CSS, or the compiler error for that file.
    Compared(BTreeMap<String, Result<String, String>>),
}

pub fn prove_style_catalog(upstream: &UpstreamCss) -> ProofReport {
    let mut report = prove_style_rules(&crate::all_rules(), upstream);
    // The shell `<style>` element is this function, not a second join.
    let emitted = leptos_stylesheet_css();
    if report.leptos_verdict != emitted {
        report
            .failures
            .push("leptos verdict is not leptos_stylesheet_css".to_string());
    }
    report.leptos_verdict = emitted;
    report
}

/// The same compiled-CSS compare as [`prove_style_catalog`], for one supplied list.
/// An empty list fails with `empty catalog` and does not invent rules.
/// `upstream` is compiled CSS. This function does not start a process.
pub fn prove_style_rules(rules: &[&StyleRule], upstream: &UpstreamCss) -> ProofReport {
    let mut failures = Vec::new();
    if rules.is_empty() {
        failures.push("empty catalog".to_string());
        failures.push("empty leptos_css string".to_string());
        return ProofReport {
            rules_checked: 0,
            failures,
            leptos_verdict: String::new(),
            gpui_verdicts: Vec::new(),
        };
    }
    let compiler_present = matches!(upstream, UpstreamCss::Compared(_));
    if let UpstreamCss::NotCompared(message) = upstream {
        if !message.is_empty() {
            failures.push(message.clone());
        }
    }
    let leptos_verdict = stylesheet_css(rules);
    if leptos_verdict.trim().is_empty() {
        failures.push("empty leptos_css string".to_string());
    } else if is_selector_plus_empty_block(&leptos_verdict) {
        failures.push("leptos_css is only a selector plus {}".to_string());
    }
    for rule in rules {
        // The stored stylesheet text. Do not fill it from `properties`.
        if rule.leptos_css.trim().is_empty() {
            failures.push(format!(
                "empty leptos_css source_path={} selector_path={}",
                rule.source_path, rule.selector_path
            ));
        } else if !compiler_present && is_selector_plus_empty_block(rule.leptos_css) {
            failures.push(format!(
                "leptos_css is only a selector plus {{}} source_path={} selector_path={}",
                rule.source_path, rule.selector_path
            ));
        }
    }
    let scanned_rules = compare_selectors_to_scan(rules, &mut failures, !compiler_present);
    let compiled_rules = if let UpstreamCss::Compared(css) = upstream {
        compare_catalog_to_compiled(rules, css, &mut failures)
    } else {
        0
    };
    // Each paintable longhand compares a fresh Style field, not a painter string.
    let gpui_verdicts = gpui_verdicts(rules, &mut failures);
    ProofReport {
        rules_checked: rules.len() + scanned_rules + compiled_rules,
        failures,
        leptos_verdict,
        gpui_verdicts,
    }
}

fn paintable_longhand(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "display"
            | "visibility"
            | "overflow"
            | "overflow-x"
            | "overflow-y"
            | "position"
            | "inset"
            | "top"
            | "right"
            | "bottom"
            | "left"
            | "width"
            | "height"
            | "min-width"
            | "min-height"
            | "max-width"
            | "max-height"
            | "aspect-ratio"
            | "margin"
            | "padding"
            | "border"
            | "border-width"
            | "border-color"
            | "border-style"
            | "border-radius"
            | "box-shadow"
            | "color"
            | "font-size"
            | "font-weight"
            | "font-style"
            | "line-height"
            | "text-align"
            | "white-space"
            | "cursor"
            | "opacity"
            | "flex-direction"
            | "flex-wrap"
            | "flex-basis"
            | "flex-grow"
            | "flex-shrink"
            | "gap"
            | "grid-template-columns"
            | "grid-template-rows"
            | "grid-column"
            | "grid-row"
            | "background"
            | "background-color"
    )
}

pub fn leptos_stylesheet_css() -> String {
    stylesheet_css(&crate::all_rules())
}

fn stylesheet_css(rules: &[&StyleRule]) -> String {
    let mut css = String::new();
    for rule in rules {
        let block = emitted_leptos_block(rule);
        if block.trim().is_empty() {
            continue;
        }
        if !css.is_empty() {
            css.push('\n');
        }
        css.push_str(&block);
    }
    css
}

/// The stored `leptos_css` text. An empty string stays empty. Properties are
/// not copied in to hide that.
fn emitted_leptos_block(rule: &StyleRule) -> String {
    rule.leptos_css.trim().to_string()
}

fn selector_of_empty_block(css: &str) -> Option<&str> {
    if !is_selector_plus_empty_block(css) {
        return None;
    }
    let open = css.find('{')?;
    let selector = css[..open].trim();
    if selector.is_empty() {
        None
    } else {
        Some(selector)
    }
}

/// `scan_scss` is the outside-media form. A catalog row is compared to that
/// path, never to another catalog row. Selector identity is membership of
/// `(source_path, selector_path)` on each side. A scanned path is not
/// consumed, so a second row with that path still passes. A match is not a
/// compiled-CSS result.
fn compare_selectors_to_scan(
    rules: &[&StyleRule],
    failures: &mut Vec<String>,
    record_mismatches: bool,
) -> usize {
    let mut catalog_by_file: BTreeMap<&str, Vec<&StyleRule>> = BTreeMap::new();
    for rule in rules {
        catalog_by_file
            .entry(rule.source_path)
            .or_default()
            .push(*rule);
    }

    let root_rel = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ref/mempool/frontend/src");
    let root = match root_rel.canonicalize() {
        Ok(root) => root,
        Err(error) => {
            if record_mismatches {
                failures.push(format!(
                    "upstream scss root unavailable {}: {error}",
                    root_rel.display()
                ));
                for rule in rules {
                    failures.push(catalog_form_failure(rule.source_path, rule.selector_path));
                }
            }
            return 0;
        }
    };

    let mut files = Vec::new();
    collect_scss(&root, &mut files, failures);
    files.sort();

    let mut scanned_rules = 0usize;
    let mut seen = BTreeSet::new();
    for file in &files {
        let source_path = match relative_source(&root, file) {
            Ok(source_path) => source_path,
            Err(error) => {
                failures.push(error);
                file.display().to_string()
            }
        };
        seen.insert(source_path.clone());
        let text = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) => {
                if record_mismatches {
                    failures.push(format!(
                        "failed to read upstream scss source_path={source_path}: {error}"
                    ));
                    if let Some(rows) = catalog_by_file.get(source_path.as_str()) {
                        for rule in rows {
                            failures
                                .push(catalog_form_failure(rule.source_path, rule.selector_path));
                        }
                    }
                }
                continue;
            }
        };
        let scanned = scan_scss(&text);
        scanned_rules += scanned.len();
        if record_mismatches {
            let rows = catalog_by_file
                .get(source_path.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            push_selector_mismatches(&source_path, &scanned, rows, failures);
        }
    }

    if record_mismatches {
        for (source_path, rows) in &catalog_by_file {
            if seen.contains(*source_path) {
                continue;
            }
            for rule in rows {
                failures.push(catalog_form_failure(rule.source_path, rule.selector_path));
            }
        }
    }
    scanned_rules
}

pub fn push_selector_mismatches(
    source_path: &str,
    scanned: &[ScannedRule],
    catalog_rows: &[&StyleRule],
    failures: &mut Vec<String>,
) {
    let scanned_keys: BTreeSet<(&str, &str)> = scanned
        .iter()
        .map(|rule| (source_path, rule.selector_path.as_str()))
        .collect();
    let catalog_keys: BTreeSet<(&str, &str)> = catalog_rows
        .iter()
        .map(|rule| (rule.source_path, rule.selector_path))
        .collect();
    for rule in catalog_rows {
        let key = (rule.source_path, rule.selector_path);
        if !scanned_keys.contains(&key) {
            failures.push(catalog_form_failure(rule.source_path, rule.selector_path));
        }
    }
    for rule in scanned {
        let key = (source_path, rule.selector_path.as_str());
        if !catalog_keys.contains(&key) {
            failures.push(scanned_form_failure(
                source_path,
                rule.selector_path.as_str(),
            ));
        }
    }
}

fn catalog_form_failure(source_path: &str, selector_path: &str) -> String {
    format!(
        "catalog selector_path is not the scanner outside-media form source_path={source_path} selector_path={selector_path}"
    )
}

fn scanned_form_failure(source_path: &str, selector_path: &str) -> String {
    format!(
        "scanned rule has no catalog selector_path in the scanner outside-media form source_path={source_path} selector_path={selector_path}"
    )
}

fn collect_scss(dir: &Path, out: &mut Vec<PathBuf>, failures: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            failures.push(format!(
                "failed to read scss directory {}: {error}",
                dir.display()
            ));
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                failures.push(format!(
                    "failed to read scss directory entry under {}: {error}",
                    dir.display()
                ));
                continue;
            }
        };
        let path = entry.path();
        if path.is_dir() {
            collect_scss(&path, out, failures);
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

fn upstream_scss_root() -> Result<PathBuf, String> {
    let root_rel = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ref/mempool/frontend/src");
    root_rel.canonicalize().map_err(|error| {
        format!(
            "upstream scss root unavailable {}: {error}",
            root_rel.display()
        )
    })
}

fn compare_catalog_to_compiled(
    rules: &[&StyleRule],
    compiled_by_source: &BTreeMap<String, Result<String, String>>,
    failures: &mut Vec<String>,
) -> usize {
    let mut catalog_by_file: BTreeMap<&str, Vec<&StyleRule>> = BTreeMap::new();
    for rule in rules {
        catalog_by_file
            .entry(rule.source_path)
            .or_default()
            .push(*rule);
    }
    let root = match upstream_scss_root() {
        Ok(root) => root,
        Err(error) => {
            failures.push(error);
            for rule in rules {
                failures.push(compiled_unavailable(rule.source_path, rule.selector_path));
            }
            return 0;
        }
    };
    let mut files = Vec::new();
    collect_scss(&root, &mut files, failures);
    files.sort();
    let mut compiled_rules = 0usize;
    let mut seen = BTreeSet::new();
    for file in &files {
        let source_path = match relative_source(&root, file) {
            Ok(source_path) => source_path,
            Err(error) => {
                failures.push(error);
                file.display().to_string()
            }
        };
        seen.insert(source_path.clone());
        let rows = catalog_by_file
            .get(source_path.as_str())
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let compiled_css = compiled_by_source
            .get(&source_path)
            .cloned()
            .unwrap_or_else(|| {
                Err(format!(
                    "compiled CSS was not supplied for source_path={source_path}"
                ))
            });
        match compiled_css {
            Ok(css) => {
                let compiled = parse_compiled_css(&css);
                let origin_map = style_proof_origin::parse_embedded_source_map(&css);
                let compiled: Vec<CompiledRule> = compiled
                    .into_iter()
                    .filter(|rule| keep_planned_compiled_rule(rule, origin_map.as_ref()))
                    .collect();
                compiled_rules += compiled.len();
                push_compiled_mismatches(&source_path, &compiled, rows, failures);
                match fs::read_to_string(file) {
                    Ok(text) => {
                        let scanned = scan_scss(&text);
                        push_scanned_compiled_mismatches(
                            &source_path,
                            &scanned,
                            &compiled,
                            failures,
                        );
                    }
                    Err(error) => {
                        failures.push(format!(
                            "failed to read upstream scss for compiled compare source_path={source_path}: {error}"
                        ));
                    }
                }
            }
            Err(error) => {
                failures.push(format!(
                    "sass compiler failed source_path={source_path}: {}",
                    one_line(&error)
                ));
                for rule in rows {
                    failures.push(compiled_unavailable(rule.source_path, rule.selector_path));
                }
            }
        }
    }
    for (source_path, rows) in &catalog_by_file {
        if seen.contains(*source_path) {
            continue;
        }
        for rule in rows {
            failures.push(compiled_unavailable(rule.source_path, rule.selector_path));
        }
    }
    compiled_rules
}

fn compiled_unavailable(source_path: &str, selector_path: &str) -> String {
    format!("compiled CSS unavailable source_path={source_path} selector_path={selector_path}")
}

struct CompiledRule {
    selector_path: String,
    declarations: Vec<(String, String)>,
    /// 0-based line and column in the compiler CSS. The source map uses them.
    generated_at: Option<(u32, u32)>,
}

fn keep_planned_compiled_rule(
    rule: &CompiledRule,
    map: Option<&style_proof_origin::EmbeddedSourceMap>,
) -> bool {
    let Some(map) = map else {
        return true;
    };
    let Some((line, column)) = rule.generated_at else {
        return true;
    };
    match style_proof_origin::origin_at(map, line, column) {
        Some(origin) => !style_proof_origin::skip_non_upstream_origin(origin),
        None => true,
    }
}

fn push_compiled_mismatches(
    source_path: &str,
    compiled: &[CompiledRule],
    catalog_rows: &[&StyleRule],
    failures: &mut Vec<String>,
) {
    // Same selector, separate blocks. Do not union their declarations.
    let mut compiled_by: BTreeMap<String, Vec<Vec<(String, String)>>> = BTreeMap::new();
    let mut compiled_display: BTreeMap<String, String> = BTreeMap::new();
    for rule in compiled {
        let key = collapse_ws(&rule.selector_path);
        compiled_display
            .entry(key.clone())
            .or_insert_with(|| rule.selector_path.clone());
        compiled_by
            .entry(key)
            .or_default()
            .push(rule.declarations.clone());
    }
    let mut catalog_by: BTreeMap<String, Vec<&StyleRule>> = BTreeMap::new();
    for rule in catalog_rows {
        catalog_by
            .entry(collapse_ws(rule.selector_path))
            .or_default()
            .push(*rule);
    }
    for rule in catalog_rows {
        let key = collapse_ws(rule.selector_path);
        let emitted = emitted_leptos_block(rule);
        let Some(blocks) = compiled_by.get(&key) else {
            failures.push(format!(
                "catalog selector_path is not in compiled CSS source_path={} selector_path={}",
                rule.source_path, rule.selector_path
            ));
            if is_selector_plus_empty_block(&emitted) {
                failures.push(format!(
                    "invented empty block source_path={} selector_path={}",
                    rule.source_path, rule.selector_path
                ));
            }
            continue;
        };
        if is_selector_plus_empty_block(&emitted) && !blocks.iter().any(|decls| decls.is_empty()) {
            failures.push(format!(
                "invented empty block source_path={} selector_path={}",
                rule.source_path, rule.selector_path
            ));
        }
        let matched = blocks
            .iter()
            .find(|decls| declaration_sets_equal(rule.properties, decls));
        if let Some(decls) = matched {
            if !leptos_matches_compiled(&emitted, rule.selector_path, decls) {
                failures.push(format!(
                    "leptos_css is not the compiled block source_path={} selector_path={}",
                    rule.source_path, rule.selector_path
                ));
            }
            continue;
        }
        for &(name, value) in rule.properties {
            let in_some_block = blocks.iter().any(|decls| {
                decls.iter().any(|(compiled_name, compiled_value)| {
                    specified_decl_eq(name, value, compiled_name, compiled_value)
                })
            });
            if !in_some_block {
                failures.push(format!(
                    "catalog declaration is not in compiled CSS source_path={} selector_path={} property={} value={}",
                    rule.source_path,
                    rule.selector_path,
                    one_line(name),
                    one_line(value)
                ));
            }
        }
        failures.push(format!(
            "catalog declarations do not match compiled CSS source_path={} selector_path={}",
            rule.source_path, rule.selector_path
        ));
        if !blocks
            .iter()
            .any(|decls| leptos_matches_compiled(&emitted, rule.selector_path, decls))
        {
            failures.push(format!(
                "leptos_css is not the compiled block source_path={} selector_path={}",
                rule.source_path, rule.selector_path
            ));
        }
    }
    for (key, blocks) in &compiled_by {
        let shown = compiled_display
            .get(key)
            .map(String::as_str)
            .unwrap_or(key.as_str());
        let Some(rows) = catalog_by.get(key) else {
            failures.push(format!(
                "compiled rule has no catalog selector_path source_path={source_path} selector_path={shown}"
            ));
            continue;
        };
        for decls in blocks {
            if rows
                .iter()
                .any(|rule| declaration_sets_equal(rule.properties, decls))
            {
                continue;
            }
            for (name, value) in decls {
                let covered = rows.iter().any(|rule| {
                    rule.properties.iter().any(|(row_name, row_value)| {
                        specified_decl_eq(row_name, row_value, name, value)
                    })
                });
                if !covered {
                    failures.push(format!(
                        "compiled declaration missing from catalog source_path={source_path} selector_path={shown} property={} value={}",
                        one_line(name),
                        one_line(value)
                    ));
                }
            }
            failures.push(format!(
                "compiled block does not match a catalog row source_path={source_path} selector_path={shown}"
            ));
        }
    }
}

/// A scanned rule with declarations must be one compiled block.
/// Sass drops a rule that has no declarations of its own, so an empty
/// property list is not a compiled block and is not a failure.
fn push_scanned_compiled_mismatches(
    source_path: &str,
    scanned: &[ScannedRule],
    compiled: &[CompiledRule],
    failures: &mut Vec<String>,
) {
    let compiled_keys: BTreeSet<String> = compiled
        .iter()
        .map(|rule| selector_match_key(&rule.selector_path))
        .collect();
    for rule in scanned {
        if rule.properties.is_empty() {
            continue;
        }
        let key = selector_match_key(&rule.selector_path);
        if compiled_keys.contains(&key) {
            continue;
        }
        failures.push(format!(
            "scanned rule is not in compiled CSS source_path={source_path} selector_path={}",
            rule.selector_path
        ));
    }
}

fn leptos_matches_compiled(
    emitted: &str,
    selector_path: &str,
    compiled: &[(String, String)],
) -> bool {
    let emitted = emitted.trim();
    if emitted.is_empty() {
        return false;
    }
    if is_selector_plus_empty_block(emitted) {
        let Some(selector) = selector_of_empty_block(emitted) else {
            return false;
        };
        return compiled.is_empty() && collapse_ws(selector) == collapse_ws(selector_path);
    }
    let (selector, decls) = split_emitted_rule(emitted);
    if let Some(selector) = selector {
        if collapse_ws(&selector) != collapse_ws(selector_path) {
            return false;
        }
    }
    declaration_sets_equal_owned(&decls, compiled)
}

fn split_emitted_rule(emitted: &str) -> (Option<String>, Vec<(String, String)>) {
    if emitted.contains('{') {
        let parsed = parse_compiled_css(emitted);
        if let [rule] = parsed.as_slice() {
            return (Some(rule.selector_path.clone()), rule.declarations.clone());
        }
        return (Some(String::new()), Vec::new());
    }
    (None, parse_declarations(emitted))
}

fn declaration_sets_equal(catalog: &[(&str, &str)], compiled: &[(String, String)]) -> bool {
    let left: BTreeSet<(String, String)> = catalog
        .iter()
        .map(|(name, value)| specified_pair(name, value))
        .collect();
    let right: BTreeSet<(String, String)> = compiled
        .iter()
        .map(|(name, value)| specified_pair(name, value))
        .collect();
    if left != right {
        return false;
    }
    catalog.iter().all(|(name, value)| {
        compiled.iter().any(|(compiled_name, compiled_value)| {
            specified_decl_eq(name, value, compiled_name, compiled_value)
        })
    })
}

fn declaration_sets_equal_owned(left: &[(String, String)], right: &[(String, String)]) -> bool {
    let left_set: BTreeSet<(String, String)> = left
        .iter()
        .map(|(name, value)| specified_pair(name, value))
        .collect();
    let right_set: BTreeSet<(String, String)> = right
        .iter()
        .map(|(name, value)| specified_pair(name, value))
        .collect();
    if left_set != right_set {
        return false;
    }
    left.iter().all(|(name, value)| {
        right.iter().any(|(compiled_name, compiled_value)| {
            specified_decl_eq(name, value, compiled_name, compiled_value)
        })
    })
}

fn specified_decl_eq(
    left_name: &str,
    left_value: &str,
    right_name: &str,
    right_value: &str,
) -> bool {
    if specified_pair(left_name, left_value) != specified_pair(right_name, right_value) {
        return false;
    }
    canonical_specified_agrees(left_name, left_value, right_value)
}

/// Both specified values, already equal as text, also agree when
/// [`canonical_paint`] can read them. The compiler value is the right-hand side.
fn canonical_specified_agrees(name: &str, left_value: &str, right_value: &str) -> bool {
    let Some(sample) = painter_sample(name) else {
        return true;
    };
    let left = canonical_paint(&sample, &normalize_value(&specified_value(left_value)));
    let right = canonical_paint(&sample, &normalize_value(&specified_value(right_value)));
    match (left, right) {
        (Some(left), Some(right)) => left == right,
        _ => true,
    }
}

fn specified_pair(name: &str, value: &str) -> (String, String) {
    (
        collapse_ws(name).to_ascii_lowercase(),
        specified_value(value),
    )
}

/// Specified value as written. `var(--token)` is not resolved.
fn specified_value(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches(';').trim();
    unify_quotes(&collapse_ws(trimmed).to_ascii_lowercase())
}

fn unify_quotes(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' || ch == '\'' {
            let quote = ch;
            out.push('\'');
            while let Some(inner) = chars.next() {
                if inner == '\\' {
                    out.push('\\');
                    if let Some(escaped) = chars.next() {
                        out.push(escaped);
                    }
                    continue;
                }
                if inner == quote {
                    break;
                }
                if inner == '\'' && quote == '"' {
                    out.push('\\');
                }
                out.push(inner);
            }
            out.push('\'');
        } else {
            out.push(ch);
        }
    }
    out
}

/// Spelling shared by the scanner and Dart Sass. The selector is still one
/// block: this does not merge declaration lists and it does not drop a rule.
fn selector_match_key(selector: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = selector.chars().collect();
    let mut index = 0;
    let mut paren = 0i32;
    let mut quote: Option<char> = None;
    while index < chars.len() {
        let ch = chars[index];
        if let Some(opened) = quote {
            out.push(ch);
            if ch == opened {
                quote = None;
            }
            index += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            out.push(ch);
            index += 1;
            continue;
        }
        if ch == '(' {
            paren += 1;
            if out.ends_with("@media") || out.ends_with("@supports") || out.ends_with("@container")
            {
                out.push(' ');
            }
            out.push('(');
            index += 1;
            while index < chars.len() && chars[index].is_whitespace() {
                index += 1;
            }
            continue;
        }
        if ch == ')' {
            if paren > 0 {
                paren -= 1;
            }
            while out.ends_with(char::is_whitespace) {
                out.pop();
            }
            out.push(')');
            index += 1;
            continue;
        }
        if paren == 0 && matches!(ch, '>' | '+' | '~') {
            while out.ends_with(char::is_whitespace) {
                out.pop();
            }
            out.push(' ');
            out.push(ch);
            out.push(' ');
            index += 1;
            while index < chars.len() && chars[index].is_whitespace() {
                index += 1;
            }
            continue;
        }
        out.push(ch);
        index += 1;
    }
    let collapsed = collapse_ws(&out);
    let collapsed = collapsed.replace(" AND ", " and ");
    strip_nth_whitespace(&collapsed)
}

fn strip_nth_whitespace(selector: &str) -> String {
    const FUNCTIONS: &[&str] = &[
        ":nth-child(",
        ":nth-of-type(",
        ":nth-last-child(",
        ":nth-last-of-type(",
    ];
    let mut rest = selector;
    let mut out = String::new();
    while !rest.is_empty() {
        let mut found: Option<(usize, &str)> = None;
        for name in FUNCTIONS {
            if let Some(at) = rest.find(name) {
                if found.is_none_or(|(prev, _)| at < prev) {
                    found = Some((at, name));
                }
            }
        }
        let Some((at, name)) = found else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..at]);
        out.push_str(name);
        rest = &rest[at + name.len()..];
        let mut depth = 1i32;
        let mut inner = String::new();
        for ch in rest.chars() {
            if ch == '(' {
                depth += 1;
                inner.push(ch);
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                inner.push(ch);
            } else if !ch.is_whitespace() {
                inner.push(ch);
            }
        }
        out.push_str(&inner);
        out.push(')');
        let consumed = inner.chars().count()
            + rest
                .chars()
                .take_while(|ch| *ch != ')' || depth > 0)
                .count();
        // `inner` omitted whitespace, so walk the original tail to the matching `)`.
        let mut depth = 1i32;
        let mut bytes = 0;
        for ch in rest.chars() {
            bytes += ch.len_utf8();
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
        }
        rest = &rest[bytes..];
        let _ = consumed;
    }
    out
}

fn collapse_ws(text: &str) -> String {
    let mut out = String::new();
    let mut pending = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !out.is_empty() {
                pending = true;
            }
            continue;
        }
        if pending {
            out.push(' ');
            pending = false;
        }
        out.push(ch);
    }
    out
}

fn parse_compiled_css(css: &str) -> Vec<CompiledRule> {
    let (stripped, index_map) = strip_css_comments_mapped(css);
    let bom = if stripped.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let mut parser = CssParser::new(&stripped[bom..], bom, &index_map, css);
    let mut rules = Vec::new();
    parser.parse_rule_list(&[], &mut rules);
    rules
}

fn strip_css_comments_mapped(css: &str) -> (String, Vec<u32>) {
    let mut out = String::new();
    let mut map = Vec::new();
    let mut chars = css.char_indices().peekable();
    let mut quote: Option<char> = None;
    let mut escape = false;
    while let Some((idx, ch)) = chars.next() {
        if escape {
            push_mapped_char(&mut out, &mut map, idx, ch);
            escape = false;
            continue;
        }
        if let Some(opened) = quote {
            push_mapped_char(&mut out, &mut map, idx, ch);
            if ch == '\\' {
                escape = true;
            } else if ch == opened {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            push_mapped_char(&mut out, &mut map, idx, ch);
            continue;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '*') {
            chars.next();
            while let Some((_, inner)) = chars.next() {
                if inner == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
                    chars.next();
                    break;
                }
            }
            push_mapped_char(&mut out, &mut map, idx, ' ');
            continue;
        }
        push_mapped_char(&mut out, &mut map, idx, ch);
    }
    (out, map)
}

fn push_mapped_char(out: &mut String, map: &mut Vec<u32>, idx: usize, ch: char) {
    let before = out.len();
    out.push(ch);
    for offset in 0..(out.len() - before) {
        map.push((idx + offset) as u32);
    }
}

fn line_and_column(text: &str, byte: usize) -> (u32, u32) {
    let byte = byte.min(text.len());
    let head = &text[..byte];
    let mut line = 0u32;
    let mut column_start = 0usize;
    for (idx, ch) in head.char_indices() {
        if ch == '\n' {
            line = line.saturating_add(1);
            column_start = idx + ch.len_utf8();
        }
    }
    let column = text[column_start..byte]
        .chars()
        .filter(|ch| *ch != '\r')
        .count() as u32;
    (line, column)
}

struct CssParser<'src, 'map> {
    src: &'src str,
    index: usize,
    stripped_base: usize,
    index_map: &'map [u32],
    original: &'map str,
}

impl<'src, 'map> CssParser<'src, 'map> {
    fn new(
        src: &'src str,
        stripped_base: usize,
        index_map: &'map [u32],
        original: &'map str,
    ) -> Self {
        Self {
            src,
            index: 0,
            stripped_base,
            index_map,
            original,
        }
    }

    fn generated_at(&self, index_in_src: usize) -> Option<(u32, u32)> {
        let at = self.stripped_base.checked_add(index_in_src)?;
        let original_byte = usize::try_from(*self.index_map.get(at)?).ok()?;
        if original_byte > self.original.len() {
            return None;
        }
        Some(line_and_column(self.original, original_byte))
    }

    fn peek(&self) -> Option<char> {
        self.src[self.index..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.index += ch.len_utf8();
        Some(ch)
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|ch| ch.is_whitespace()) {
            self.bump();
        }
    }

    fn parse_rule_list(&mut self, outside: &[String], out: &mut Vec<CompiledRule>) {
        loop {
            self.skip_ws();
            if self.peek().is_none() || self.peek() == Some('}') {
                break;
            }
            let before = self.index;
            let prelude_at = self.generated_at(self.index);
            let prelude = self.read_prelude();
            if self.peek() == Some(';') {
                self.bump();
                if self.index == before {
                    break;
                }
                continue;
            }
            if self.peek() != Some('{') {
                break;
            }
            self.bump();
            let body_index = self.index;
            let body = self.read_body();
            if self.peek() == Some('}') {
                self.bump();
            }
            let body_base = self.stripped_base + body_index;
            dispatch_prelude(
                &prelude,
                prelude_at,
                &body,
                body_base,
                outside,
                out,
                self.index_map,
                self.original,
            );
            if self.index == before {
                break;
            }
        }
    }

    fn read_prelude(&mut self) -> String {
        let start = self.index;
        let mut paren = 0i32;
        let mut bracket = 0i32;
        let mut quote: Option<char> = None;
        let mut escape = false;
        while let Some(ch) = self.peek() {
            if escape {
                escape = false;
                self.bump();
                continue;
            }
            if let Some(opened) = quote {
                if ch == '\\' {
                    escape = true;
                } else if ch == opened {
                    quote = None;
                }
                self.bump();
                continue;
            }
            match ch {
                '"' | '\'' => quote = Some(ch),
                '(' => paren += 1,
                ')' => {
                    if paren > 0 {
                        paren -= 1;
                    }
                }
                '[' => bracket += 1,
                ']' => {
                    if bracket > 0 {
                        bracket -= 1;
                    }
                }
                '{' | ';' if paren == 0 && bracket == 0 => break,
                _ => {}
            }
            self.bump();
        }
        self.src[start..self.index].to_string()
    }

    fn read_body(&mut self) -> String {
        let start = self.index;
        let mut depth = 1i32;
        let mut quote: Option<char> = None;
        let mut escape = false;
        while let Some(ch) = self.peek() {
            if escape {
                escape = false;
                self.bump();
                continue;
            }
            if let Some(opened) = quote {
                if ch == '\\' {
                    escape = true;
                } else if ch == opened {
                    quote = None;
                }
                self.bump();
                continue;
            }
            match ch {
                '"' | '\'' => quote = Some(ch),
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return self.src[start..self.index].to_string();
                    }
                }
                _ => {}
            }
            self.bump();
        }
        self.src[start..self.index].to_string()
    }
}

fn dispatch_prelude(
    prelude: &str,
    prelude_at: Option<(u32, u32)>,
    body: &str,
    body_base: usize,
    outside: &[String],
    out: &mut Vec<CompiledRule>,
    index_map: &[u32],
    original: &str,
) {
    let prelude = collapse_ws(prelude.trim());
    if prelude.is_empty() {
        return;
    }
    if starts_at(&prelude, "@keyframes") {
        // Steps are timeline samples. They are not a widget's resting paint.
        parse_keyframe_samples(&prelude, body, body_base, outside, out, index_map, original);
        return;
    }
    if is_conditional_at(&prelude) || (prelude.starts_with('@') && body_has_block(body)) {
        let next = push_outside(outside, &prelude);
        let mut parser = CssParser::new(body, body_base, index_map, original);
        parser.parse_rule_list(&next, out);
        return;
    }
    if body_has_block(body) {
        parse_nested_style(
            &prelude, prelude_at, body, body_base, outside, out, index_map, original,
        );
        return;
    }
    emit_style(&prelude, body, outside, out, prelude_at);
}

fn parse_keyframe_samples(
    prelude: &str,
    body: &str,
    body_base: usize,
    outside: &[String],
    out: &mut Vec<CompiledRule>,
    index_map: &[u32],
    original: &str,
) {
    let name = prelude.strip_prefix("@keyframes").unwrap_or(prelude).trim();
    let mut parser = CssParser::new(body, body_base, index_map, original);
    loop {
        parser.skip_ws();
        if parser.peek().is_none() {
            break;
        }
        let step_at = parser.generated_at(parser.index);
        let step = parser.read_prelude();
        if parser.peek() != Some('{') {
            break;
        }
        parser.bump();
        let decls = parser.read_body();
        if parser.peek() == Some('}') {
            parser.bump();
        }
        let step = collapse_ws(step.trim());
        if step.is_empty() {
            continue;
        }
        let selector = format!("@keyframes {name} {step}");
        emit_style(&selector, &decls, outside, out, step_at);
    }
}

fn parse_nested_style(
    parent: &str,
    parent_at: Option<(u32, u32)>,
    body: &str,
    body_base: usize,
    outside: &[String],
    out: &mut Vec<CompiledRule>,
    index_map: &[u32],
    original: &str,
) {
    let mut parser = CssParser::new(body, body_base, index_map, original);
    let mut decls = String::new();
    loop {
        parser.skip_ws();
        if parser.peek().is_none() {
            break;
        }
        let start = parser.index;
        let child_at = parser.generated_at(start);
        let prelude = parser.read_prelude();
        if parser.peek() == Some(';') {
            parser.bump();
            decls.push_str(&parser.src[start..parser.index]);
            continue;
        }
        if parser.peek() != Some('{') {
            break;
        }
        parser.bump();
        let nested_index = parser.index;
        let nested = parser.read_body();
        if parser.peek() == Some('}') {
            parser.bump();
        }
        let nested_base = parser.stripped_base + nested_index;
        let child = collapse_ws(prelude.trim());
        if child.is_empty() {
            continue;
        }
        if is_conditional_at(&child) {
            let next = push_outside(outside, &child);
            if body_has_block(&nested) {
                parse_nested_style(
                    parent,
                    parent_at,
                    &nested,
                    nested_base,
                    &next,
                    out,
                    index_map,
                    original,
                );
            } else {
                emit_style(parent, &nested, &next, out, parent_at);
            }
            continue;
        }
        let joined = join_nested(parent, &child);
        if body_has_block(&nested) {
            parse_nested_style(
                &joined,
                child_at,
                &nested,
                nested_base,
                outside,
                out,
                index_map,
                original,
            );
        } else {
            emit_style(&joined, &nested, outside, out, child_at);
        }
    }
    if !decls.trim().is_empty() {
        emit_style(parent, &decls, outside, out, parent_at);
    }
}

fn emit_style(
    selector: &str,
    body: &str,
    outside: &[String],
    out: &mut Vec<CompiledRule>,
    generated_at: Option<(u32, u32)>,
) {
    let selector = collapse_ws(selector.trim());
    if selector.is_empty() {
        return;
    }
    out.push(CompiledRule {
        selector_path: render_outside(outside, &selector),
        declarations: parse_declarations(body),
        generated_at,
    });
}

fn render_outside(outside: &[String], selector: &str) -> String {
    let mut body = selector.to_string();
    for at_rule in outside.iter().rev() {
        body = format!("{at_rule} {{ {body} }}");
    }
    body
}

fn push_outside(outside: &[String], prelude: &str) -> Vec<String> {
    let prelude = collapse_ws(prelude.trim());
    let mut next = outside.to_vec();
    if starts_at(&prelude, "@media") {
        if let Some(last) = next.last_mut() {
            if starts_at(last, "@media") {
                let left = media_condition(last);
                let right = media_condition(&prelude);
                *last = format!("@media {left} and {right}");
                return next;
            }
        }
    }
    next.push(prelude);
    next
}

fn media_condition(prelude: &str) -> String {
    prelude
        .trim()
        .strip_prefix("@media")
        .unwrap_or(prelude)
        .trim()
        .to_string()
}

fn is_conditional_at(prelude: &str) -> bool {
    starts_at(prelude, "@media")
        || starts_at(prelude, "@supports")
        || starts_at(prelude, "@container")
}

fn starts_at(selector: &str, name: &str) -> bool {
    let Some(rest) = selector.strip_prefix(name) else {
        return false;
    };
    match rest.chars().next() {
        None => true,
        Some(ch) => ch.is_whitespace() || ch == '(',
    }
}

fn body_has_block(body: &str) -> bool {
    let mut quote: Option<char> = None;
    let mut escape = false;
    for ch in body.chars() {
        if escape {
            escape = false;
            continue;
        }
        if let Some(opened) = quote {
            if ch == '\\' {
                escape = true;
            } else if ch == opened {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '{' => return true,
            _ => {}
        }
    }
    false
}

fn parse_declarations(body: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for part in split_semicolons(body) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some((name, value)) = split_decl(part) else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        out.push((name.to_string(), value.trim().to_string()));
    }
    out
}

fn split_semicolons(body: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut quote: Option<char> = None;
    let mut escape = false;
    for (index, ch) in body.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if let Some(opened) = quote {
            if ch == '\\' {
                escape = true;
            } else if ch == opened {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => paren += 1,
            ')' => {
                if paren > 0 {
                    paren -= 1;
                }
            }
            '[' => bracket += 1,
            ']' => {
                if bracket > 0 {
                    bracket -= 1;
                }
            }
            ';' if paren == 0 && bracket == 0 => {
                out.push(&body[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    out.push(&body[start..]);
    out
}

fn split_decl(part: &str) -> Option<(&str, &str)> {
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut quote: Option<char> = None;
    let mut escape = false;
    for (index, ch) in part.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if let Some(opened) = quote {
            if ch == '\\' {
                escape = true;
            } else if ch == opened {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => paren += 1,
            ')' => {
                if paren > 0 {
                    paren -= 1;
                }
            }
            '[' => bracket += 1,
            ']' => {
                if bracket > 0 {
                    bracket -= 1;
                }
            }
            ':' if paren == 0 && bracket == 0 => {
                return Some((&part[..index], &part[index + ch.len_utf8()..]));
            }
            _ => {}
        }
    }
    None
}

fn join_nested(parent: &str, child: &str) -> String {
    let parents = split_selector_list(parent);
    let children = split_selector_list(child);
    let parents = if parents.is_empty() {
        vec![String::new()]
    } else {
        parents
    };
    let children = if children.is_empty() {
        vec![child.to_string()]
    } else {
        children
    };
    let mut out = String::new();
    for parent_item in &parents {
        for child_item in &children {
            if !out.is_empty() {
                out.push_str(", ");
            }
            if let Some(rest) = child_item.strip_prefix('&') {
                out.push_str(parent_item);
                out.push_str(rest);
            } else if parent_item.is_empty() {
                out.push_str(child_item);
            } else {
                out.push_str(parent_item);
                out.push(' ');
                out.push_str(child_item);
            }
        }
    }
    out
}

fn split_selector_list(selector: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut quote: Option<char> = None;
    let mut split = false;
    for (index, ch) in selector.char_indices() {
        if let Some(opened) = quote {
            if ch == opened {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => paren += 1,
            ')' => {
                if paren > 0 {
                    paren -= 1;
                }
            }
            '[' => bracket += 1,
            ']' => {
                if bracket > 0 {
                    bracket -= 1;
                }
            }
            ',' if paren == 0 && bracket == 0 => {
                split = true;
                let item = selector[start..index].trim();
                if !item.is_empty() {
                    out.push(item.to_string());
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if !split {
        let item = selector.trim();
        if item.is_empty() {
            return Vec::new();
        }
        return vec![item.to_string()];
    }
    let item = selector[start..].trim();
    if !item.is_empty() {
        out.push(item.to_string());
    }
    out
}

/// Style field for one paintable longhand. The painted text is the canonical
/// form of that rule's specified value, not one shared painted value.
struct PainterSample {
    field: &'static str,
    kind: SampleKind,
}

enum SampleKind {
    Keyword,
    Number,
    LengthPx,
    /// `line-height` is a length or a unitless number.
    LengthOrNumber,
    /// `flex-basis` is a length or a keyword such as `auto`.
    LengthOrKeyword,
    Color,
    BoxShadow,
    GridTemplate,
    /// One axis of a grid placement. The other axis stays Auto and is not painted.
    GridAxis(&'static str),
    /// A grid longhand that is not one axis of line indexes. The painter is an
    /// unparsed explanation, not `GridLocation::default()`.
    UnquotedDefault,
}

enum LonghandClass {
    Feature(&'static str),
    Painted {
        field: &'static str,
        painter: &'static str,
        matches: bool,
    },
}

fn gpui_verdicts(rules: &[&StyleRule], failures: &mut Vec<String>) -> Vec<GpuiVerdict> {
    let mut verdicts = Vec::new();
    for rule in rules {
        let before = verdicts.len();
        match rule.gpui {
            GpuiOutcome::NotAWidget { feature } => {
                let named = selector_features(rule.selector_path);
                if named.is_empty() {
                    push_feature(&mut verdicts, failures, rule, feature, "");
                } else {
                    for name in named {
                        push_feature(&mut verdicts, failures, rule, name, "");
                    }
                }
            }
            GpuiOutcome::Paint => {
                // `:host` and `:not` still select an element. `:hover`, `:focus`,
                // `::before`, `@media`, and `keyframes` keep the longhands off the Style.
                if selector_blocks_resting_paint(rule.selector_path) {
                    for feature in selector_features(rule.selector_path) {
                        push_feature(&mut verdicts, failures, rule, feature, "");
                    }
                } else {
                    for &(name, value) in rule.properties {
                        push_longhand(&mut verdicts, failures, rule, name, value);
                    }
                }
            }
        }
        if verdicts.len() == before && no_properties_and_no_feature(rule) {
            push_feature(&mut verdicts, failures, rule, "no-declarations", "");
        }
        if rule
            .selector_path
            .to_ascii_lowercase()
            .contains("@keyframes")
            && verdicts[before..]
                .iter()
                .any(|verdict| !verdict.field.is_empty())
        {
            failures.push(format!(
                "keyframes treated as resting paint source_path={} selector_path={}",
                rule.source_path, rule.selector_path
            ));
        }
        let blocks_resting = selector_blocks_resting_paint(rule.selector_path);
        for token in selector_features(rule.selector_path) {
            if !token.starts_with(':') {
                continue;
            }
            // `:host` and `:not` with no other blocking token are resting paint.
            // `:host:focus-visible`, `:not(:hover)`, and `@media { :host }` still name the token.
            if !token_blocks_resting_paint(token) && !blocks_resting {
                continue;
            }
            let named = verdicts[before..]
                .iter()
                .any(|verdict| verdict.feature == token);
            if !named {
                failures.push(format!(
                    "selector token has no gpui feature source_path={} selector_path={} token={token}",
                    rule.source_path, rule.selector_path
                ));
            }
            if verdicts[before..]
                .iter()
                .any(|verdict| !verdict.field.is_empty())
            {
                failures.push(format!(
                    "selector token painted as resting style source_path={} selector_path={} token={token}",
                    rule.source_path, rule.selector_path
                ));
            }
        }
    }
    verdicts
}

fn no_properties_and_no_feature(rule: &StyleRule) -> bool {
    let named_feature = match rule.gpui {
        GpuiOutcome::NotAWidget { feature } => {
            !feature.is_empty() || !selector_features(rule.selector_path).is_empty()
        }
        GpuiOutcome::Paint => !selector_features(rule.selector_path).is_empty(),
    };
    !named_feature && rule.properties.is_empty()
}

fn push_feature(
    verdicts: &mut Vec<GpuiVerdict>,
    failures: &mut Vec<String>,
    rule: &StyleRule,
    feature: &'static str,
    specified: &'static str,
) {
    if feature.is_empty() {
        failures.push(format!(
            "gpui empty feature name source_path={} selector_path={}",
            rule.source_path, rule.selector_path
        ));
        return;
    }
    if feature == "pseudo" {
        failures.push(format!(
            "gpui feature pseudo is not a selector token source_path={} selector_path={}",
            rule.source_path, rule.selector_path
        ));
        return;
    }
    verdicts.push(GpuiVerdict {
        source_path: rule.source_path,
        selector_path: rule.selector_path,
        field: "",
        specified,
        painter: "",
        feature,
    });
    // A named unhostable feature is the verdict. A paintable longhand still
    // needs a field compare, so that name stays a failure.
    if paintable_longhand(feature) {
        failures.push(format!(
            "gpui feature name has no field compare source_path={} selector_path={} feature={feature}",
            rule.source_path, rule.selector_path
        ));
    }
}

fn push_longhand(
    verdicts: &mut Vec<GpuiVerdict>,
    failures: &mut Vec<String>,
    rule: &StyleRule,
    name: &'static str,
    value: &'static str,
) {
    if name.trim().eq_ignore_ascii_case("border") {
        push_border_shorthand(verdicts, failures, rule, value);
        return;
    }
    match classify_longhand(name, value) {
        LonghandClass::Feature(feature) => {
            push_feature(verdicts, failures, rule, feature, value);
        }
        LonghandClass::Painted {
            field,
            painter,
            matches,
        } => {
            if field.is_empty() {
                failures.push(format!(
                    "gpui paintable longhand has no Style field property={} specified={} source_path={} selector_path={}",
                    one_line(name),
                    one_line(value),
                    rule.source_path,
                    rule.selector_path
                ));
                return;
            }
            verdicts.push(GpuiVerdict {
                source_path: rule.source_path,
                selector_path: rule.selector_path,
                field,
                specified: value,
                painter,
                feature: "",
            });
            if !matches {
                failures.push(mismatch_failure(rule, field, value, painter));
            }
        }
    }
}

fn classify_longhand(name: &'static str, value: &str) -> LonghandClass {
    let lowered = name.trim().to_ascii_lowercase();
    let normalized = normalize_value(value);
    if let Some(feature) = refusal_feature(&lowered, &normalized) {
        return LonghandClass::Feature(feature);
    }
    let Some(sample) = painter_sample(&lowered) else {
        return LonghandClass::Feature(if lowered.is_empty() { "" } else { name });
    };
    if let Some(class) = gpui_field_class(&lowered, &normalized, &sample) {
        return class;
    }
    match write_normalized_longhand(&lowered, &normalized) {
        SpecifiedStyle::Feature(feature) => LonghandClass::Feature(feature),
        SpecifiedStyle::Written(style) => {
            // The match is the Style field, not the canonical string.
            let matches = style_holds_longhand(&lowered, &normalized, &style);
            let painter = intern_paint(painted_field_text(&lowered, &style));
            if painter.is_empty() {
                return LonghandClass::Feature("unhostable-value");
            }
            painted_match(sample.field, painter, matches)
        }
    }
}

fn refusal_feature(name: &str, normalized: &str) -> Option<&'static str> {
    if name.starts_with("--") || normalized.contains("var(--") {
        return Some("custom-property");
    }
    if name == "letter-spacing" {
        return Some("letter-spacing");
    }
    if name == "text-transform" {
        return Some("text-transform");
    }
    if name == "animation" || name.starts_with("animation-") {
        return Some("animation");
    }
    if name == "keyframes" || normalized.contains("@keyframes") {
        return Some("keyframes");
    }
    if normalized.contains("radial-gradient(") {
        return Some("radial-gradient");
    }
    if normalized.contains("conic-gradient(") {
        return Some("conic-gradient");
    }
    if normalized.contains("linear-gradient(") {
        return Some("linear-gradient");
    }
    if name == "position" && normalized == "fixed" {
        return Some("position:fixed");
    }
    if name == "position" && normalized == "sticky" {
        return Some("position:sticky");
    }
    None
}

/// gpui 0.3.5 variants as painter strings. Display is Block, Flex, Grid, None.
/// Overflow is Visible, Clip, Hidden, Scroll. There is no Auto. Cursor is the
/// CursorStyle name. GridTemplate is the parsed repeat and min size, not the
/// theme sample.
fn gpui_field_class(name: &str, normalized: &str, sample: &PainterSample) -> Option<LonghandClass> {
    match sample.field {
        "display" => Some(classify_display(normalized)),
        "overflow" => Some(classify_overflow(name, normalized)),
        "mouse_cursor" => Some(classify_cursor(normalized)),
        "grid_cols" | "grid_rows" => Some(classify_grid_template(sample.field, normalized)),
        _ => None,
    }
}

/// One display, overflow, or cursor longhand on a fresh [`Style`], or a named
/// feature that does not write that field.
pub enum SpecifiedStyle {
    Written(Style),
    Feature(&'static str),
}

/// Paints `name: value` onto [`Style::default`].
/// `overflow-x: auto` is the feature `overflow-x:auto` and does not assign [`Overflow`].
/// `cursor: grab` writes [`CursorStyle::OpenHand`].
pub fn write_specified_style(name: &str, value: &str) -> SpecifiedStyle {
    let name = name.trim().to_ascii_lowercase();
    let normalized = normalize_value(value);
    write_normalized_longhand(&name, &normalized)
}

fn write_normalized_longhand(name: &str, normalized: &str) -> SpecifiedStyle {
    if let Some(feature) = refusal_feature(name, normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    if let Some(feature) = keyword_feature(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    match name {
        "display" => write_display(normalized),
        "overflow" | "overflow-x" | "overflow-y" => write_overflow(name, normalized),
        "cursor" => write_cursor(normalized),
        "visibility" => write_visibility(normalized),
        "position" => write_position(normalized),
        "opacity" => write_opacity(normalized),
        "flex-direction" => write_flex_direction(normalized),
        "flex-wrap" => write_flex_wrap(normalized),
        "flex-grow" => write_number_field(normalized, "flex-grow"),
        "flex-shrink" => write_number_field(normalized, "flex-shrink"),
        "flex-basis" => write_flex_basis(normalized),
        "aspect-ratio" => write_aspect_ratio(normalized),
        "color" => write_text_color(normalized),
        "font-size" => write_font_size(normalized),
        "font-weight" => write_font_weight(normalized),
        "font-style" => write_font_style(normalized),
        "line-height" => write_line_height(normalized),
        "text-align" => write_text_align(normalized),
        "white-space" => write_white_space(normalized),
        "background" | "background-color" => write_background(name, normalized),
        "box-shadow" => write_box_shadow(normalized),
        "border" => write_border_shorthand_style(normalized),
        "border-style" => write_border_style(normalized),
        "grid-template-columns" => write_grid_template("grid_cols", normalized),
        "grid-template-rows" => write_grid_template("grid_rows", normalized),
        "grid-column" => write_grid_axis("column", normalized),
        "grid-row" => write_grid_axis("row", normalized),
        "grid-area" | "grid-column-start" | "grid-column-end" | "grid-row-start"
        | "grid-row-end" => SpecifiedStyle::Feature("unhostable-grid-location"),
        _ if name == "margin" || name.starts_with("margin-") => write_margin(name, normalized),
        _ if name == "padding" || name.starts_with("padding-") => write_padding(name, normalized),
        _ if is_inset_name(name) => write_inset(name, normalized),
        _ if name == "width"
            || name == "height"
            || name.starts_with("min-")
            || name.starts_with("max-") =>
        {
            write_size_axis(name, normalized)
        }
        _ if name == "gap" || name == "row-gap" || name == "column-gap" => {
            write_gap(name, normalized)
        }
        _ if name == "border-width"
            || (name.starts_with("border-") && name.ends_with("-width")) =>
        {
            write_border_width(name, normalized)
        }
        _ if name == "border-color"
            || (name.starts_with("border-") && name.ends_with("-color")) =>
        {
            write_border_color(name, normalized)
        }
        _ if name == "border-radius"
            || (name.starts_with("border-") && name.contains("radius")) =>
        {
            write_radius(name, normalized)
        }
        _ if name.starts_with("background-") => {
            SpecifiedStyle::Feature(background_feature_name(name, normalized))
        }
        _ => SpecifiedStyle::Feature("unmapped"),
    }
}

fn keyword_feature(normalized: &str) -> Option<&'static str> {
    Some(match normalized {
        "inherit" => "inherit",
        "initial" => "initial",
        "unset" => "unset",
        "revert" => "revert",
        "revert-layer" => "revert-layer",
        _ => return None,
    })
}

fn is_inset_name(name: &str) -> bool {
    matches!(name, "inset" | "top" | "right" | "bottom" | "left") || name.starts_with("inset-")
}

fn length_refusal(normalized: &str) -> Option<&'static str> {
    if let Some(feature) = explicit_unhostable(normalized) {
        return Some(feature);
    }
    if has_em_not_rem(normalized) {
        return Some("em");
    }
    if has_viewport_unit(normalized) {
        return Some("viewport-length");
    }
    None
}

fn color_refusal(normalized: &str) -> Option<&'static str> {
    if normalized.contains("currentcolor") {
        return Some("currentcolor");
    }
    if normalized.contains("hsl(") || normalized.contains("hsla(") {
        return Some("hsl");
    }
    length_refusal(normalized)
}

/// The written field equals a second write of the same longhand.
/// Text longhands other than `color` must leave `text.color` empty.
fn style_holds_longhand(name: &str, normalized: &str, style: &Style) -> bool {
    let SpecifiedStyle::Written(again) = write_normalized_longhand(name, normalized) else {
        return false;
    };
    longhand_style_eq(name, style, &again)
}

fn longhand_style_eq(name: &str, left: &Style, right: &Style) -> bool {
    let same = match name {
        "display" => left.display == right.display,
        "visibility" => left.visibility == right.visibility,
        "overflow" => left.overflow == right.overflow,
        "overflow-x" => left.overflow.x == right.overflow.x,
        "overflow-y" => left.overflow.y == right.overflow.y,
        "position" => left.position == right.position,
        "cursor" => left.mouse_cursor == right.mouse_cursor,
        "opacity" => left.opacity == right.opacity,
        "flex-direction" => left.flex_direction == right.flex_direction,
        "flex-wrap" => left.flex_wrap == right.flex_wrap,
        "flex-grow" => left.flex_grow == right.flex_grow,
        "flex-shrink" => left.flex_shrink == right.flex_shrink,
        "flex-basis" => left.flex_basis == right.flex_basis,
        "aspect-ratio" => left.aspect_ratio == right.aspect_ratio,
        "color" => left.text.color == right.text.color,
        "font-size" => left.text.font_size == right.text.font_size,
        "font-weight" => left.text.font_weight == right.text.font_weight,
        "font-style" => left.text.font_style == right.text.font_style,
        "line-height" => left.text.line_height == right.text.line_height,
        "text-align" => left.text.text_align == right.text.text_align,
        "white-space" => left.text.white_space == right.text.white_space,
        "background" | "background-color" => left.background == right.background,
        "box-shadow" => left.box_shadow == right.box_shadow,
        "border" => {
            left.border_widths == right.border_widths
                && left.border_style == right.border_style
                && left.border_color == right.border_color
        }
        "border-style" => left.border_style == right.border_style,
        "grid-template-columns" => {
            left.grid_cols == right.grid_cols && left.grid_location.is_none()
        }
        "grid-template-rows" => left.grid_rows == right.grid_rows && left.grid_location.is_none(),
        "grid-column" | "grid-row" => left.grid_location == right.grid_location,
        _ => box_field_eq(name, left, right),
    };
    same && !text_longhand_used_color(name, left)
}

fn text_longhand_used_color(name: &str, style: &Style) -> bool {
    matches!(
        name,
        "font-weight" | "font-style" | "line-height" | "text-align" | "white-space" | "font-size"
    ) && style.text.color.is_some()
}

fn box_field_eq(name: &str, left: &Style, right: &Style) -> bool {
    if name == "margin" || name.starts_with("margin-") {
        return left.margin == right.margin;
    }
    if name == "padding" || name.starts_with("padding-") {
        return left.padding == right.padding;
    }
    if name == "width" || name == "height" {
        return left.size == right.size;
    }
    if name == "min-width" || name == "min-height" {
        return left.min_size == right.min_size;
    }
    if name == "max-width" || name == "max-height" {
        return left.max_size == right.max_size;
    }
    if name == "gap" || name == "row-gap" || name == "column-gap" {
        return left.gap == right.gap;
    }
    if is_inset_name(name) {
        return left.inset == right.inset;
    }
    if name == "border-width" || (name.starts_with("border-") && name.ends_with("-width")) {
        return left.border_widths == right.border_widths;
    }
    if name == "border-color" || (name.starts_with("border-") && name.ends_with("-color")) {
        return left.border_color == right.border_color;
    }
    if name.contains("radius") {
        return left.corner_radii == right.corner_radii;
    }
    false
}

fn painted_field_text(name: &str, style: &Style) -> String {
    match name {
        "visibility" => format!("{:?}", style.visibility),
        "position" => format!("{:?}", style.position),
        "opacity" => format!("{:?}", style.opacity),
        "flex-direction" => format!("{:?}", style.flex_direction),
        "flex-wrap" => format!("{:?}", style.flex_wrap),
        "flex-grow" => format!("{:?}", style.flex_grow),
        "flex-shrink" => format!("{:?}", style.flex_shrink),
        "flex-basis" => format!("{:?}", style.flex_basis),
        "aspect-ratio" => format!("{:?}", style.aspect_ratio),
        "color" => format!("{:?}", style.text.color),
        "font-size" => format!("{:?}", style.text.font_size),
        "font-weight" => format!("{:?}", style.text.font_weight),
        "font-style" => format!("{:?}", style.text.font_style),
        "line-height" => format!("{:?}", style.text.line_height),
        "text-align" => format!("{:?}", style.text.text_align),
        "white-space" => format!("{:?}", style.text.white_space),
        "background" | "background-color" => format!("{:?}", style.background),
        "box-shadow" => format!("{:?}", style.box_shadow),
        "border-style" => format!("{:?}", style.border_style),
        "grid-template-columns" => format!("{:?}", style.grid_cols),
        "grid-template-rows" => format!("{:?}", style.grid_rows),
        "grid-column" | "grid-row" => format!("{:?}", style.grid_location),
        "margin" => format!("{:?}", style.margin),
        "padding" => format!("{:?}", style.padding),
        "gap" | "row-gap" | "column-gap" => format!("{:?}", style.gap),
        _ if name.starts_with("margin") => format!("{:?}", style.margin),
        _ if name.starts_with("padding") => format!("{:?}", style.padding),
        _ if name == "width" || name == "height" => format!("{:?}", style.size),
        _ if name.starts_with("min-") => format!("{:?}", style.min_size),
        _ if name.starts_with("max-") => format!("{:?}", style.max_size),
        _ if is_inset_name(name) => format!("{:?}", style.inset),
        _ if name.contains("width") => format!("{:?}", style.border_widths),
        _ if name.contains("color") => format!("{:?}", style.border_color),
        _ if name.contains("radius") => format!("{:?}", style.corner_radii),
        _ => format!("Style.{name}"),
    }
}

fn classify_display(normalized: &str) -> LonghandClass {
    let style = match write_display(normalized) {
        SpecifiedStyle::Feature(feature) => return LonghandClass::Feature(feature),
        SpecifiedStyle::Written(style) => style,
    };
    // The painter name is not the match. The match is `style.display`.
    let matches = display_field_equals(&style, normalized);
    painted_match("display", display_painter(style.display), matches)
}

fn write_display(normalized: &str) -> SpecifiedStyle {
    let Some(display) = display_value(normalized) else {
        return SpecifiedStyle::Feature("unhostable-display");
    };
    let mut style = Style::default();
    style.display = display;
    SpecifiedStyle::Written(style)
}

fn display_value(normalized: &str) -> Option<Display> {
    Some(match normalized {
        "block" => Display::Block,
        "flex" => Display::Flex,
        "grid" => Display::Grid,
        "none" => Display::None,
        _ => return None,
    })
}

fn display_field_equals(style: &Style, normalized: &str) -> bool {
    display_value(normalized).is_some_and(|specified| style.display == specified)
}

fn display_painter(display: Display) -> &'static str {
    match display {
        Display::Block => "Block",
        Display::Flex => "Flex",
        Display::Grid => "Grid",
        Display::None => "None",
    }
}

fn classify_overflow(name: &str, normalized: &str) -> LonghandClass {
    let style = match write_overflow(name, normalized) {
        SpecifiedStyle::Feature(feature) => return LonghandClass::Feature(feature),
        SpecifiedStyle::Written(style) => style,
    };
    // The painter name is not the match. The match is `style.overflow`.
    let matches = overflow_field_equals(&style, name, normalized);
    let painted = match name {
        "overflow-y" => style.overflow.y,
        _ => style.overflow.x,
    };
    painted_match("overflow", overflow_painter(painted), matches)
}

fn write_overflow(name: &str, normalized: &str) -> SpecifiedStyle {
    if normalized == "auto" {
        let feature = match name {
            "overflow-x" => "overflow-x:auto",
            "overflow-y" => "overflow-y:auto",
            _ => "overflow:auto",
        };
        return SpecifiedStyle::Feature(feature);
    }
    let Some(overflow) = overflow_value(normalized) else {
        return SpecifiedStyle::Feature("unhostable-overflow");
    };
    let mut style = Style::default();
    match name {
        "overflow-x" => style.overflow.x = overflow,
        "overflow-y" => style.overflow.y = overflow,
        _ => style.overflow = gpui::point(overflow, overflow),
    }
    SpecifiedStyle::Written(style)
}

fn overflow_value(normalized: &str) -> Option<Overflow> {
    Some(match normalized {
        "visible" => Overflow::Visible,
        "clip" => Overflow::Clip,
        "hidden" => Overflow::Hidden,
        "scroll" => Overflow::Scroll,
        _ => return None,
    })
}

fn overflow_field_equals(style: &Style, name: &str, normalized: &str) -> bool {
    let Some(specified) = overflow_value(normalized) else {
        return false;
    };
    match name {
        "overflow-x" => style.overflow.x == specified,
        "overflow-y" => style.overflow.y == specified,
        _ => style.overflow == gpui::point(specified, specified),
    }
}

fn overflow_painter(overflow: Overflow) -> &'static str {
    match overflow {
        Overflow::Visible => "Visible",
        Overflow::Clip => "Clip",
        Overflow::Hidden => "Hidden",
        Overflow::Scroll => "Scroll",
    }
}

fn classify_cursor(normalized: &str) -> LonghandClass {
    let style = match write_cursor(normalized) {
        SpecifiedStyle::Feature(feature) => return LonghandClass::Feature(feature),
        SpecifiedStyle::Written(style) => style,
    };
    // The painter name is not the match. The match is `style.mouse_cursor`.
    let matches = cursor_field_equals(&style, normalized);
    let painter = style
        .mouse_cursor
        .map(cursor_painter)
        .unwrap_or("unhostable-cursor");
    painted_match("mouse_cursor", painter, matches)
}

fn write_cursor(normalized: &str) -> SpecifiedStyle {
    let Some(cursor) = cursor_value(cursor_key(normalized)) else {
        return SpecifiedStyle::Feature("unhostable-cursor");
    };
    let mut style = Style::default();
    style.mouse_cursor = Some(cursor);
    SpecifiedStyle::Written(style)
}

fn cursor_field_equals(style: &Style, normalized: &str) -> bool {
    cursor_value(cursor_key(normalized))
        .is_some_and(|specified| style.mouse_cursor == Some(specified))
}

fn cursor_key(normalized: &str) -> &str {
    normalized
        .strip_suffix(" !important")
        .unwrap_or(normalized)
        .trim()
}

/// CSS cursor names that `CursorStyle` in gpui 0.3.5 documents.
fn cursor_value(normalized: &str) -> Option<CursorStyle> {
    Some(match normalized {
        "default" => CursorStyle::Arrow,
        "text" => CursorStyle::IBeam,
        "crosshair" => CursorStyle::Crosshair,
        "grabbing" => CursorStyle::ClosedHand,
        "grab" => CursorStyle::OpenHand,
        "pointer" => CursorStyle::PointingHand,
        "w-resize" => CursorStyle::ResizeLeft,
        "e-resize" => CursorStyle::ResizeRight,
        "ew-resize" => CursorStyle::ResizeLeftRight,
        "n-resize" => CursorStyle::ResizeUp,
        "s-resize" => CursorStyle::ResizeDown,
        "ns-resize" => CursorStyle::ResizeUpDown,
        "nesw-resize" => CursorStyle::ResizeUpLeftDownRight,
        "nwse-resize" => CursorStyle::ResizeUpRightDownLeft,
        "col-resize" => CursorStyle::ResizeColumn,
        "row-resize" => CursorStyle::ResizeRow,
        "vertical-text" => CursorStyle::IBeamCursorForVerticalLayout,
        "not-allowed" => CursorStyle::OperationNotAllowed,
        "alias" => CursorStyle::DragLink,
        "copy" => CursorStyle::DragCopy,
        "context-menu" => CursorStyle::ContextualMenu,
        _ => return None,
    })
}

fn cursor_painter(cursor: CursorStyle) -> &'static str {
    match cursor {
        CursorStyle::Arrow => "Arrow",
        CursorStyle::IBeam => "IBeam",
        CursorStyle::Crosshair => "Crosshair",
        CursorStyle::ClosedHand => "ClosedHand",
        CursorStyle::OpenHand => "OpenHand",
        CursorStyle::PointingHand => "PointingHand",
        CursorStyle::ResizeLeft => "ResizeLeft",
        CursorStyle::ResizeRight => "ResizeRight",
        CursorStyle::ResizeLeftRight => "ResizeLeftRight",
        CursorStyle::ResizeUp => "ResizeUp",
        CursorStyle::ResizeDown => "ResizeDown",
        CursorStyle::ResizeUpDown => "ResizeUpDown",
        CursorStyle::ResizeUpLeftDownRight => "ResizeUpLeftDownRight",
        CursorStyle::ResizeUpRightDownLeft => "ResizeUpRightDownLeft",
        CursorStyle::ResizeColumn => "ResizeColumn",
        CursorStyle::ResizeRow => "ResizeRow",
        CursorStyle::IBeamCursorForVerticalLayout => "IBeamCursorForVerticalLayout",
        CursorStyle::OperationNotAllowed => "OperationNotAllowed",
        CursorStyle::DragLink => "DragLink",
        CursorStyle::DragCopy => "DragCopy",
        CursorStyle::ContextualMenu => "ContextualMenu",
    }
}

fn classify_grid_template(field: &'static str, normalized: &str) -> LonghandClass {
    let style = match write_grid_template(field, normalized) {
        SpecifiedStyle::Feature(feature) => return LonghandClass::Feature(feature),
        SpecifiedStyle::Written(style) => style,
    };
    let matches = match write_grid_template(field, normalized) {
        SpecifiedStyle::Written(again) => match field {
            "grid_rows" => style.grid_rows == again.grid_rows && style.grid_location.is_none(),
            _ => style.grid_cols == again.grid_cols && style.grid_location.is_none(),
        },
        SpecifiedStyle::Feature(_) => false,
    };
    let painter = intern_paint(match field {
        "grid_rows" => format!("{:?}", style.grid_rows),
        _ => format!("{:?}", style.grid_cols),
    });
    painted_match(field, painter, matches)
}

fn write_grid_template(field: &str, normalized: &str) -> SpecifiedStyle {
    let Some((repeat, min_size)) = parse_grid_template(normalized) else {
        return SpecifiedStyle::Feature("unhostable-grid-template");
    };
    let min_size = match min_size {
        "0" => gpui::GridTemplateMinSize::Zero,
        "min-content" => gpui::GridTemplateMinSize::MinContent,
        "max-content" => gpui::GridTemplateMinSize::MaxContent,
        _ => return SpecifiedStyle::Feature("unhostable-grid-template"),
    };
    let template = gpui::GridTemplate { repeat, min_size };
    let mut style = Style::default();
    match field {
        "grid_rows" => style.grid_rows = Some(template),
        _ => style.grid_cols = Some(template),
    }
    SpecifiedStyle::Written(style)
}

fn write_grid_axis(axis: &str, normalized: &str) -> SpecifiedStyle {
    let Some((start, end)) = grid_placements(normalized) else {
        return SpecifiedStyle::Feature("unhostable-grid-location");
    };
    let mut location = gpui::GridLocation::default();
    match axis {
        "column" => location.column = start..end,
        "row" => location.row = start..end,
        _ => return SpecifiedStyle::Feature("unhostable-grid-location"),
    }
    let mut style = Style::default();
    style.grid_location = Some(location);
    SpecifiedStyle::Written(style)
}

fn grid_placements(normalized: &str) -> Option<(gpui::GridPlacement, gpui::GridPlacement)> {
    if let Some((start, end)) = split_grid_line_pair(normalized) {
        let start = parse_grid_placement(start)?;
        let end = match end {
            Some(end) => parse_grid_placement(end)?,
            None => gpui::GridPlacement::Auto,
        };
        return Some((start, end));
    }
    let start = parse_grid_placement(normalized)?;
    Some((start, gpui::GridPlacement::Auto))
}

fn parse_grid_placement(token: &str) -> Option<gpui::GridPlacement> {
    if token == "auto" {
        return Some(gpui::GridPlacement::Auto);
    }
    if let Some(rest) = token.strip_prefix("span ") {
        let span: u16 = rest.trim().parse().ok()?;
        if span == 0 {
            return None;
        }
        return Some(gpui::GridPlacement::Span(span));
    }
    let index = parse_grid_line_index(token)?;
    Some(gpui::GridPlacement::Line(index))
}

fn write_visibility(normalized: &str) -> SpecifiedStyle {
    let visibility = match normalized {
        "visible" => gpui::Visibility::Visible,
        "hidden" => gpui::Visibility::Hidden,
        _ => return SpecifiedStyle::Feature("unhostable-visibility"),
    };
    let mut style = Style::default();
    style.visibility = visibility;
    SpecifiedStyle::Written(style)
}

fn write_position(normalized: &str) -> SpecifiedStyle {
    let position = match normalized {
        "relative" => gpui::Position::Relative,
        "absolute" => gpui::Position::Absolute,
        _ => return SpecifiedStyle::Feature("unhostable-position"),
    };
    let mut style = Style::default();
    style.position = position;
    SpecifiedStyle::Written(style)
}

fn write_opacity(normalized: &str) -> SpecifiedStyle {
    if !is_plain_number(normalized) {
        return SpecifiedStyle::Feature("unhostable-opacity");
    }
    let Some(number) = parse_f32(normalized) else {
        return SpecifiedStyle::Feature("unhostable-opacity");
    };
    let mut style = Style::default();
    style.opacity = Some(number);
    SpecifiedStyle::Written(style)
}

fn write_flex_direction(normalized: &str) -> SpecifiedStyle {
    let direction = match normalized {
        "row" => gpui::FlexDirection::Row,
        "column" => gpui::FlexDirection::Column,
        "row-reverse" => gpui::FlexDirection::RowReverse,
        "column-reverse" => gpui::FlexDirection::ColumnReverse,
        _ => return SpecifiedStyle::Feature("unhostable-flex-direction"),
    };
    let mut style = Style::default();
    style.flex_direction = direction;
    SpecifiedStyle::Written(style)
}

fn write_flex_wrap(normalized: &str) -> SpecifiedStyle {
    let wrap = match normalized {
        "nowrap" => gpui::FlexWrap::NoWrap,
        "wrap" => gpui::FlexWrap::Wrap,
        "wrap-reverse" => gpui::FlexWrap::WrapReverse,
        _ => return SpecifiedStyle::Feature("unhostable-flex-wrap"),
    };
    let mut style = Style::default();
    style.flex_wrap = wrap;
    SpecifiedStyle::Written(style)
}

fn write_number_field(normalized: &str, name: &str) -> SpecifiedStyle {
    if !is_plain_number(normalized) {
        return SpecifiedStyle::Feature(if name == "flex-shrink" {
            "unhostable-flex-shrink"
        } else {
            "unhostable-flex-grow"
        });
    }
    let Some(number) = parse_f32(normalized) else {
        return SpecifiedStyle::Feature("unhostable-value");
    };
    let mut style = Style::default();
    match name {
        "flex-shrink" => style.flex_shrink = number,
        _ => style.flex_grow = number,
    }
    SpecifiedStyle::Written(style)
}

fn write_flex_basis(normalized: &str) -> SpecifiedStyle {
    if let Some(feature) = length_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    let Some(length) = parse_one_len(normalized, true, true) else {
        return SpecifiedStyle::Feature(unhostable_for("flex-basis", normalized));
    };
    let mut style = Style::default();
    style.flex_basis = length;
    SpecifiedStyle::Written(style)
}

fn write_aspect_ratio(normalized: &str) -> SpecifiedStyle {
    let number = if let Some((left, right)) = normalized.split_once('/') {
        let Some(left) = parse_f32(left.trim()) else {
            return SpecifiedStyle::Feature("unhostable-aspect-ratio");
        };
        let Some(right) = parse_f32(right.trim()) else {
            return SpecifiedStyle::Feature("unhostable-aspect-ratio");
        };
        if right == 0.0 {
            return SpecifiedStyle::Feature("unhostable-aspect-ratio");
        }
        left / right
    } else if is_plain_number(normalized) {
        let Some(number) = parse_f32(normalized) else {
            return SpecifiedStyle::Feature("unhostable-aspect-ratio");
        };
        number
    } else {
        return SpecifiedStyle::Feature("unhostable-aspect-ratio");
    };
    if !number.is_finite() {
        return SpecifiedStyle::Feature("unhostable-aspect-ratio");
    }
    let mut style = Style::default();
    style.aspect_ratio = Some(number);
    SpecifiedStyle::Written(style)
}

fn write_font_weight(normalized: &str) -> SpecifiedStyle {
    let weight = if normalized == "normal" {
        gpui::FontWeight::NORMAL
    } else if normalized == "bold" {
        gpui::FontWeight::BOLD
    } else if is_plain_number(normalized) {
        let Some(number) = parse_f32(normalized) else {
            return SpecifiedStyle::Feature("unhostable-font-weight");
        };
        gpui::FontWeight(number)
    } else {
        return SpecifiedStyle::Feature("unhostable-font-weight");
    };
    let mut style = Style::default();
    style.text.font_weight = Some(weight);
    SpecifiedStyle::Written(style)
}

fn write_font_style(normalized: &str) -> SpecifiedStyle {
    let font_style = match normalized {
        "normal" => gpui::FontStyle::Normal,
        "italic" => gpui::FontStyle::Italic,
        "oblique" => gpui::FontStyle::Oblique,
        _ => return SpecifiedStyle::Feature("unhostable-font-style"),
    };
    let mut style = Style::default();
    style.text.font_style = Some(font_style);
    SpecifiedStyle::Written(style)
}

fn write_text_align(normalized: &str) -> SpecifiedStyle {
    let align = match normalized {
        "left" => gpui::TextAlign::Left,
        "center" => gpui::TextAlign::Center,
        "right" => gpui::TextAlign::Right,
        _ => return SpecifiedStyle::Feature("unhostable-text-align"),
    };
    let mut style = Style::default();
    style.text.text_align = Some(align);
    SpecifiedStyle::Written(style)
}

fn write_white_space(normalized: &str) -> SpecifiedStyle {
    let white_space = match normalized {
        "normal" => gpui::WhiteSpace::Normal,
        "nowrap" => gpui::WhiteSpace::Nowrap,
        _ => return SpecifiedStyle::Feature("unhostable-white-space"),
    };
    let mut style = Style::default();
    style.text.white_space = Some(white_space);
    SpecifiedStyle::Written(style)
}

fn write_line_height(normalized: &str) -> SpecifiedStyle {
    if let Some(feature) = length_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    if is_plain_number(normalized) {
        return SpecifiedStyle::Feature("unhostable-line-height");
    }
    let Some(length) = parse_one_def(normalized, true) else {
        return SpecifiedStyle::Feature(unhostable_for("line-height", normalized));
    };
    let mut style = Style::default();
    style.text.line_height = Some(length);
    SpecifiedStyle::Written(style)
}

fn write_font_size(normalized: &str) -> SpecifiedStyle {
    if let Some(feature) = length_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    if normalized.contains('%') {
        return SpecifiedStyle::Feature("percent");
    }
    let Some(length) = parse_abs(normalized) else {
        return SpecifiedStyle::Feature(unhostable_for("font-size", normalized));
    };
    let mut style = Style::default();
    style.text.font_size = Some(length);
    SpecifiedStyle::Written(style)
}

fn write_text_color(normalized: &str) -> SpecifiedStyle {
    if let Some(feature) = color_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    let Some(color) = hsla_from_canonical(normalized) else {
        return SpecifiedStyle::Feature(unhostable_for("color", normalized));
    };
    let mut style = Style::default();
    style.text.color = Some(color);
    SpecifiedStyle::Written(style)
}

fn write_background(name: &str, normalized: &str) -> SpecifiedStyle {
    if let Some(feature) = non_color_background(name, normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    if let Some(feature) = color_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    let Some(color) = hsla_from_canonical(normalized) else {
        return SpecifiedStyle::Feature(unhostable_for(name, normalized));
    };
    let mut style = Style::default();
    style.background = Some(color.into());
    SpecifiedStyle::Written(style)
}

fn write_margin(name: &str, normalized: &str) -> SpecifiedStyle {
    write_len_edges(name, normalized, true, true, EdgeTarget::Margin)
}

fn write_padding(name: &str, normalized: &str) -> SpecifiedStyle {
    if normalized.split_whitespace().any(|token| token == "auto") {
        return SpecifiedStyle::Feature("unhostable-padding");
    }
    write_len_edges(name, normalized, true, false, EdgeTarget::Padding)
}

fn write_inset(name: &str, normalized: &str) -> SpecifiedStyle {
    if name == "inset-block" || name == "inset-inline" {
        if let Some(feature) = length_refusal(normalized) {
            return SpecifiedStyle::Feature(feature);
        }
        if normalized.split_whitespace().count() != 1 {
            return SpecifiedStyle::Feature(unhostable_for(name, normalized));
        }
        let Some(length) = parse_one_len(normalized, true, true) else {
            return SpecifiedStyle::Feature(unhostable_for(name, normalized));
        };
        let mut style = Style::default();
        if name == "inset-block" {
            style.inset.top = length;
            style.inset.bottom = length;
        } else {
            style.inset.left = length;
            style.inset.right = length;
        }
        return SpecifiedStyle::Written(style);
    }
    write_len_edges(name, normalized, true, true, EdgeTarget::Inset)
}

fn write_border_width(name: &str, normalized: &str) -> SpecifiedStyle {
    if normalized.contains('%') {
        return SpecifiedStyle::Feature("percent");
    }
    write_len_edges(name, normalized, false, false, EdgeTarget::BorderWidth)
}

fn write_size_axis(name: &str, normalized: &str) -> SpecifiedStyle {
    if !matches!(
        name,
        "width" | "height" | "min-width" | "min-height" | "max-width" | "max-height"
    ) {
        return SpecifiedStyle::Feature("unmapped");
    }
    if let Some(feature) = length_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    if normalized.split_whitespace().count() != 1 {
        return SpecifiedStyle::Feature(unhostable_for(name, normalized));
    }
    let Some(length) = parse_one_len(normalized, true, true) else {
        return SpecifiedStyle::Feature(unhostable_for(name, normalized));
    };
    let mut style = Style::default();
    match name {
        "width" => style.size.width = length,
        "height" => style.size.height = length,
        "min-width" => style.min_size.width = length,
        "min-height" => style.min_size.height = length,
        "max-width" => style.max_size.width = length,
        "max-height" => style.max_size.height = length,
        _ => return SpecifiedStyle::Feature("unmapped"),
    }
    SpecifiedStyle::Written(style)
}

fn write_gap(name: &str, normalized: &str) -> SpecifiedStyle {
    if let Some(feature) = length_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let mut style = Style::default();
    match name {
        "row-gap" => {
            if tokens.len() != 1 {
                return SpecifiedStyle::Feature(unhostable_for(name, normalized));
            }
            let Some(length) = parse_one_def(tokens[0], true) else {
                return SpecifiedStyle::Feature(unhostable_for(name, normalized));
            };
            style.gap.height = length;
        }
        "column-gap" => {
            if tokens.len() != 1 {
                return SpecifiedStyle::Feature(unhostable_for(name, normalized));
            }
            let Some(length) = parse_one_def(tokens[0], true) else {
                return SpecifiedStyle::Feature(unhostable_for(name, normalized));
            };
            style.gap.width = length;
        }
        "gap" => match tokens.len() {
            1 => {
                let Some(length) = parse_one_def(tokens[0], true) else {
                    return SpecifiedStyle::Feature(unhostable_for(name, normalized));
                };
                style.gap.width = length;
                style.gap.height = length;
            }
            2 => {
                let Some(row) = parse_one_def(tokens[0], true) else {
                    return SpecifiedStyle::Feature(unhostable_for(name, normalized));
                };
                let Some(column) = parse_one_def(tokens[1], true) else {
                    return SpecifiedStyle::Feature(unhostable_for(name, normalized));
                };
                style.gap.height = row;
                style.gap.width = column;
            }
            _ => return SpecifiedStyle::Feature(unhostable_for(name, normalized)),
        },
        _ => return SpecifiedStyle::Feature("unmapped"),
    }
    SpecifiedStyle::Written(style)
}

enum EdgeTarget {
    Margin,
    Padding,
    Inset,
    BorderWidth,
}

fn write_len_edges(
    name: &str,
    normalized: &str,
    allow_percent: bool,
    allow_auto: bool,
    target: EdgeTarget,
) -> SpecifiedStyle {
    if let Some(feature) = logical_edge_feature(name) {
        return SpecifiedStyle::Feature(feature);
    }
    if let Some(feature) = length_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    let shorthand = matches!(
        name,
        "margin" | "padding" | "inset" | "border-width" | "border-radius"
    );
    let mut style = Style::default();
    if shorthand {
        let tokens: Vec<&str> = normalized.split_whitespace().collect();
        if tokens.is_empty() || tokens.len() > 4 {
            return SpecifiedStyle::Feature(unhostable_for(name, normalized));
        }
        match target {
            EdgeTarget::Margin | EdgeTarget::Inset => {
                let mut lengths = Vec::new();
                for token in tokens {
                    let Some(length) = parse_one_len(token, allow_percent, allow_auto) else {
                        return SpecifiedStyle::Feature(unhostable_for(name, normalized));
                    };
                    lengths.push(length);
                }
                let Some(edges) = expand_edges(lengths) else {
                    return SpecifiedStyle::Feature(unhostable_for(name, normalized));
                };
                let [top, right, bottom, left] = edges;
                let edges = gpui::Edges {
                    top,
                    right,
                    bottom,
                    left,
                };
                match target {
                    EdgeTarget::Margin => style.margin = edges,
                    EdgeTarget::Inset => style.inset = edges,
                    _ => return SpecifiedStyle::Feature("unmapped"),
                }
            }
            EdgeTarget::Padding => {
                let mut lengths = Vec::new();
                for token in tokens {
                    let Some(length) = parse_one_def(token, allow_percent) else {
                        return SpecifiedStyle::Feature(unhostable_for(name, normalized));
                    };
                    lengths.push(length);
                }
                let Some(edges) = expand_edges(lengths) else {
                    return SpecifiedStyle::Feature(unhostable_for(name, normalized));
                };
                let [top, right, bottom, left] = edges;
                style.padding = gpui::Edges {
                    top,
                    right,
                    bottom,
                    left,
                };
            }
            EdgeTarget::BorderWidth => {
                let mut lengths = Vec::new();
                for token in tokens {
                    let Some(length) = parse_abs(token) else {
                        return SpecifiedStyle::Feature(unhostable_for(name, normalized));
                    };
                    lengths.push(length);
                }
                let Some(edges) = expand_edges(lengths) else {
                    return SpecifiedStyle::Feature(unhostable_for(name, normalized));
                };
                let [top, right, bottom, left] = edges;
                style.border_widths = gpui::Edges {
                    top,
                    right,
                    bottom,
                    left,
                };
            }
        }
        return SpecifiedStyle::Written(style);
    }
    if normalized.split_whitespace().count() != 1 {
        return SpecifiedStyle::Feature(unhostable_for(name, normalized));
    }
    match target {
        EdgeTarget::Margin | EdgeTarget::Inset => {
            let Some(length) = parse_one_len(normalized, allow_percent, allow_auto) else {
                return SpecifiedStyle::Feature(unhostable_for(name, normalized));
            };
            assign_len_side(&mut style, name, length, target);
        }
        EdgeTarget::Padding => {
            let Some(length) = parse_one_def(normalized, allow_percent) else {
                return SpecifiedStyle::Feature(unhostable_for(name, normalized));
            };
            assign_def_side(&mut style, name, length);
        }
        EdgeTarget::BorderWidth => {
            let Some(length) = parse_abs(normalized) else {
                return SpecifiedStyle::Feature(unhostable_for(name, normalized));
            };
            assign_abs_side(&mut style, name, length);
        }
    }
    SpecifiedStyle::Written(style)
}

fn assign_len_side(style: &mut Style, name: &str, length: gpui::Length, target: EdgeTarget) {
    let side = side_of(name);
    match target {
        EdgeTarget::Margin => match side {
            "top" => style.margin.top = length,
            "right" => style.margin.right = length,
            "bottom" => style.margin.bottom = length,
            _ => style.margin.left = length,
        },
        EdgeTarget::Inset => match side {
            "top" => style.inset.top = length,
            "right" => style.inset.right = length,
            "bottom" => style.inset.bottom = length,
            _ => style.inset.left = length,
        },
        _ => {}
    }
}

fn assign_def_side(style: &mut Style, name: &str, length: gpui::DefiniteLength) {
    match side_of(name) {
        "top" => style.padding.top = length,
        "right" => style.padding.right = length,
        "bottom" => style.padding.bottom = length,
        _ => style.padding.left = length,
    }
}

fn assign_abs_side(style: &mut Style, name: &str, length: gpui::AbsoluteLength) {
    match side_of(name) {
        "top" => style.border_widths.top = length,
        "right" => style.border_widths.right = length,
        "bottom" => style.border_widths.bottom = length,
        _ => style.border_widths.left = length,
    }
}

/// GPUI edges are physical top, right, bottom, and left. A logical
/// margin, padding, or border side has no field of its own.
fn logical_edge_feature(name: &str) -> Option<&'static str> {
    if name.starts_with("inset") {
        return None;
    }
    let inline = name.contains("inline");
    let block = name.contains("block");
    let logical_side = name.contains("-start") || name.contains("-end");
    if !inline && !block && !logical_side {
        return None;
    }
    Some(if name.starts_with("margin") {
        if inline {
            "margin-inline"
        } else if block {
            "margin-block"
        } else {
            "margin-logical"
        }
    } else if name.starts_with("padding") {
        if inline {
            "padding-inline"
        } else if block {
            "padding-block"
        } else {
            "padding-logical"
        }
    } else if name.starts_with("border") {
        if inline {
            "border-inline"
        } else if block {
            "border-block"
        } else {
            "border-logical"
        }
    } else {
        "logical-edge"
    })
}

fn side_of(name: &str) -> &'static str {
    if name.contains("top") || name == "inset-block-start" {
        "top"
    } else if name.contains("right") || name == "inset-inline-end" {
        "right"
    } else if name.contains("bottom") || name == "inset-block-end" {
        "bottom"
    } else if name.contains("left") || name == "inset-inline-start" {
        "left"
    } else if name == "inset-block" {
        "block"
    } else if name == "inset-inline" {
        "inline"
    } else {
        "left"
    }
}

fn expand_edges<T: Clone>(lengths: Vec<T>) -> Option<[T; 4]> {
    match lengths.len() {
        1 => Some([
            lengths[0].clone(),
            lengths[0].clone(),
            lengths[0].clone(),
            lengths[0].clone(),
        ]),
        2 => Some([
            lengths[0].clone(),
            lengths[1].clone(),
            lengths[0].clone(),
            lengths[1].clone(),
        ]),
        3 => Some([
            lengths[0].clone(),
            lengths[1].clone(),
            lengths[2].clone(),
            lengths[1].clone(),
        ]),
        4 => Some([
            lengths[0].clone(),
            lengths[1].clone(),
            lengths[2].clone(),
            lengths[3].clone(),
        ]),
        _ => None,
    }
}

fn write_radius(name: &str, normalized: &str) -> SpecifiedStyle {
    if normalized.contains('/') {
        return SpecifiedStyle::Feature("unhostable-radius");
    }
    if let Some(feature) = length_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    if normalized.contains('%') {
        return SpecifiedStyle::Feature("percent");
    }
    let mut style = Style::default();
    if name == "border-radius" {
        let tokens: Vec<&str> = normalized.split_whitespace().collect();
        let mut lengths = Vec::new();
        for token in tokens {
            let Some(length) = parse_abs(token) else {
                return SpecifiedStyle::Feature(unhostable_for(name, normalized));
            };
            lengths.push(length);
        }
        let Some(corners) = expand_edges(lengths) else {
            return SpecifiedStyle::Feature(unhostable_for(name, normalized));
        };
        let [top_left, top_right, bottom_right, bottom_left] = corners;
        style.corner_radii = gpui::Corners {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        };
        return SpecifiedStyle::Written(style);
    }
    if normalized.split_whitespace().count() != 1 {
        return SpecifiedStyle::Feature(unhostable_for(name, normalized));
    }
    let Some(length) = parse_abs(normalized) else {
        return SpecifiedStyle::Feature(unhostable_for(name, normalized));
    };
    match name {
        "border-top-left-radius" => style.corner_radii.top_left = length,
        "border-top-right-radius" => style.corner_radii.top_right = length,
        "border-bottom-right-radius" => style.corner_radii.bottom_right = length,
        "border-bottom-left-radius" => style.corner_radii.bottom_left = length,
        _ => return SpecifiedStyle::Feature("unhostable-radius"),
    }
    SpecifiedStyle::Written(style)
}

fn write_border_style(normalized: &str) -> SpecifiedStyle {
    let tokens = tokenize_css_value(normalized);
    if tokens.is_empty() {
        return SpecifiedStyle::Feature("unhostable-border-style");
    }
    let mut styles = Vec::new();
    for token in &tokens {
        let border_style = match token.as_str() {
            "solid" => gpui::BorderStyle::Solid,
            "dashed" => gpui::BorderStyle::Dashed,
            _ => return SpecifiedStyle::Feature("unhostable-border-style"),
        };
        styles.push(border_style);
    }
    if styles.iter().any(|border_style| *border_style != styles[0]) {
        return SpecifiedStyle::Feature("unhostable-border-style");
    }
    let mut style = Style::default();
    style.border_style = styles[0];
    SpecifiedStyle::Written(style)
}

fn write_border_color(name: &str, normalized: &str) -> SpecifiedStyle {
    if let Some(feature) = logical_edge_feature(name) {
        return SpecifiedStyle::Feature(feature);
    }
    if let Some(feature) = color_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    let tokens = tokenize_css_value(normalized);
    if tokens.is_empty() {
        return SpecifiedStyle::Feature(unhostable_for(name, normalized));
    }
    let mut colors = Vec::new();
    for token in &tokens {
        let Some(color) = hsla_from_canonical(token) else {
            return SpecifiedStyle::Feature(unhostable_for(name, normalized));
        };
        colors.push(color);
    }
    if name == "border-color" && colors.iter().any(|color| *color != colors[0]) {
        return SpecifiedStyle::Feature("unhostable-border-color");
    }
    if name != "border-color" && colors.len() != 1 {
        return SpecifiedStyle::Feature("unhostable-border-color");
    }
    let mut style = Style::default();
    style.border_color = Some(colors[0]);
    SpecifiedStyle::Written(style)
}

fn write_border_shorthand_style(normalized: &str) -> SpecifiedStyle {
    if let Some(feature) = explicit_unhostable(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    if let Some(feature) = color_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    let Some(parts) = split_border(normalized) else {
        return SpecifiedStyle::Feature("unhostable-border");
    };
    if parts.width.is_none() && parts.style.is_none() && parts.color.is_none() {
        return SpecifiedStyle::Feature("unhostable-border");
    }
    let mut style = Style::default();
    if let Some(width) = parts.width {
        let SpecifiedStyle::Written(painted) = write_normalized_longhand("border-width", &width)
        else {
            return SpecifiedStyle::Feature("unhostable-border");
        };
        style.border_widths = painted.border_widths;
    }
    if let Some(border_style) = parts.style {
        let SpecifiedStyle::Written(painted) =
            write_normalized_longhand("border-style", &border_style)
        else {
            return SpecifiedStyle::Feature("unhostable-border");
        };
        style.border_style = painted.border_style;
    }
    if let Some(color) = parts.color {
        let SpecifiedStyle::Written(painted) = write_normalized_longhand("border-color", &color)
        else {
            return SpecifiedStyle::Feature("unhostable-border");
        };
        style.border_color = painted.border_color;
    }
    SpecifiedStyle::Written(style)
}

fn write_box_shadow(normalized: &str) -> SpecifiedStyle {
    if normalized == "none" {
        return SpecifiedStyle::Written(Style::default());
    }
    if let Some(feature) = length_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    if let Some(feature) = color_refusal(normalized) {
        return SpecifiedStyle::Feature(feature);
    }
    let mut shadows = Vec::new();
    for part in split_top_level_commas(normalized) {
        let Some(shadow) = parse_one_shadow(part.trim()) else {
            return SpecifiedStyle::Feature("unhostable-box-shadow");
        };
        shadows.push(shadow);
    }
    if shadows.is_empty() {
        return SpecifiedStyle::Feature("unhostable-box-shadow");
    }
    let mut style = Style::default();
    style.box_shadow = shadows;
    SpecifiedStyle::Written(style)
}

fn split_top_level_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&text[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
}

fn parse_one_shadow(part: &str) -> Option<gpui::BoxShadow> {
    let mut tokens = tokenize_css_value(part);
    if tokens.is_empty() {
        return None;
    }
    let mut inset = false;
    tokens.retain(|token| {
        if token == "inset" {
            inset = true;
            false
        } else {
            true
        }
    });
    let color_at = tokens
        .iter()
        .position(|token| canonical_color(token).is_some())?;
    let color = hsla_from_canonical(&tokens[color_at])?;
    tokens.remove(color_at);
    if !(2..=4).contains(&tokens.len()) {
        return None;
    }
    let mut lengths = Vec::new();
    for token in &tokens {
        lengths.push(parse_px_pixels(token)?);
    }
    let offset_x = lengths[0];
    let offset_y = lengths[1];
    let blur = lengths.get(2).copied().unwrap_or_else(|| gpui::px(0.0));
    let spread = lengths.get(3).copied().unwrap_or_else(|| gpui::px(0.0));
    let mut shadow = gpui::BoxShadow::new(offset_x, offset_y, color)
        .blur_radius(blur)
        .spread_radius(spread);
    if inset {
        shadow = shadow.inset();
    }
    Some(shadow)
}

fn parse_px_pixels(token: &str) -> Option<gpui::Pixels> {
    if token == "0" {
        return Some(gpui::px(0.0));
    }
    let number = token.strip_suffix("px")?;
    Some(gpui::px(parse_f32(number)?))
}

fn parse_abs(token: &str) -> Option<gpui::AbsoluteLength> {
    if token == "0" {
        return Some(gpui::px(0.0).into());
    }
    if let Some(number) = token.strip_suffix("px") {
        return Some(gpui::px(parse_f32(number)?).into());
    }
    if let Some(number) = token.strip_suffix("rem") {
        return Some(gpui::rems(parse_f32(number)?).into());
    }
    None
}

fn parse_one_def(token: &str, allow_percent: bool) -> Option<gpui::DefiniteLength> {
    if let Some(length) = parse_abs(token) {
        return Some(length.into());
    }
    if !allow_percent {
        return None;
    }
    let number = token.strip_suffix('%')?;
    let percent = parse_f32(number)?;
    Some(gpui::relative(percent / 100.0))
}

fn parse_one_len(token: &str, allow_percent: bool, allow_auto: bool) -> Option<gpui::Length> {
    if token == "auto" {
        if allow_auto {
            return Some(gpui::Length::Auto);
        }
        return None;
    }
    parse_one_def(token, allow_percent).map(gpui::Length::from)
}

fn parse_f32(text: &str) -> Option<f32> {
    let number = parse_f64(text)? as f32;
    if number.is_finite() {
        Some(number)
    } else {
        None
    }
}

fn hsla_from_canonical(value: &str) -> Option<gpui::Hsla> {
    let hex = canonical_color(value)?;
    hsla_from_hex(&hex)
}

fn hsla_from_hex(hex: &str) -> Option<gpui::Hsla> {
    if hex.len() == 6 {
        let number = u32::from_str_radix(hex, 16).ok()?;
        return Some(gpui::rgb(number).into());
    }
    if hex.len() == 8 {
        let number = u32::from_str_radix(hex, 16).ok()?;
        return Some(gpui::rgba(number).into());
    }
    None
}

/// `repeat(N, minmax(MIN, 1fr))` only. MIN is 0, min-content, or max-content.
fn parse_grid_template(normalized: &str) -> Option<(u16, &'static str)> {
    let rest = normalized.strip_prefix("repeat(")?;
    let rest = rest.strip_suffix(')')?;
    let (count, minmax) = split_once_top_level_comma(rest)?;
    let count: u16 = count.trim().parse().ok()?;
    let minmax = minmax.trim();
    let inner = minmax.strip_prefix("minmax(")?.strip_suffix(')')?;
    let (min, track) = split_once_top_level_comma(inner)?;
    if track.trim() != "1fr" {
        return None;
    }
    let min_size = match min.trim() {
        "0" | "0px" => "0",
        "min-content" => "min-content",
        "max-content" => "max-content",
        _ => return None,
    };
    Some((count, min_size))
}

fn split_once_top_level_comma(text: &str) -> Option<(&str, &str)> {
    let mut depth = 0i32;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                return Some((&text[..index], &text[index + ch.len_utf8()..]));
            }
            _ => {}
        }
    }
    None
}

fn painted_match(field: &'static str, painter: &'static str, matches: bool) -> LonghandClass {
    LonghandClass::Painted {
        field,
        painter,
        matches,
    }
}

/// `border` is width, style, and color. A value that cannot be split is an
/// unhostable feature, not a paintable name with no field compare.
fn push_border_shorthand(
    verdicts: &mut Vec<GpuiVerdict>,
    failures: &mut Vec<String>,
    rule: &StyleRule,
    value: &'static str,
) {
    let normalized = normalize_value(value);
    if let Some(feature) = explicit_unhostable(&normalized) {
        push_feature(verdicts, failures, rule, feature, value);
        return;
    }
    let Some(parts) = split_border(&normalized) else {
        push_feature(verdicts, failures, rule, "unhostable-border", value);
        return;
    };
    if parts.width.is_none() && parts.style.is_none() && parts.color.is_none() {
        push_feature(verdicts, failures, rule, "unhostable-border", value);
        return;
    }
    if let Some(width) = parts.width {
        push_border_part(
            verdicts,
            failures,
            rule,
            "border-width",
            "border_widths",
            value,
            &width,
        );
    }
    if let Some(style) = parts.style {
        push_border_part(
            verdicts,
            failures,
            rule,
            "border-style",
            "border_style",
            value,
            &style,
        );
    }
    if let Some(color) = parts.color {
        push_border_part(
            verdicts,
            failures,
            rule,
            "border-color",
            "border_color",
            value,
            &color,
        );
    }
}

fn push_border_part(
    verdicts: &mut Vec<GpuiVerdict>,
    failures: &mut Vec<String>,
    rule: &StyleRule,
    css_name: &str,
    field: &'static str,
    original: &'static str,
    part: &str,
) {
    match write_normalized_longhand(css_name, part) {
        SpecifiedStyle::Feature(feature) => {
            push_feature(verdicts, failures, rule, feature, original);
        }
        SpecifiedStyle::Written(style) => {
            let matches = style_holds_longhand(css_name, part, &style);
            let painter = intern_paint(painted_field_text(css_name, &style));
            push_painted(verdicts, failures, rule, field, original, painter, matches);
        }
    }
}

fn push_painted(
    verdicts: &mut Vec<GpuiVerdict>,
    failures: &mut Vec<String>,
    rule: &StyleRule,
    field: &'static str,
    value: &'static str,
    painter: &'static str,
    matches: bool,
) {
    if painter.is_empty() {
        push_feature(verdicts, failures, rule, "unhostable-value", value);
        return;
    }
    verdicts.push(GpuiVerdict {
        source_path: rule.source_path,
        selector_path: rule.selector_path,
        field,
        specified: value,
        painter,
        feature: "",
    });
    if !matches {
        failures.push(mismatch_failure(rule, field, value, painter));
    }
}

struct BorderParts {
    width: Option<String>,
    style: Option<String>,
    color: Option<String>,
}

fn split_border(normalized: &str) -> Option<BorderParts> {
    let tokens = tokenize_css_value(normalized);
    if tokens.is_empty() {
        return None;
    }
    let mut width = None;
    let mut style = None;
    let mut color = None;
    for token in tokens {
        if style.is_none() && is_border_style(&token) {
            style = Some(token);
            continue;
        }
        if width.is_none() {
            if let Some(length) = canonical_one_length(&token, false) {
                width = Some(length);
                continue;
            }
        }
        if color.is_none() {
            if let Some(hex) = canonical_color(&token) {
                color = Some(format!("#{hex}"));
                continue;
            }
        }
        return None;
    }
    if width.is_none() && style.is_none() && color.is_none() {
        return None;
    }
    Some(BorderParts {
        width,
        style,
        color,
    })
}

fn is_border_style(token: &str) -> bool {
    matches!(
        token,
        "none"
            | "hidden"
            | "dotted"
            | "dashed"
            | "solid"
            | "double"
            | "groove"
            | "ridge"
            | "inset"
            | "outset"
    )
}

fn tokenize_css_value(normalized: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    for ch in normalized.chars() {
        if ch == '(' {
            depth += 1;
            current.push(ch);
            continue;
        }
        if ch == ')' {
            depth = depth.saturating_sub(1);
            current.push(ch);
            continue;
        }
        if ch.is_whitespace() && depth == 0 {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn explicit_unhostable(normalized: &str) -> Option<&'static str> {
    if normalized.contains("calc(") {
        return Some("calc");
    }
    if normalized.contains("-webkit-fill-available") {
        return Some("-webkit-fill-available");
    }
    if normalized.contains("-moz-available") {
        return Some("-moz-available");
    }
    if contains_css_token(normalized, "fit-content") {
        return Some("fit-content");
    }
    if contains_css_token(normalized, "min-content") {
        return Some("min-content");
    }
    if contains_css_token(normalized, "max-content") {
        return Some("max-content");
    }
    None
}

fn contains_css_token(normalized: &str, token: &str) -> bool {
    normalized
        .split(|ch: char| ch.is_whitespace() || ch == '(' || ch == ')' || ch == ',')
        .any(|part| part == token)
}

fn non_color_background(name: &str, normalized: &str) -> Option<&'static str> {
    if name == "background-color" {
        return None;
    }
    if name == "background" {
        if canonical_color(normalized).is_some() {
            return None;
        }
        return Some(background_feature_name(name, normalized));
    }
    if name.starts_with("background-") {
        return Some(background_feature_name(name, normalized));
    }
    None
}

fn background_feature_name(name: &str, normalized: &str) -> &'static str {
    if name == "background-size" {
        return "background-size";
    }
    if name == "background-position" {
        return "background-position";
    }
    if name == "background-image" || normalized.contains("url(") || normalized.contains("gradient(")
    {
        return "background-image";
    }
    if normalized.contains('%') {
        return "background-size";
    }
    if looks_like_two_lengths(normalized) {
        return "background-position";
    }
    match name {
        "background-repeat" => "background-repeat",
        "background-attachment" => "background-attachment",
        "background-clip" => "background-clip",
        "background-origin" => "background-origin",
        _ => "background-unhostable",
    }
}

fn looks_like_two_lengths(normalized: &str) -> bool {
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    tokens.len() == 2
        && tokens.iter().all(|token| {
            canonical_one_length(token, false).is_some()
                || token.trim_end_matches(',').ends_with('%')
        })
}

fn unhostable_for(name: &str, normalized: &str) -> &'static str {
    if let Some(feature) = explicit_unhostable(normalized) {
        return feature;
    }
    if normalized.contains("currentcolor") {
        return "currentcolor";
    }
    if normalized.contains("hsl(") || normalized.contains("hsla(") {
        return "hsl";
    }
    if normalized.contains('%') {
        return "percent";
    }
    if has_em_not_rem(normalized) {
        return "em";
    }
    if has_viewport_unit(normalized) {
        return "viewport-length";
    }
    if name == "background" || name.starts_with("background") {
        return background_feature_name(name, normalized);
    }
    "unhostable-value"
}

fn has_em_not_rem(normalized: &str) -> bool {
    tokenize_css_value(normalized).iter().any(|token| {
        let token = token.trim_end_matches(',');
        token.ends_with("em") && !token.ends_with("rem")
    })
}

fn has_viewport_unit(normalized: &str) -> bool {
    tokenize_css_value(normalized).iter().any(|token| {
        let token = token.trim_end_matches(',');
        token.ends_with("vh")
            || token.ends_with("vw")
            || token.ends_with("vmin")
            || token.ends_with("vmax")
            || token.ends_with("svh")
            || token.ends_with("dvh")
            || token.ends_with("lvh")
            || token.ends_with("vb")
            || token.ends_with("vi")
            || token.ends_with("ch")
            || token.ends_with("ex")
    })
}

/// Colon selector tokens, plus `@media` and `keyframes` when those words appear.
/// `::-webkit-scrollbar` and `:focus` stay those strings. The catalog word `pseudo` is not a token.
pub fn selector_feature_tokens(selector: &str) -> Vec<&'static str> {
    selector_features(selector)
}

/// `:host` is the component box. `:not` still selects an element.
/// Every other colon token, `@media`, and `keyframes` is not resting paint.
/// `:host-context` is not `:host`. `:hover` inside `:not(:hover)` still blocks.
pub fn token_blocks_resting_paint(token: &str) -> bool {
    if token == ":host" || token == ":not" {
        return false;
    }
    token.starts_with(':') || token == "@media" || token == "keyframes"
}

/// True when any token on `selector` is not resting paint.
/// `:host:focus-visible`, `:host ::ng-deep`, `:not(:hover)`, and `@media { :host }` block.
/// A selector whose colon tokens are only `:host` or `:not` does not.
pub fn selector_blocks_resting_paint(selector: &str) -> bool {
    selector_feature_tokens(selector)
        .iter()
        .any(|token| token_blocks_resting_paint(token))
}

fn selector_features(selector: &str) -> Vec<&'static str> {
    let lower = selector.to_ascii_lowercase();
    let mut features = Vec::new();
    if lower.contains("@keyframes") {
        features.push("keyframes");
    }
    if lower.contains("@media") {
        features.push("@media");
    }
    let bytes = lower.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b':' {
            index += 1;
            continue;
        }
        let next = bytes.get(index + 1).copied();
        let pseudo = match next {
            Some(b':') => true,
            Some(b'a'..=b'z' | b'_' | b'-') => true,
            _ => false,
        };
        if !pseudo {
            index += 1;
            continue;
        }
        let start = index;
        index += if next == Some(b':') { 2 } else { 1 };
        let name_at = index;
        while index < bytes.len()
            && (bytes[index].is_ascii_alphanumeric()
                || bytes[index] == b'-'
                || bytes[index] == b'_')
        {
            index += 1;
        }
        if index == name_at {
            index = start + 1;
            continue;
        }
        // The argument stays in the scan so `:not(:disabled)` also names `:disabled`.
        let token = intern_feature(&lower[start..index]);
        if !features.contains(&token) {
            features.push(token);
        }
    }
    features
}

fn intern_feature(token: &str) -> &'static str {
    use std::collections::BTreeMap;
    use std::sync::{LazyLock, Mutex};
    static KNOWN: LazyLock<Mutex<BTreeMap<String, &'static str>>> =
        LazyLock::new(|| Mutex::new(BTreeMap::new()));
    let mut known = match KNOWN.lock() {
        Ok(known) => known,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(existing) = known.get(token) {
        return existing;
    }
    let leaked: &'static str = Box::leak(token.to_string().into_boxed_str());
    known.insert(token.to_string(), leaked);
    leaked
}

fn painter_sample(name: &str) -> Option<PainterSample> {
    match name {
        "display" => Some(host("display", SampleKind::Keyword)),
        "visibility" => Some(host("visibility", SampleKind::Keyword)),
        "overflow" | "overflow-x" | "overflow-y" => Some(host("overflow", SampleKind::Keyword)),
        "position" => Some(host("position", SampleKind::Keyword)),
        "opacity" => Some(host("opacity", SampleKind::Number)),
        "flex-direction" => Some(host("flex_direction", SampleKind::Keyword)),
        "flex-wrap" => Some(host("flex_wrap", SampleKind::Keyword)),
        "flex-grow" => Some(host("flex_grow", SampleKind::Number)),
        "flex-shrink" => Some(host("flex_shrink", SampleKind::Number)),
        "flex-basis" => Some(host("flex_basis", SampleKind::LengthOrKeyword)),
        "aspect-ratio" => Some(host("aspect_ratio", SampleKind::Number)),
        "cursor" => Some(host("mouse_cursor", SampleKind::Keyword)),
        "color" => Some(host("text.color", SampleKind::Color)),
        "box-shadow" => Some(host("box_shadow", SampleKind::BoxShadow)),
        "gap" | "row-gap" | "column-gap" => Some(host("gap", SampleKind::LengthPx)),
        "grid-template-columns" => Some(host("grid_cols", SampleKind::GridTemplate)),
        "grid-template-rows" => Some(host("grid_rows", SampleKind::GridTemplate)),
        "grid-column" => Some(host("grid_location", SampleKind::GridAxis("column"))),
        "grid-row" => Some(host("grid_location", SampleKind::GridAxis("row"))),
        "grid-area" | "grid-column-start" | "grid-column-end" | "grid-row-start"
        | "grid-row-end" => Some(host("grid_location", SampleKind::UnquotedDefault)),
        "font-size" => Some(host("text.font_size", SampleKind::LengthPx)),
        "font-weight" => Some(host("text.font_weight", SampleKind::Number)),
        "font-style" => Some(host("text.font_style", SampleKind::Keyword)),
        "line-height" => Some(host("text.line_height", SampleKind::LengthOrNumber)),
        "text-align" => Some(host("text.text_align", SampleKind::Keyword)),
        "white-space" => Some(host("text.white_space", SampleKind::Keyword)),
        _ => painter_sample_prefix(name),
    }
}

fn painter_sample_prefix(name: &str) -> Option<PainterSample> {
    if name == "margin" || name.starts_with("margin-") {
        return Some(host("margin", SampleKind::LengthPx));
    }
    if name == "padding" || name.starts_with("padding-") {
        return Some(host("padding", SampleKind::LengthPx));
    }
    if name == "top"
        || name == "right"
        || name == "bottom"
        || name == "left"
        || name == "inset"
        || name.starts_with("inset-")
    {
        return Some(host("inset", SampleKind::LengthPx));
    }
    if name == "width" || name == "height" {
        return Some(host("size", SampleKind::LengthPx));
    }
    if name == "min-width" || name == "min-height" {
        return Some(host("min_size", SampleKind::LengthPx));
    }
    if name == "max-width" || name == "max-height" {
        return Some(host("max_size", SampleKind::LengthPx));
    }
    if name == "background" || name.starts_with("background-") {
        return Some(host("background", SampleKind::Color));
    }
    if name.contains("border-image") {
        return None;
    }
    if name == "border-width" || (name.starts_with("border-") && name.ends_with("-width")) {
        return Some(host("border_widths", SampleKind::LengthPx));
    }
    if name == "border-color" || (name.starts_with("border-") && name.ends_with("-color")) {
        return Some(host("border_color", SampleKind::Color));
    }
    if name == "border-style" || (name.starts_with("border-") && name.ends_with("-style")) {
        return Some(host("border_style", SampleKind::Keyword));
    }
    if name == "border-radius" || (name.starts_with("border-") && name.contains("radius")) {
        return Some(host("corner_radii", SampleKind::LengthPx));
    }
    None
}

fn host(field: &'static str, kind: SampleKind) -> PainterSample {
    PainterSample { field, kind }
}

fn canonical_paint(sample: &PainterSample, normalized: &str) -> Option<&'static str> {
    let owned = match sample.kind {
        SampleKind::Keyword => canonical_keyword(normalized)?,
        SampleKind::Number => canonical_number(sample.field, normalized)?,
        SampleKind::LengthPx => {
            if sample.field == "margin" || sample.field == "padding" {
                canonical_edges(sample.field, normalized)?
            } else if normalized == "auto" && matches!(sample.field, "size" | "min_size" | "inset")
            {
                "auto".to_string()
            } else {
                canonical_length(
                    normalized,
                    allows_percent(sample.field),
                    sample.field == "inset",
                )?
            }
        }
        SampleKind::LengthOrNumber => {
            if let Some(length) = canonical_length(normalized, allows_percent(sample.field), false)
            {
                length
            } else {
                canonical_number(sample.field, normalized)?
            }
        }
        SampleKind::LengthOrKeyword => {
            if let Some(length) = canonical_length(normalized, allows_percent(sample.field), false)
            {
                length
            } else {
                canonical_keyword(normalized)?
            }
        }
        SampleKind::Color => {
            let hex = canonical_color(normalized)?;
            format!("#{hex}")
        }
        SampleKind::BoxShadow | SampleKind::GridTemplate => {
            if normalized.is_empty() {
                return None;
            }
            normalized.to_string()
        }
        SampleKind::GridAxis(axis) => canonical_grid_axis(axis, normalized)?,
        SampleKind::UnquotedDefault => return None,
    };
    Some(intern_paint(owned))
}

fn intern_paint(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// Canonical painter for one grid axis. The other axis stays Auto and is not
/// written. End Auto is the word `auto`, because Auto is not a line index.
/// A line index is `line N`, and N keeps its minus sign.
fn canonical_grid_axis(axis: &str, normalized: &str) -> Option<String> {
    if axis != "column" && axis != "row" {
        return None;
    }
    let (start, end) = split_grid_line_pair(normalized)?;
    let start = parse_grid_line_index(start)?;
    let end_paint = match end {
        None => "auto".to_string(),
        Some(end) => {
            let end = parse_grid_line_index(end)?;
            paint_grid_line(end)
        }
    };
    Some(format!("{axis} {} / {end_paint}", paint_grid_line(start)))
}

fn split_grid_line_pair(normalized: &str) -> Option<(&str, Option<&str>)> {
    if normalized.is_empty() {
        return None;
    }
    if let Some((start, end)) = normalized.split_once('/') {
        let start = start.trim();
        let end = end.trim();
        if start.is_empty()
            || end.is_empty()
            || start.contains(char::is_whitespace)
            || end.contains(char::is_whitespace)
            || end.contains('/')
        {
            return None;
        }
        return Some((start, Some(end)));
    }
    if normalized.contains(char::is_whitespace) {
        return None;
    }
    Some((normalized, None))
}

fn parse_grid_line_index(token: &str) -> Option<i16> {
    let mut chars = token.chars();
    match chars.next()? {
        '-' => {
            let mut saw_digit = false;
            for ch in chars {
                if !ch.is_ascii_digit() {
                    return None;
                }
                saw_digit = true;
            }
            if !saw_digit {
                return None;
            }
        }
        first if first.is_ascii_digit() => {
            if chars.any(|ch| !ch.is_ascii_digit()) {
                return None;
            }
        }
        _ => return None,
    }
    token.parse::<i16>().ok()
}

fn paint_grid_line(index: i16) -> String {
    format!("line {index}")
}

fn canonical_keyword(normalized: &str) -> Option<String> {
    let mut count = 0usize;
    for token in normalized.split_whitespace() {
        count += 1;
        let letters = token.chars().any(|ch| ch.is_ascii_lowercase());
        let ident = token.chars().all(|ch| ch.is_ascii_lowercase() || ch == '-');
        if !letters || !ident {
            return None;
        }
    }
    if count == 0 {
        None
    } else {
        Some(normalized.to_string())
    }
}

/// A plain number, or a `/` ratio, is representable. CSS `normal` is weight
/// 400 on `text.font_weight` only. `bold` is 700.
fn canonical_number(field: &str, normalized: &str) -> Option<String> {
    if field == "text.font_weight" && normalized == "normal" {
        return Some("400".to_string());
    }
    if field == "text.font_weight" && normalized == "bold" {
        return Some("700".to_string());
    }
    if let Some((left, right)) = normalized.split_once('/') {
        let left = parse_f64(left)?;
        let right = parse_f64(right)?;
        if right == 0.0 {
            return None;
        }
        return Some(format_finite(left / right));
    }
    if !is_plain_number(normalized) {
        return None;
    }
    parse_f64(normalized)?;
    Some(normalized.to_string())
}

fn canonical_length(normalized: &str, allow_percent: bool, allow_auto: bool) -> Option<String> {
    let mut parts = Vec::new();
    for token in normalized.split_whitespace() {
        if token == "auto" {
            if !allow_auto {
                return None;
            }
            parts.push("auto".to_string());
            continue;
        }
        parts.push(canonical_one_length(token, allow_percent)?);
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

/// `Length` and `DefiniteLength` store a percent. `AbsoluteLength` does not.
/// `auto` is a `Length` variant, legal on margin and inset, not on padding
/// or on `DefiniteLength` (`gap`, line height).
fn allows_percent(field: &str) -> bool {
    matches!(
        field,
        "margin"
            | "padding"
            | "inset"
            | "size"
            | "min_size"
            | "max_size"
            | "gap"
            | "flex_basis"
            | "text.line_height"
    )
}

/// One length GPUI can store. Bare `0` is `0px`. A px length stays px. `rem`
/// stays rem. Percent is a `Length` variant via `Length::try_from`, and
/// `DefiniteLength::Fraction` stores percent/100, so a percent token is
/// canonical when `allow_percent` is set. The painter keeps that percent
/// text: `50%` stays `50%`, and `12.5%` stays `12.5%` because the stored
/// f32 is 0.125, not `Display`'s integer truncation. `AbsoluteLength` is
/// only pixels and rems, so percent is not canonical for that type. `em`
/// is not a variant.
fn canonical_one_length(token: &str, allow_percent: bool) -> Option<String> {
    if token == "0" {
        return Some("0px".to_string());
    }
    if let Some(number) = token.strip_suffix("px") {
        if is_plain_number(number) && parse_f64(number).is_some() {
            return Some(format!("{number}px"));
        }
        return None;
    }
    if let Some(number) = token.strip_suffix("rem") {
        if is_plain_number(number) && parse_f64(number).is_some() {
            return Some(format!("{number}rem"));
        }
        return None;
    }
    if !allow_percent {
        return None;
    }
    let Some(number) = token.strip_suffix('%') else {
        return None;
    };
    if !is_plain_number(number) {
        return None;
    }
    let percent = number.parse::<f32>().ok()?;
    if !percent.is_finite() {
        return None;
    }
    let fraction = percent / 100.0;
    if !fraction.is_finite() {
        return None;
    }
    Some(format!("{number}%"))
}

/// Margin and padding: 1 to 4 lengths become four edges (top, right, bottom,
/// left). `auto` is a `Length` variant, so margin may use it. Padding may not.
/// Percent is legal on both, because margin is `Edges<Length>` and padding
/// is `Edges<DefiniteLength>`.
fn canonical_edges(field: &str, normalized: &str) -> Option<String> {
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    if tokens.is_empty() || tokens.len() > 4 {
        return None;
    }
    let mut one = Vec::new();
    for token in tokens {
        if token == "auto" {
            if field != "margin" {
                return None;
            }
            one.push("auto".to_string());
            continue;
        }
        one.push(canonical_one_length(token, allows_percent(field))?);
    }
    let four = match one.len() {
        1 => vec![
            one[0].clone(),
            one[0].clone(),
            one[0].clone(),
            one[0].clone(),
        ],
        2 => vec![
            one[0].clone(),
            one[1].clone(),
            one[0].clone(),
            one[1].clone(),
        ],
        3 => vec![
            one[0].clone(),
            one[1].clone(),
            one[2].clone(),
            one[1].clone(),
        ],
        4 => one,
        _ => return None,
    };
    Some(four.join(" "))
}

fn is_plain_number(text: &str) -> bool {
    let mut chars = text.chars().peekable();
    if matches!(chars.peek(), Some('+' | '-')) {
        chars.next();
    }
    let mut saw_digit = false;
    let mut saw_dot = false;
    for ch in chars {
        if ch.is_ascii_digit() {
            saw_digit = true;
        } else if ch == '.' && !saw_dot {
            saw_dot = true;
        } else {
            return false;
        }
    }
    saw_digit
}

fn format_finite(number: f64) -> String {
    if !number.is_finite() {
        return "0".to_string();
    }
    if (number - number.round()).abs() < 1e-6 && number.abs() < 1e15 {
        return format!("{}", number.round() as i64);
    }
    let text = format!("{number:.6}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn canonical_color(value: &str) -> Option<String> {
    if let Some(hex) = named_color_hex(value) {
        return Some(hex.to_string());
    }
    if let Some(hex) = value.strip_prefix('#') {
        return expand_hex(hex);
    }
    let body = if let Some(body) = value.strip_prefix("rgba(") {
        body
    } else if let Some(body) = value.strip_prefix("rgb(") {
        body
    } else {
        return None;
    };
    let body = body.strip_suffix(')')?.trim();
    if let Some((channels, alpha)) = body.split_once('/') {
        return rgb_channels(channels, Some(alpha.trim()));
    }
    rgb_channels(body, None)
}

fn rgb_channels(body: &str, alpha: Option<&str>) -> Option<String> {
    let parts: Vec<&str> = if body.contains(',') {
        body.split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect()
    } else {
        body.split_whitespace()
            .filter(|part| !part.is_empty())
            .collect()
    };
    let (red, green, blue, alpha_text) = if parts.len() == 3 {
        (parts[0], parts[1], parts[2], alpha)
    } else if parts.len() == 4 && alpha.is_none() {
        (parts[0], parts[1], parts[2], Some(parts[3]))
    } else {
        return None;
    };
    let red = channel(red)?;
    let green = channel(green)?;
    let blue = channel(blue)?;
    match alpha_text {
        None => Some(format!("{red:02x}{green:02x}{blue:02x}")),
        Some(text) => {
            let alpha = alpha_byte(text)?;
            if alpha == 255 {
                Some(format!("{red:02x}{green:02x}{blue:02x}"))
            } else {
                Some(format!("{red:02x}{green:02x}{blue:02x}{alpha:02x}"))
            }
        }
    }
}

/// `Rgba` and `Hsla` store alpha. `transparent` is black at alpha 0.
fn named_color_hex(name: &str) -> Option<&'static str> {
    Some(match name {
        "black" => "000000",
        "white" => "ffffff",
        "transparent" => "00000000",
        "red" => "ff0000",
        "green" => "008000",
        "blue" => "0000ff",
        "gray" | "grey" => "808080",
        "silver" => "c0c0c0",
        "maroon" => "800000",
        "orange" => "ffa500",
        "yellow" => "ffff00",
        "purple" => "800080",
        "navy" => "000080",
        "teal" => "008080",
        "aqua" | "cyan" => "00ffff",
        "fuchsia" | "magenta" => "ff00ff",
        "lime" => "00ff00",
        "olive" => "808000",
        _ => return None,
    })
}

fn alpha_byte(text: &str) -> Option<u8> {
    let text = text.trim();
    if let Some(percent) = text.strip_suffix('%') {
        let number = parse_f64(percent)?;
        if !(0.0..=100.0).contains(&number) {
            return None;
        }
        return Some((number / 100.0 * 255.0).round() as u8);
    }
    let number = parse_f64(text)?;
    if (0.0..=1.0).contains(&number) {
        return Some((number * 255.0).round() as u8);
    }
    None
}

fn channel(text: &str) -> Option<u8> {
    if let Some(percent) = text.strip_suffix('%') {
        let number = parse_f64(percent)?;
        if !(0.0..=100.0).contains(&number) {
            return None;
        }
        return Some((number / 100.0 * 255.0).round() as u8);
    }
    let number = parse_f64(text)?;
    if !(0.0..=255.0).contains(&number) {
        return None;
    }
    Some(number.round() as u8)
}

fn expand_hex(hex: &str) -> Option<String> {
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match hex.len() {
        3 | 4 => {
            let chars: Vec<char> = hex.chars().collect();
            let rgb = format!(
                "{}{}{}{}{}{}",
                chars[0], chars[0], chars[1], chars[1], chars[2], chars[2]
            );
            if hex.len() == 3 || chars[3] == 'f' {
                Some(rgb)
            } else {
                Some(format!("{rgb}{}{}", chars[3], chars[3]))
            }
        }
        6 => Some(hex.to_string()),
        8 => {
            if &hex[6..] == "ff" {
                Some(hex[..6].to_string())
            } else {
                Some(hex.to_string())
            }
        }
        _ => None,
    }
}

fn parse_f64(text: &str) -> Option<f64> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    text.parse::<f64>().ok().filter(|number| number.is_finite())
}

fn normalize_value(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches(';').trim();
    let lower = trimmed.to_ascii_lowercase();
    let lower = lower.strip_suffix("!important").unwrap_or(&lower).trim();
    let mut out = String::new();
    let mut pending_space = false;
    for ch in lower.chars() {
        if ch.is_whitespace() {
            pending_space = true;
            continue;
        }
        if pending_space && !out.is_empty() {
            out.push(' ');
        }
        pending_space = false;
        out.push(ch);
    }
    out
}

fn mismatch_failure(rule: &StyleRule, field: &str, specified: &str, painter: &str) -> String {
    format!(
        "gpui field={field} specified={} painter={painter} source_path={} selector_path={}",
        one_line(specified),
        rule.source_path,
        rule.selector_path
    )
}

fn one_line(text: &str) -> String {
    text.replace(['\n', '\r'], " ")
}

/// True when the last `{` ... `}` pair is empty and the text before that `{` is not.
/// An earlier `{`, such as `@media <query> { selector }{}`, does not hide that pair.
fn is_selector_plus_empty_block(css: &str) -> bool {
    let trimmed = css.trim();
    let Some(open) = trimmed.rfind('{') else {
        return false;
    };
    let Some(close) = trimmed.rfind('}') else {
        return false;
    };
    if close != trimmed.len() - 1 || close <= open {
        return false;
    }
    trimmed[open + 1..close].trim().is_empty() && !trimmed[..open].trim().is_empty()
}
