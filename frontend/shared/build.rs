//! Generate `style_catalog_mods.rs` from whatever catalog files exist.
//!
//! The directory may be missing or still being filled. A missing directory
//! does not fail the build.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let catalog_dir = manifest_dir.join("src/style_catalog");
    println!("cargo:rerun-if-changed={}", catalog_dir.display());

    let mut files = Vec::new();
    collect_rs(&catalog_dir, &mut files);
    files.sort();
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let dest = out_dir.join("style_catalog_mods.rs");
    let generated = generate(&catalog_dir, &files);
    fs::write(&dest, generated).unwrap_or_else(|error| {
        panic!("write {}: {error}", dest.display());
    });
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
            continue;
        }
        let is_rs = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".rs"));
        if is_rs {
            out.push(path);
        }
    }
}

fn generate(catalog_dir: &Path, files: &[PathBuf]) -> String {
    if files.is_empty() {
        return "pub fn all_rules() -> Vec<&'static crate::style_rule::StyleRule> {\n    Vec::new()\n}\n"
            .to_string();
    }

    let mut used = HashSet::new();
    let mut mods = Vec::new();
    for file in files {
        let relative = file.strip_prefix(catalog_dir).unwrap_or(file.as_path());
        let ident = unique_ident(relative, &mut used);
        mods.push((ident, file.display().to_string()));
    }

    let mut out = String::new();
    for (ident, path) in &mods {
        out.push_str("#[path = ");
        push_rust_string(&mut out, path);
        out.push_str("]\nmod ");
        out.push_str(ident);
        out.push_str(";\n");
    }
    out.push_str("\npub fn all_rules() -> Vec<&'static crate::style_rule::StyleRule> {\n");
    out.push_str("    let mut rules = Vec::new();\n");
    for (ident, _) in &mods {
        out.push_str("    rules.extend(");
        out.push_str(ident);
        out.push_str("::rules().iter());\n");
    }
    out.push_str("    rules\n}\n");
    out
}

fn unique_ident(relative: &Path, used: &mut HashSet<String>) -> String {
    let mut ident = String::from("catalog");
    for component in relative.components() {
        ident.push('_');
        let text = component.as_os_str().to_string_lossy();
        let text = text.strip_suffix(".rs").unwrap_or(&text);
        if text.is_empty() {
            ident.push('_');
            continue;
        }
        for ch in text.chars() {
            if ch.is_ascii_alphanumeric() {
                ident.push(ch.to_ascii_lowercase());
            } else {
                ident.push('_');
            }
        }
    }
    if ident.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        ident.insert(0, '_');
    }
    let base = ident;
    let mut candidate = base.clone();
    let mut suffix = 2u32;
    while !used.insert(candidate.clone()) {
        candidate = format!("{base}_{suffix}");
        suffix += 1;
    }
    candidate
}

fn push_rust_string(out: &mut String, path: &str) {
    out.push('"');
    for ch in path.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out.push('"');
}
