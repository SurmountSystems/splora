//! Scanner for upstream `.scss` rules.
//!
//! `.foo` + `&.active` is `.foo.active`. `.foo` + `& .bar` is `.foo .bar`.
//! `.foo` + `&-bar` is `.foo-bar`. A selector that starts with `&` replaces
//! that leading `&` only.
//!
//! Nested `@media` and `@supports` sit outside the composed selector.
//! `@media (min-width: 1px)` around `.foo` then `.bar` is
//! `@media (min-width: 1px) { .foo .bar }`. A block is still emitted when
//! its own declaration list is empty, including a parent that only
//! contains nested rules or a mixin include. The query after the selector
//! is a different rule. `@keyframes` is the only at-rule that changes kind.
//! `::` marks a pseudo-element; a single `:` such as `:hover` or `:before`
//! stays a rule.
//!
//! `@include block-cube.faces` expands mixin `faces` from `_block-cube.scss`.
//! `@include keyframes` stays a selector prefix when that mixin is not defined.
//! When mixin `keyframes` is defined, the include expands to the mixin's
//! at-rules. `@for $i from A through B` repeats its body with `$i` replaced.
//! `@extend` adds the extending selector to every rule whose selector contains
//! the extended simple selector, using the compiler's ancestor weave.
//!
//! ```scss
//! .foo {
//!   color: red;
//!   &:hover { background: blue; }
//!   &::before { content: ""; }
//! }
//! @media (min-width: 1px) { .a { margin: 0; } }
//! @keyframes fade {
//!   from { opacity: 0; }
//!   to { opacity: 1; }
//! }
//! ```
//!
//! Selector text collapses whitespace so `&:after,\n  &:before` joins as
//! `.skeleton-loader:after, &:before`. Property values keep internal
//! whitespace and only trim the ends. A declaration may omit the semicolon
//! when `}` closes the block.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::style_scan::ScanKind::{Keyframes, Pseudo, Rule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedRule {
    pub selector_path: String,
    pub properties: Vec<(String, String)>,
    pub kind: ScanKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanKind {
    Rule,
    Pseudo,
    Keyframes,
}

pub fn scan_scss(source: &str) -> Vec<ScannedRule> {
    let mut scanner = Scanner::new(source);
    scanner.scan();
    apply_extends(&mut scanner.rules, &scanner.extends);
    scanner.rules
}

#[derive(Clone, Copy)]
enum Nest {
    Interp { resume: Option<char> },
    Brace,
}

struct Frame {
    /// Composed selector. `@media` and `@supports` are not stored here.
    selector: String,
    /// Outer `@media` and `@supports`, outermost first.
    outside: Vec<String>,
    rule_index: Option<usize>,
}

struct ExtendUse {
    outside: Vec<String>,
    extender: String,
    target: String,
}

struct Scanner {
    chars: Vec<char>,
    index: usize,
    buf: String,
    quote: Option<char>,
    escape: bool,
    paren: i32,
    nests: Vec<Nest>,
    stack: Vec<Frame>,
    rules: Vec<ScannedRule>,
    mixins: HashMap<String, MixinDef>,
    extends: Vec<ExtendUse>,
    include_wrap: bool,
}

