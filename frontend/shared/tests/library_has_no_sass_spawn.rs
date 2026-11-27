//! `frontend/shared/src` must not start `sass` or `sassc`.
//! The load path `/home/hunter/.local/sass-load` must not appear there.
//! A CSS class, a color, or a comment that only contains the letters sass
//! is not a process.

use std::fs;
use std::path::{Path, PathBuf};

const LOAD_PATH: &str = "/home/hunter/.local/sass-load";

#[test]
fn shared_library_source_does_not_spawn_sass() {
    assert!(
        violations(r#"const CLASS: &str = "btn-sass"; // sass color #000000"#).is_empty(),
        "a class or comment that only spells sass must not fail the scan"
    );
    assert!(
        violations("// Command::new(\"sass\")\n// find_sass_compiler\nfn paint() {}\n").is_empty(),
        "a comment that does not start a process must not fail the scan"
    );
    assert!(
        violations(
            "fn lookup(name: &str) { Command::new(name); }\nfn note() { lookup(\"not-a-compiler\"); }\n"
        )
        .is_empty(),
        "Command::new of some other program must not fail the scan"
    );
    assert!(
        !violations("fn find_sass_compiler() {}\n").is_empty(),
        "find_sass_compiler in library source must fail the scan"
    );
    assert!(
        !violations("fn main() { std::process::Command::new(\"sass\").status().ok(); }\n")
            .is_empty(),
        "Command::new(\"sass\") must fail the scan"
    );
    assert!(
        !violations("fn main() { std::process::Command::new(\"sassc\"); }\n").is_empty(),
        "Command::new(\"sassc\") must fail the scan"
    );
    assert!(
        !violations(
            "fn lookup(name: &str) { Command::new(name); }\nfn note() { lookup(\"sass\"); lookup(\"sassc\"); }\n"
        )
        .is_empty(),
        "Command::new of a sass or sassc argument must fail the scan"
    );
    assert!(
        !violations(&format!("const PATH: &str = \"{LOAD_PATH}\";\n")).is_empty(),
        "the sass load path must fail the scan"
    );
    assert!(
        !violations(&format!("// {LOAD_PATH}\n")).is_empty(),
        "the sass load path in a comment must fail the scan"
    );

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_files(&root, &mut files);
    assert!(
        !files.is_empty(),
        "frontend/shared/src has no files to scan"
    );
    let mut problems = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_else(|error| {
            panic!("read {} as utf-8: {error}", file.display());
        });
        let found = violations(&text);
        if found.is_empty() {
            continue;
        }
        problems.push(format!("{}: {}", file.display(), found.join("; ")));
    }
    assert!(
        problems.is_empty(),
        "frontend/shared/src must not start sass or sassc or contain {LOAD_PATH}:\n{}",
        problems.join("\n")
    );
}

fn violations(raw: &str) -> Vec<&'static str> {
    let mut found = Vec::new();
    if raw.contains(LOAD_PATH) {
        found.push("contains /home/hunter/.local/sass-load");
    }
    let code = strip_rust_comments(raw);
    if has_ident(&code, "find_sass_compiler") {
        found.push("names find_sass_compiler");
    }
    if has_ident(&code, "compile_upstream_scss") {
        found.push("names compile_upstream_scss");
    }
    if spawns_sass_process(&code) {
        found.push("starts a sass or sassc process");
    }
    found
}

fn spawns_sass_process(code: &str) -> bool {
    let has_compiler_bin = file_has_compiler_bin_literal(code);
    for arg in command_new_args(code) {
        if arg_is_compiler_literal(&arg) {
            return true;
        }
        if arg_ident(&arg).is_some() && has_compiler_bin {
            return true;
        }
    }
    false
}

fn command_new_args(code: &str) -> Vec<String> {
    let needle = "Command::new";
    let bytes = code.as_bytes();
    let mut args = Vec::new();
    let mut index = 0;
    while index + needle.len() <= bytes.len() {
        if &bytes[index..index + needle.len()] == needle.as_bytes() {
            let before_ok = index == 0 || !is_ident_byte(bytes[index - 1]);
            if before_ok {
                let mut next = index + needle.len();
                while next < bytes.len() && bytes[next].is_ascii_whitespace() {
                    next += 1;
                }
                if next < bytes.len() && bytes[next] == b'(' {
                    args.push(read_first_arg(&code[next + 1..]));
                }
            }
        }
        index += 1;
    }
    args
}

