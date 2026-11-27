//! Every Sass rule under the mempool frontend must have a catalog row.
//!
//! Existence is source_path plus selector_path. One catalog row may cover
//! duplicate selector paths in the same file. Ordinary rules must paint.
//! Pseudo-elements and keyframes are not widgets, and they need a feature name.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use splora_frontend_shared::style_rule::{GpuiOutcome, StyleRule};
use splora_frontend_shared::style_scan::{ScanKind, scan_scss};

#[test]
fn style_catalog_covers_every_upstream_scss() {
    let root_rel = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ref/mempool/frontend/src");
    let root = root_rel.canonicalize().unwrap_or_else(|error| {
        panic!(
            "canonicalize upstream scss root {}: {error}",
            root_rel.display()
        )
    });

    let mut files = Vec::new();
    collect_scss(&root, &mut files);
    files.sort();
    println!("scss file count: {}", files.len());
    assert_eq!(
        files.len(),
        207,
        "upstream scss file count must be 207, found {} under {}",
        files.len(),
        root.display()
    );

    let mut by_key: HashMap<(&str, &str), Vec<&StyleRule>> = HashMap::new();
    for rule in splora_frontend_shared::all_rules() {
        by_key
            .entry((rule.source_path, rule.selector_path))
            .or_default()
            .push(rule);
    }

    let mut misses = Vec::new();
    for file in &files {
        let source_path = relative_source(&root, file);
        let text = fs::read_to_string(file)
            .unwrap_or_else(|error| panic!("read {} ({source_path}): {error}", file.display()));
        for scanned in scan_scss(&text) {
            let Some(matched) = by_key.get(&(source_path.as_str(), scanned.selector_path.as_str()))
            else {
                misses.push(format!(
                    "missing catalog row source_path={source_path} selector_path={}",
                    scanned.selector_path
                ));
                continue;
            };
            for rule in matched {
                if rule.leptos_css.is_empty() {
                    misses.push(format!(
                        "empty leptos_css source_path={source_path} selector_path={}",
                        scanned.selector_path
                    ));
                }
                match (&scanned.kind, &rule.gpui) {
                    (ScanKind::Rule, GpuiOutcome::Paint) => {}
                    (ScanKind::Rule, GpuiOutcome::NotAWidget { .. }) => {
                        misses.push(format!(
                            "ScanKind::Rule paired with GpuiOutcome::NotAWidget source_path={source_path} selector_path={}",
                            scanned.selector_path
                        ));
                    }
                    (ScanKind::Pseudo | ScanKind::Keyframes, GpuiOutcome::Paint) => {
                        misses.push(format!(
                            "Pseudo or Keyframes paired with GpuiOutcome::Paint source_path={source_path} selector_path={}",
                            scanned.selector_path
                        ));
                    }
                    (
                        ScanKind::Pseudo | ScanKind::Keyframes,
                        GpuiOutcome::NotAWidget { feature },
                    ) => {
                        if feature.is_empty() {
                            misses.push(format!(
                                "Pseudo or Keyframes NotAWidget feature is empty source_path={source_path} selector_path={}",
                                scanned.selector_path
                            ));
                        }
                    }
                }
            }
        }
    }

    if !misses.is_empty() {
        let shown = misses.len().min(20);
        let preview = misses
            .iter()
            .take(shown)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        panic!(
            "{} catalog miss(es); showing first {shown}:\n{preview}",
            misses.len()
        );
    }
}

fn collect_scss(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| {
        panic!("read_dir {}: {error}", dir.display());
    });
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!("read_dir entry under {}: {error}", dir.display());
        });
        let path = entry.path();
        if path.is_dir() {
            collect_scss(&path, out);
            continue;
        }
        let is_scss = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".scss"));
        if is_scss && path.is_file() {
            out.push(path);
        }
    }
}

fn relative_source(root: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(root).unwrap_or_else(|error| {
        panic!(
            "strip {} from upstream scss root {}: {error}",
            file.display(),
            root.display()
        )
    });
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