impl Scanner {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            buf: String::new(),
            quote: None,
            escape: false,
            paren: 0,
            nests: Vec::new(),
            stack: Vec::new(),
            rules: Vec::new(),
            mixins: HashMap::new(),
            extends: Vec::new(),
            include_wrap: false,
        }
    }

    fn scan(&mut self) {
        while self.index < self.chars.len() {
            let ch = self.chars[self.index];
            if self.escape {
                self.buf.push(ch);
                self.escape = false;
                self.index += 1;
                continue;
            }
            if self.quote.is_some() {
                self.scan_string(ch);
                continue;
            }
            if self.starts_line_comment() {
                self.skip_line_comment();
                continue;
            }
            if self.starts_block_comment() {
                self.skip_block_comment();
                continue;
            }
            if self.starts_interpolation() {
                self.begin_interpolation(None);
                continue;
            }
            if !self.nests.is_empty() {
                self.scan_interpolation(ch);
                continue;
            }
            self.scan_code(ch);
        }
        // An empty declaration list still leaves the selector. A nesting
        // shell is a catalog selector. Children already joined it.
    }

    fn scan_string(&mut self, ch: char) {
        let quote = self.quote.expect("string state");
        if ch == '\\' {
            self.escape = true;
            self.buf.push(ch);
            self.index += 1;
            return;
        }
        if ch == quote {
            self.quote = None;
            self.buf.push(ch);
            self.index += 1;
            return;
        }
        if self.starts_interpolation() {
            self.begin_interpolation(Some(quote));
            return;
        }
        self.buf.push(ch);
        self.index += 1;
    }

    fn scan_interpolation(&mut self, ch: char) {
        if ch == '"' || ch == '\'' {
            self.quote = Some(ch);
            self.buf.push(ch);
            self.index += 1;
            return;
        }
        if ch == '(' {
            self.paren += 1;
        } else if ch == ')' && self.paren > 0 {
            self.paren -= 1;
        } else if ch == '{' {
            self.nests.push(Nest::Brace);
        } else if ch == '}' {
            if let Some(nest) = self.nests.pop() {
                if let Nest::Interp { resume } = nest {
                    self.quote = resume;
                }
            }
        }
        self.buf.push(ch);
        self.index += 1;
    }

    fn scan_code(&mut self, ch: char) {
        if ch == '"' || ch == '\'' {
            self.quote = Some(ch);
            self.buf.push(ch);
            self.index += 1;
            return;
        }
        if ch == '(' {
            self.paren += 1;
            self.buf.push(ch);
            self.index += 1;
            return;
        }
        if ch == ')' {
            if self.paren > 0 {
                self.paren -= 1;
            }
            self.buf.push(ch);
            self.index += 1;
            return;
        }
        if self.paren > 0 {
            self.buf.push(ch);
            self.index += 1;
            return;
        }
        if ch == '{' {
            let header = normalize_selector(&self.buf);
            if self.store_mixin(&header) {
                return;
            }
            if self.expand_for(&header) {
                return;
            }
            if self.expand_keyframes_include(&header) {
                return;
            }
            if let Some(at_header) = self.expand_include_header(&header) {
                self.buf = at_header;
                self.include_wrap = true;
            }
            self.open_rule();
            self.index += 1;
            return;
        }
        if ch == '}' {
            self.flush_declaration();
            self.stack.pop();
            self.index += 1;
            return;
        }
        if ch == ';' {
            self.flush_declaration();
            self.index += 1;
            return;
        }
        self.buf.push(ch);
        self.index += 1;
    }

    fn open_rule(&mut self) {
        let child = normalize_selector(&self.buf);
        self.buf.clear();
        let wrap_include = self.include_wrap;
        self.include_wrap = false;
        let (parent_selector, parent_outside) = self
            .stack
            .last()
            .map(|frame| (frame.selector.clone(), frame.outside.clone()))
            .unwrap_or_else(|| (String::new(), Vec::new()));
        if child.is_empty() {
            self.stack.push(Frame {
                selector: parent_selector,
                outside: parent_outside,
                rule_index: None,
            });
            return;
        }
        let (selector, outside) = if wrap_include || is_outer_conditional(&child) {
            let mut outside = parent_outside;
            outside.push(child);
            (parent_selector, outside)
        } else {
            (join_selector(&parent_selector, &child), parent_outside)
        };
        let selector_path = render_selector_path(&outside, &selector);
        let kind = kind_for(&selector_path);
        self.rules.push(ScannedRule {
            selector_path,
            properties: Vec::new(),
            kind,
        });
        let rule_index = self.rules.len() - 1;
        self.stack.push(Frame {
            selector,
            outside,
            rule_index: Some(rule_index),
        });
    }

    fn flush_declaration(&mut self) {
        let stmt = std::mem::take(&mut self.buf);
        let header = normalize_selector(&stmt);
        if is_block_cube_faces_include(&header) {
            self.expand_block_cube_faces();
            return;
        }
        if let Some(at_header) = self.expand_include_header(&header) {
            self.emit_wrapped_include(&at_header);
            return;
        }
        if let Some(target) = extend_target(&header) {
            if let Some(frame) = self.stack.last() {
                for extender in split_selector_list(&frame.selector) {
                    self.extends.push(ExtendUse {
                        outside: frame.outside.clone(),
                        extender,
                        target: target.clone(),
                    });
                }
            }
            return;
        }
        let Some(rule_index) = self.stack.last().and_then(|frame| frame.rule_index) else {
            return;
        };
        if let Some((name, value)) = split_declaration(&stmt) {
            self.rules[rule_index].properties.push((name, value));
        }
    }

    fn store_mixin(&mut self, header: &str) -> bool {
        let Some((name, args_src)) = directive_name_and_args(header, "@mixin") else {
            return false;
        };
        let body = self.take_brace_body();
        self.buf.clear();
        self.mixins.insert(
            name,
            MixinDef {
                params: mixin_params(&args_src),
                body,
            },
        );
        true
    }

    fn expand_include_header(&self, header: &str) -> Option<String> {
        let (name, args_src) = directive_name_and_args(header, "@include")?;
        let mixin = self.mixins.get(&name)?;
        expand_mixin(mixin, &args_src)
    }

    fn expand_block_cube_faces(&mut self) {
        let Some(body) = block_cube_faces_body() else {
            return;
        };
        let (selector, outside) = self
            .stack
            .last()
            .map(|frame| (frame.selector.clone(), frame.outside.clone()))
            .unwrap_or_else(|| (String::new(), Vec::new()));
        let mut nested = Scanner::new(&body);
        nested.stack.push(Frame {
            selector,
            outside,
            rule_index: None,
        });
        nested.scan();
        self.rules.append(&mut nested.rules);
    }

    fn expand_for(&mut self, header: &str) -> bool {
        let Some((var, start, end, inclusive)) = parse_for_header(header) else {
            return false;
        };
        let body = self.take_brace_body();
        self.buf.clear();
        let values: Vec<i64> = if inclusive {
            (start..=end).collect()
        } else {
            (start..end).collect()
        };
        for value in values {
            let substituted = substitute_sass_var(&body, &var, value);
            self.scan_nested(&substituted);
        }
        true
    }

    fn expand_keyframes_include(&mut self, header: &str) -> bool {
        let Some((name, args_src)) = directive_name_and_args(header, "@include") else {
            return false;
        };
        if name != "keyframes" {
            return false;
        }
        let Some(mixin) = self.mixins.get(&name) else {
            return false;
        };
        if mixin.params.len() != 1 {
            return false;
        }
        let arg = args_src.trim();
        if arg.is_empty() || arg.contains(',') {
            return false;
        }
        let param = mixin.params[0].clone();
        let mixin_body = mixin.body.clone();
        let content = self.take_brace_body();
        self.buf.clear();
        let expanded = substitute_keyframes_mixin(&mixin_body, &param, arg, &content);
        self.scan_nested(&expanded);
        true
    }

    fn scan_nested(&mut self, source: &str) {
        let (selector, outside) = self
            .stack
            .last()
            .map(|frame| (frame.selector.clone(), frame.outside.clone()))
            .unwrap_or_else(|| (String::new(), Vec::new()));
        let mut nested = Scanner::new(source);
        nested.mixins.clone_from(&self.mixins);
        if !selector.is_empty() || !outside.is_empty() {
            nested.stack.push(Frame {
                selector,
                outside,
                rule_index: None,
            });
        }
        nested.scan();
        self.rules.append(&mut nested.rules);
        self.extends.append(&mut nested.extends);
    }

    fn emit_wrapped_include(&mut self, at_header: &str) {
        let (selector, mut outside) = self
            .stack
            .last()
            .map(|frame| (frame.selector.clone(), frame.outside.clone()))
            .unwrap_or_else(|| (String::new(), Vec::new()));
        outside.push(at_header.to_string());
        let selector_path = render_selector_path(&outside, &selector);
        let kind = kind_for(&selector_path);
        self.rules.push(ScannedRule {
            selector_path,
            properties: Vec::new(),
            kind,
        });
    }

    fn take_brace_body(&mut self) -> String {
        if self.chars.get(self.index) != Some(&'{') {
            return String::new();
        }
        self.index += 1;
        let start = self.index;
        let mut depth = 1i32;
        let mut quote: Option<char> = None;
        let mut escape = false;
        while self.index < self.chars.len() && depth > 0 {
            let ch = self.chars[self.index];
            if escape {
                escape = false;
                self.index += 1;
                continue;
            }
            if let Some(opened) = quote {
                if ch == '\\' {
                    escape = true;
                } else if ch == opened {
                    quote = None;
                }
                self.index += 1;
                continue;
            }
            if ch == '"' || ch == '\'' {
                quote = Some(ch);
                self.index += 1;
                continue;
            }
            if ch == '/' && self.chars.get(self.index + 1) == Some(&'/') {
                self.index += 2;
                while self.index < self.chars.len() && self.chars[self.index] != '\n' {
                    self.index += 1;
                }
                continue;
            }
            if ch == '/' && self.chars.get(self.index + 1) == Some(&'*') {
                self.index += 2;
                while self.index + 1 < self.chars.len()
                    && !(self.chars[self.index] == '*' && self.chars[self.index + 1] == '/')
                {
                    self.index += 1;
                }
                if self.index + 1 < self.chars.len() {
                    self.index += 2;
                } else {
                    self.index = self.chars.len();
                }
                continue;
            }
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    let body: String = self.chars[start..self.index].iter().collect();
                    self.index += 1;
                    return body;
                }
            }
            self.index += 1;
        }
        self.chars[start..self.index].iter().collect()
    }

    fn starts_line_comment(&self) -> bool {
        self.chars.get(self.index) == Some(&'/') && self.chars.get(self.index + 1) == Some(&'/')
    }

    fn starts_block_comment(&self) -> bool {
        self.chars.get(self.index) == Some(&'/') && self.chars.get(self.index + 1) == Some(&'*')
    }

    fn starts_interpolation(&self) -> bool {
        self.chars.get(self.index) == Some(&'#') && self.chars.get(self.index + 1) == Some(&'{')
    }

    fn begin_interpolation(&mut self, resume: Option<char>) {
        self.nests.push(Nest::Interp { resume });
        self.quote = None;
        self.buf.push('#');
        self.buf.push('{');
        self.index += 2;
    }

    fn skip_line_comment(&mut self) {
        self.buf.push(' ');
        self.index += 2;
        while self.index < self.chars.len() && self.chars[self.index] != '\n' {
            self.index += 1;
        }
    }

    fn skip_block_comment(&mut self) {
        self.buf.push(' ');
        self.index += 2;
        while self.index + 1 < self.chars.len()
            && !(self.chars[self.index] == '*' && self.chars[self.index + 1] == '/')
        {
            self.index += 1;
        }
        if self.index + 1 < self.chars.len() {
            self.index += 2;
        } else {
            self.index = self.chars.len();
        }
    }
}