fn read_first_arg(after_paren: &str) -> String {
    let mut depth = 0i32;
    let mut out = String::new();
    let mut chars = after_paren.chars();
    while let Some(ch) = chars.next() {
        if ch == '(' {
            depth += 1;
            out.push(ch);
            continue;
        }
        if ch == ')' && depth == 0 {
            break;
        }
        if ch == ')' {
            depth -= 1;
            out.push(ch);
            continue;
        }
        if ch == ',' && depth == 0 {
            break;
        }
        if ch == '"' {
            out.push(ch);
            while let Some(inner) = chars.next() {
                out.push(inner);
                if inner == '\\' {
                    if let Some(escaped) = chars.next() {
                        out.push(escaped);
                    }
                    continue;
                }
                if inner == '"' {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out.trim().to_string()
}

fn arg_is_compiler_literal(arg: &str) -> bool {
    rust_string_literal(arg.trim()).is_some_and(|value| compiler_bin_name(&value))
}

fn arg_ident(arg: &str) -> Option<&str> {
    let arg = arg
        .trim()
        .trim_start_matches('&')
        .trim()
        .trim_start_matches("mut ")
        .trim();
    if is_plain_ident(arg) { Some(arg) } else { None }
}

fn is_plain_ident(arg: &str) -> bool {
    let mut chars = arg.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn file_has_compiler_bin_literal(code: &str) -> bool {
    rust_string_literals(code)
        .iter()
        .any(|value| compiler_bin_name(value))
}

fn compiler_bin_name(value: &str) -> bool {
    let file = value.rsplit(['/', '\\']).next().unwrap_or(value);
    file == "sass" || file == "sassc"
}

fn rust_string_literal(token: &str) -> Option<String> {
    let token = token.trim();
    if token.len() >= 2 && token.starts_with('"') && token.ends_with('"') {
        return Some(unescape_rust_string(&token[1..token.len() - 1]));
    }
    None
}

fn rust_string_literals(code: &str) -> Vec<String> {
    let mut values = Vec::new();
    let chars: Vec<char> = code.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '"' {
            index += 1;
            continue;
        }
        index += 1;
        let mut raw = String::new();
        while index < chars.len() {
            let ch = chars[index];
            index += 1;
            if ch == '\\' && index < chars.len() {
                raw.push('\\');
                raw.push(chars[index]);
                index += 1;
                continue;
            }
            if ch == '"' {
                break;
            }
            raw.push(ch);
        }
        values.push(unescape_rust_string(&raw));
    }
    values
}

fn unescape_rust_string(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn has_ident(code: &str, name: &str) -> bool {
    let bytes = code.as_bytes();
    let needle = name.as_bytes();
    let mut index = 0;
    while index + needle.len() <= bytes.len() {
        if &bytes[index..index + needle.len()] == needle {
            let before_ok = index == 0 || !is_ident_byte(bytes[index - 1]);
            let after = index + needle.len();
            let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
            if before_ok && after_ok {
                return true;
            }
        }
        index += 1;
    }
    false
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Comments are not process construction. Strings stay, so a load path or a
/// compiler name inside a string is still visible to the caller that needs it.
fn strip_rust_comments(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '/' && chars.get(index + 1) == Some(&'/') {
            index += 2;
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }
            continue;
        }
        if ch == '/' && chars.get(index + 1) == Some(&'*') {
            index += 2;
            while index + 1 < chars.len() && !(chars[index] == '*' && chars[index + 1] == '/') {
                index += 1;
            }
            if index + 1 < chars.len() {
                index += 2;
            } else {
                index = chars.len();
            }
            out.push(' ');
            continue;
        }
        if ch == '"' {
            out.push(ch);
            index += 1;
            while index < chars.len() {
                let inner = chars[index];
                out.push(inner);
                index += 1;
                if inner == '\\' && index < chars.len() {
                    out.push(chars[index]);
                    index += 1;
                    continue;
                }
                if inner == '"' {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
        index += 1;
    }
    out
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| {
        panic!("read directory {}: {error}", dir.display());
    });
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!("read directory entry under {}: {error}", dir.display());
        });
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if name == "tests" || name == "target" {
            continue;
        }
        if path.is_dir() {
            collect_files(&path, out);
            continue;
        }
        if path.is_file() {
            out.push(path);
        }
    }
}