fn is_block_cube_faces_include(header: &str) -> bool {
    header == "@include block-cube.faces" || header == "@include block-cube.faces()"
}

/// Mixin `faces` in the address-blocks partial. A missing file skips the
/// include. Declarations are not invented.
fn block_cube_faces_body() -> Option<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ref/mempool/frontend/src/app/components/address-blocks/_block-cube.scss");
    let text = fs::read_to_string(path).ok()?;
    mixin_body(&text, "faces")
}

fn mixin_body(source: &str, name: &str) -> Option<String> {
    let needle = format!("@mixin {name}");
    let start = source.find(&needle)?;
    let after = start + needle.len();
    let tail = &source[after..];
    match tail.chars().next() {
        Some(ch) if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' => return None,
        _ => {}
    }
    let trimmed = tail.trim_start();
    let head = if let Some(rest) = trimmed.strip_prefix('(') {
        let close = rest.find(')')?;
        if !rest[..close].trim().is_empty() {
            return None;
        }
        rest[close + 1..].trim_start()
    } else {
        trimmed
    };
    if !head.starts_with('{') {
        return None;
    }
    let brace_at = source.len() - head.len();
    let mut scanner = Scanner::new(&source[brace_at..]);
    Some(scanner.take_brace_body())
}

fn kind_for(selector: &str) -> ScanKind {
    if selector.contains("@keyframes") {
        Keyframes
    } else if selector.contains("::") {
        Pseudo
    } else {
        Rule
    }
}

fn join_selector(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        return child.to_string();
    }
    // `@keyframes name` keeps a percentage list as one step: `0%, 15%`.
    // Vendor keyframes are outer at-rules and never reach this join.
    if is_standard_keyframes(parent) {
        return format!("{parent} {child}");
    }
    // Top-level commas only. `(` and `[` raise depth; quotes hide both.
    let split_top = |selector: &str| -> (Vec<String>, bool) {
        let mut spans: Vec<(usize, usize)> = Vec::new();
        let mut start = 0;
        let mut paren = 0i32;
        let mut bracket = 0i32;
        let mut quote: Option<char> = None;
        let mut split = false;
        for (i, c) in selector.char_indices() {
            if let Some(q) = quote {
                if c == q {
                    quote = None;
                }
                continue;
            }
            match c {
                '"' | '\'' => quote = Some(c),
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
                    spans.push((start, i));
                    start = i + c.len_utf8();
                }
                _ => {}
            }
        }
        spans.push((start, selector.len()));
        let mut items = Vec::new();
        for (start, end) in spans {
            let item = selector[start..end].trim();
            if item.is_empty() {
                continue;
            }
            items.push(item.to_string());
        }
        (items, split)
    };
    let (parent_items, parent_split) = split_top(parent);
    // No top-level comma: keep the parent string, so a one-item parent
    // still joins exactly as before. A comma list uses each trimmed item.
    let parents: Vec<String> = if parent_split && !parent_items.is_empty() {
        parent_items
    } else {
        vec![parent.to_string()]
    };
    let (children, _) = split_top(child);
    let mut out = String::new();
    for parent_item in &parents {
        for child_item in &children {
            if !out.is_empty() {
                out.push_str(", ");
            }
            if child_item.contains('&') {
                out.push_str(&child_item.replace('&', parent_item));
            } else {
                out.push_str(parent_item);
                out.push(' ');
                out.push_str(child_item);
            }
        }
    }
    out
}

fn is_outer_conditional(selector: &str) -> bool {
    starts_at_rule(selector, "@media")
        || starts_at_rule(selector, "@supports")
        || starts_at_rule(selector, "@container")
        || starts_at_rule(selector, "@-webkit-keyframes")
        || starts_at_rule(selector, "@-moz-keyframes")
        || starts_at_rule(selector, "@-o-keyframes")
        || starts_at_rule(selector, "@-ms-keyframes")
}

fn is_standard_keyframes(selector: &str) -> bool {
    let Some(rest) = selector.strip_prefix("@keyframes") else {
        return false;
    };
    matches!(rest.chars().next(), Some(ch) if ch.is_whitespace())
}

fn starts_at_rule(selector: &str, name: &str) -> bool {
    let Some(rest) = selector.strip_prefix(name) else {
        return false;
    };
    match rest.chars().next() {
        None => true,
        Some(ch) => ch.is_whitespace() || ch == '(',
    }
}

#[derive(Clone)]
struct MixinDef {
    params: Vec<String>,
    body: String,
}

fn directive_name_and_args(header: &str, keyword: &str) -> Option<(String, String)> {
    let rest = header.trim().strip_prefix(keyword)?;
    let rest = match rest.chars().next() {
        Some(ch) if ch.is_whitespace() => rest.trim(),
        _ => return None,
    };
    if rest.is_empty() {
        return None;
    }
    if let Some(open) = rest.find('(') {
        let name = rest[..open].trim();
        if !is_ident(name) {
            return None;
        }
        let mut depth = 0i32;
        let mut close = None;
        for (offset, ch) in rest[open..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(open + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let close = close?;
        let args_src = rest[open + 1..close].trim().to_string();
        Some((name.to_string(), args_src))
    } else {
        let name = rest.split_whitespace().next().unwrap_or("");
        if !is_ident(name) {
            return None;
        }
        Some((name.to_string(), String::new()))
    }
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn mixin_params(args_src: &str) -> Vec<String> {
    if args_src.trim().is_empty() {
        return Vec::new();
    }
    args_src
        .split(',')
        .filter_map(|part| {
            let part = part.trim().trim_start_matches('$');
            let name = part.split([':', ' ']).next().unwrap_or("").trim();
            if is_ident(name) {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn expand_mixin(mixin: &MixinDef, _args_src: &str) -> Option<String> {
    if !mixin.params.is_empty() {
        return None;
    }
    let header = at_rule_around_content(&mixin.body)?;
    if header.contains('$') || header.contains("#{") {
        return None;
    }
    Some(header)
}

fn is_content_placeholder(body: &str) -> bool {
    let compact: String = body.chars().filter(|ch| !ch.is_whitespace()).collect();
    compact == "@content" || compact == "@content;"
}

fn at_rule_around_content(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if !trimmed.starts_with('@')
        || trimmed.starts_with("@mixin")
        || trimmed.starts_with("@include")
        || trimmed.starts_with("@content")
    {
        return None;
    }
    let chars: Vec<char> = trimmed.chars().collect();
    let brace = chars.iter().position(|ch| *ch == '{')?;
    let header: String = chars[..brace].iter().collect();
    let header = normalize_selector(header.trim());
    if !header.starts_with('@') {
        return None;
    }
    let mut depth = 0i32;
    let mut end = None;
    for (index, ch) in chars.iter().enumerate().skip(brace) {
        if *ch == '{' {
            depth += 1;
        } else if *ch == '}' {
            depth -= 1;
            if depth == 0 {
                end = Some(index);
                break;
            }
        }
    }
    let end = end?;
    let inner: String = chars[brace + 1..end].iter().collect();
    let rest: String = chars[end + 1..].iter().collect();
    if !rest.trim().is_empty() || !is_content_placeholder(&inner) {
        return None;
    }
    Some(header)
}

/// `@media` and `@supports` wrap the composed selector. They are not appended
/// after it, and they are not the whole path when a selector was composed.
fn render_selector_path(outside: &[String], selector: &str) -> String {
    if outside.is_empty() {
        return selector.to_string();
    }
    let mut body = selector.to_string();
    for at_rule in outside.iter().rev() {
        if body.is_empty() {
            body = at_rule.clone();
        } else {
            body = format!("{at_rule} {{ {body} }}");
        }
    }
    body
}

fn normalize_selector(text: &str) -> String {
    let mut out = String::new();
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !out.is_empty() {
                pending_space = true;
            }
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        out.push(ch);
    }
    out
}

fn split_declaration(stmt: &str) -> Option<(String, String)> {
    let chars: Vec<(usize, char)> = stmt.char_indices().collect();
    let mut index = 0;
    let mut quote: Option<char> = None;
    let mut escape = false;
    let mut paren = 0i32;
    let mut nests: Vec<Option<char>> = Vec::new();
    while index < chars.len() {
        let (byte, ch) = chars[index];
        if escape {
            escape = false;
            index += 1;
            continue;
        }
        if let Some(opened) = quote {
            if ch == '\\' {
                escape = true;
            } else if ch == opened {
                quote = None;
            } else if ch == '#' && chars.get(index + 1).is_some_and(|(_, next)| *next == '{') {
                nests.push(Some(opened));
                quote = None;
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            index += 1;
            continue;
        }
        if ch == '#' && chars.get(index + 1).is_some_and(|(_, next)| *next == '{') {
            nests.push(None);
            index += 2;
            continue;
        }
        if !nests.is_empty() {
            if ch == '{' {
                nests.push(None);
            } else if ch == '}' {
                if let Some(resume) = nests.pop() {
                    quote = resume;
                }
            } else if ch == '(' {
                paren += 1;
            } else if ch == ')' && paren > 0 {
                paren -= 1;
            }
            index += 1;
            continue;
        }
        if ch == '(' {
            paren += 1;
            index += 1;
            continue;
        }
        if ch == ')' {
            if paren > 0 {
                paren -= 1;
            }
            index += 1;
            continue;
        }
        if paren == 0 && ch == ':' {
            let name = stmt[..byte].trim().to_string();
            let value = stmt[byte + ch.len_utf8()..].trim().to_string();
            if name.is_empty() {
                return None;
            }
            return Some((name, value));
        }
        index += 1;
    }
    None
}

fn parse_for_header(header: &str) -> Option<(String, i64, i64, bool)> {
    let rest = header.strip_prefix("@for ")?.trim();
    let rest = rest.strip_prefix('$')?;
    let (var, rest) = rest.split_once(' ')?;
    if !is_ident(var) {
        return None;
    }
    let rest = rest.trim().strip_prefix("from ")?.trim();
    let mut parts = rest.split_whitespace();
    let start: i64 = parts.next()?.parse().ok()?;
    let kind = parts.next()?;
    let end: i64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let inclusive = match kind {
        "through" => true,
        "to" => false,
        _ => return None,
    };
    Some((var.to_string(), start, end, inclusive))
}

fn substitute_sass_var(body: &str, var: &str, value: i64) -> String {
    let number = value.to_string();
    let interpolated = format!("#{{${var}}}");
    let body = body.replace(&interpolated, &number);
    replace_sass_ident(&body, &format!("${var}"), &number)
}

fn substitute_keyframes_mixin(body: &str, param: &str, arg: &str, content: &str) -> String {
    let interpolated = format!("#{{${param}}}");
    let body = body.replace(&interpolated, arg);
    let body = replace_sass_ident(&body, &format!("${param}"), arg);
    let body = body.replace("@content;", content);
    body.replace("@content", content)
}

fn replace_sass_ident(body: &str, needle: &str, replacement: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = body.chars().collect();
    let needle: Vec<char> = needle.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index..].starts_with(&needle) {
            let after = index + needle.len();
            let continues = chars
                .get(after)
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-');
            if !continues {
                out.push_str(replacement);
                index = after;
                continue;
            }
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

fn extend_target(header: &str) -> Option<String> {
    let rest = header.strip_prefix("@extend ")?.trim();
    let target = rest.strip_suffix("!optional").unwrap_or(rest).trim();
    if target.is_empty() || target.contains(' ') || target.contains('>') || target.contains(',') {
        return None;
    }
    if target.contains('.') && !target.starts_with('.') && !target.starts_with('#') {
        // `.message.info` is a compound selector. Dart Sass rejects it.
        return None;
    }
    Some(target.to_string())
}

fn split_selector_list(selector: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut start = 0;
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut quote: Option<char> = None;
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
            ')' if paren > 0 => paren -= 1,
            '[' => bracket += 1,
            ']' if bracket > 0 => bracket -= 1,
            ',' if paren == 0 && bracket == 0 => {
                let item = selector[start..index].trim();
                if !item.is_empty() {
                    items.push(item.to_string());
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    let item = selector[start..].trim();
    if !item.is_empty() {
        items.push(item.to_string());
    }
    if items.is_empty() {
        items.push(selector.to_string());
    }
    items
}

#[derive(Clone)]
struct SelectorPart {
    combinator: &'static str,
    simples: Vec<String>,
}

struct MarkedSelector {
    text: String,
    applied: Vec<usize>,
}

fn apply_extends(rules: &mut [ScannedRule], extends: &[ExtendUse]) {
    if extends.is_empty() {
        return;
    }
    for rule in rules {
        let (outside, inner) = peel_outside(&rule.selector_path);
        let original = split_selector_list(&inner);
        let mut marked: Vec<MarkedSelector> = original
            .iter()
            .map(|text| MarkedSelector {
                text: text.clone(),
                applied: Vec::new(),
            })
            .collect();
        let mut index = 0;
        while index < marked.len() {
            for (extend_index, extend) in extends.iter().enumerate() {
                if !extend.outside.is_empty() && extend.outside != outside {
                    continue;
                }
                if marked[index].applied.contains(&extend_index) {
                    continue;
                }
                let extras = unify_extend(&marked[index].text, &extend.extender, &extend.target);
                marked[index].applied.push(extend_index);
                let applied = marked[index].applied.clone();
                for extra in extras {
                    if marked.iter().any(|item| item.text == extra) {
                        continue;
                    }
                    marked.push(MarkedSelector {
                        text: extra,
                        applied: applied.clone(),
                    });
                }
            }
            index += 1;
        }
        if marked.len() == original.len()
            && marked
                .iter()
                .zip(&original)
                .all(|(item, text)| item.text == *text)
        {
            continue;
        }
        let rewritten = marked
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        rule.selector_path = render_selector_path(&outside, &rewritten);
    }
}

fn peel_outside(path: &str) -> (Vec<String>, String) {
    let mut outside = Vec::new();
    let mut rest = path.trim().to_string();
    while let Some((prelude, inner)) = peel_one_outside(&rest) {
        outside.push(prelude);
        rest = inner;
    }
    (outside, rest)
}

fn peel_one_outside(path: &str) -> Option<(String, String)> {
    let path = path.trim();
    if !path.starts_with('@') {
        return None;
    }
    let bytes = path.as_bytes();
    let mut paren = 0i32;
    let mut brace = None;
    for (index, byte) in bytes.iter().enumerate() {
        match byte {
            b'(' => paren += 1,
            b')' if paren > 0 => paren -= 1,
            b'{' if paren == 0 => {
                brace = Some(index);
                break;
            }
            _ => {}
        }
    }
    let brace = brace?;
    if !path.ends_with('}') {
        return None;
    }
    let prelude = path[..brace].trim().to_string();
    let inner = path[brace + 1..path.len() - 1].trim().to_string();
    if prelude.is_empty() {
        return None;
    }
    Some((prelude, inner))
}

fn unify_extend(selector: &str, extender: &str, target: &str) -> Vec<String> {
    let original = parse_complex(selector);
    let extender_parts = parse_complex(extender);
    if original.is_empty() || extender_parts.is_empty() {
        return Vec::new();
    }
    let subject = extender_parts.last().unwrap();
    let subject_is_target = subject.simples.len() == 1 && subject.simples[0] == target;
    if subject_is_target && specificity(&original) >= specificity(&extender_parts) {
        return Vec::new();
    }
    let context: Vec<Vec<String>> = extender_parts[..extender_parts.len() - 1]
        .iter()
        .map(|part| part.simples.clone())
        .collect();
    let mut extras = Vec::new();
    for match_at in 0..original.len() {
        if !original[match_at]
            .simples
            .iter()
            .any(|simple| simple == target)
        {
            continue;
        }
        let prefix: Vec<Vec<String>> = original[..match_at]
            .iter()
            .map(|part| part.simples.clone())
            .collect();
        let replaced = replace_simple(&original[match_at].simples, target, &subject.simples);
        for woven in weave_descendants(&prefix, &context) {
            let mut parts = Vec::new();
            for (index, simples) in woven.into_iter().enumerate() {
                parts.push(SelectorPart {
                    combinator: if index == 0 { "" } else { " " },
                    simples,
                });
            }
            parts.push(SelectorPart {
                combinator: if parts.is_empty() {
                    ""
                } else {
                    original[match_at].combinator
                },
                simples: replaced.clone(),
            });
            for suffix in &original[match_at + 1..] {
                parts.push(suffix.clone());
            }
            let text = format_complex(&parts);
            if text != selector {
                extras.push(text);
            }
        }
    }
    extras
}

fn replace_simple(simples: &[String], target: &str, replacement: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for simple in simples {
        if simple == target {
            out.extend(replacement.iter().cloned());
        } else {
            out.push(simple.clone());
        }
    }
    out
}

fn weave_descendants(left: &[Vec<String>], right: &[Vec<String>]) -> Vec<Vec<Vec<String>>> {
    if left.is_empty() {
        return vec![right.to_vec()];
    }
    if right.is_empty() {
        return vec![left.to_vec()];
    }
    if left[0] == right[0] {
        let mut rest = weave_descendants(&left[1..], &right[1..]);
        for sequence in &mut rest {
            sequence.insert(0, left[0].clone());
        }
        return rest;
    }
    if simples_superset(&left[0], &right[0]) {
        return weave_descendants(left, &right[1..]);
    }
    if simples_superset(&right[0], &left[0]) {
        return weave_descendants(&left[1..], right);
    }
    if is_selector_suffix(left, right) {
        return vec![right.to_vec()];
    }
    if is_selector_suffix(right, left) {
        return vec![left.to_vec()];
    }
    let mut left_then_right = left.to_vec();
    left_then_right.extend_from_slice(right);
    let mut right_then_left = right.to_vec();
    right_then_left.extend_from_slice(left);
    vec![left_then_right, right_then_left]
}

fn simples_superset(outer: &[String], inner: &[String]) -> bool {
    inner.iter().all(|simple| outer.contains(simple))
}

fn is_selector_suffix(short: &[Vec<String>], long: &[Vec<String>]) -> bool {
    short.len() <= long.len() && long[long.len() - short.len()..] == *short
}

fn specificity(parts: &[SelectorPart]) -> (u32, u32, u32) {
    let mut ids = 0u32;
    let mut classes = 0u32;
    let mut elements = 0u32;
    for part in parts {
        for simple in &part.simples {
            if simple.starts_with('#') {
                ids += 1;
            } else if simple.starts_with("::") {
                elements += 1;
            } else if simple.starts_with('.') || simple.starts_with(':') || simple.starts_with('[')
            {
                classes += 1;
            } else {
                elements += 1;
            }
        }
    }
    (ids, classes, elements)
}

fn format_complex(parts: &[SelectorPart]) -> String {
    let mut out = String::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            match part.combinator {
                ">" | "+" | "~" => {
                    out.push(' ');
                    out.push_str(part.combinator);
                    out.push(' ');
                }
                _ => out.push(' '),
            }
        }
        for simple in &part.simples {
            out.push_str(simple);
        }
    }
    out
}

fn parse_complex(selector: &str) -> Vec<SelectorPart> {
    let mut parts = Vec::new();
    let mut simples = Vec::new();
    let mut combinator = "";
    let mut pending = "";
    let chars: Vec<char> = selector.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            if !simples.is_empty() {
                pending = " ";
            }
            index += 1;
            continue;
        }
        if matches!(ch, '>' | '+' | '~') && paren_depth_at(&chars, index) == 0 {
            if !simples.is_empty() {
                parts.push(SelectorPart {
                    combinator,
                    simples: std::mem::take(&mut simples),
                });
                combinator = match ch {
                    '>' => ">",
                    '+' => "+",
                    _ => "~",
                };
                pending = "";
            }
            index += 1;
            continue;
        }
        if pending == " " && !simples.is_empty() {
            parts.push(SelectorPart {
                combinator,
                simples: std::mem::take(&mut simples),
            });
            combinator = " ";
            pending = "";
        }
        let (simple, next) = read_simple(&chars, index);
        if simple.is_empty() {
            index += 1;
            continue;
        }
        simples.push(simple);
        index = next;
    }
    if !simples.is_empty() {
        parts.push(SelectorPart {
            combinator,
            simples,
        });
    }
    parts
}

fn paren_depth_at(chars: &[char], end: usize) -> i32 {
    let mut depth = 0i32;
    let mut index = 0;
    while index < end {
        match chars[index] {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            _ => {}
        }
        index += 1;
    }
    depth
}

fn read_simple(chars: &[char], start: usize) -> (String, usize) {
    if start >= chars.len() {
        return (String::new(), start);
    }
    let mut index = start;
    match chars[index] {
        '.' | '#' => {
            index += 1;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric()
                    || chars[index] == '_'
                    || chars[index] == '-')
            {
                index += 1;
            }
        }
        ':' => {
            index += 1;
            if chars.get(index) == Some(&':') {
                index += 1;
            }
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric()
                    || chars[index] == '_'
                    || chars[index] == '-')
            {
                index += 1;
            }
            if chars.get(index) == Some(&'(') {
                index = skip_balanced(chars, index);
            }
        }
        '[' => index = skip_balanced(chars, index),
        ch if ch.is_ascii_alphabetic() || ch == '_' || ch == '*' => {
            index += 1;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric()
                    || chars[index] == '_'
                    || chars[index] == '-')
            {
                index += 1;
            }
        }
        _ => return (String::new(), start + 1),
    }
    (chars[start..index].iter().collect(), index)
}

fn skip_balanced(chars: &[char], start: usize) -> usize {
    let opening = chars[start];
    let closing = if opening == '(' { ')' } else { ']' };
    let mut depth = 0i32;
    let mut index = start;
    while index < chars.len() {
        match chars[index] {
            ch if ch == opening => depth += 1,
            ch if ch == closing => {
                depth -= 1;
                index += 1;
                if depth == 0 {
                    break;
                }
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    index
}
