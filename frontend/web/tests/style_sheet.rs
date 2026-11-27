//! Locks shell, link, and network-button CSS to the pre-edit strings.
//!
//! `mod views` and `mod theme` are private and `lib.rs` is outside this
//! change, so these calls use the exported symbols.

use splora_frontend_shared::all_rules;
use splora_web::{ACCENT, BACKGROUND, TEXT};

unsafe extern "Rust" {
    #[link_name = "splora_web_shell_style"]
    safe fn shell_style() -> String;

    #[link_name = "splora_web_link_style"]
    safe fn link_style() -> String;

    #[link_name = "splora_web_network_button_style"]
    safe fn network_button_style(selected: bool) -> String;

    #[link_name = "splora_web_catalog_css"]
    safe fn catalog_css() -> String;

    #[link_name = "splora_web_page_stylesheet"]
    safe fn page_stylesheet() -> String;
}

#[test]
fn shell_style_is_locked() {
    assert_eq!(
        shell_style(),
        "background:#000000;color:#FFFFFF;min-height:100vh;margin:0"
    );
}

#[test]
fn link_style_is_locked() {
    assert_eq!(link_style(), "color:#FFD100");
}

#[test]
fn network_button_style_is_locked() {
    assert_eq!(
        network_button_style(true),
        "background:#FFD100;color:#000000;border:1px solid #FFD100"
    );
    assert_eq!(
        network_button_style(false),
        "background:#000000;color:#FFFFFF;border:1px solid #FFD100"
    );
}

#[test]
fn background_text_and_accent_are_locked() {
    assert_eq!(BACKGROUND, "#000000");
    assert_eq!(TEXT, "#FFFFFF");
    assert_eq!(ACCENT, "#FFD100");
}

#[test]
fn catalog_css_joins_nonempty_leptos_css() {
    let rules = all_rules();
    let css = catalog_css();
    let expected = rules
        .iter()
        .map(|rule| rule.leptos_css)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(css, expected);
    if rules.is_empty() {
        assert!(css.is_empty());
    } else {
        assert!(
            !css.is_empty(),
            "catalog_css is empty while all_rules has {} rules",
            rules.len()
        );
    }
    if rules
        .iter()
        .any(|rule| rule.leptos_css.contains("background:#000000"))
    {
        assert!(css.contains("background:#000000"));
    }
}

#[test]
fn page_includes_catalog_stylesheet() {
    let views = include_str!("../src/views.rs");
    let shell = views.split("fn Shell()").nth(1).expect("Shell component");
    let shell = shell.split("\nfn ").next().expect("Shell body");
    assert!(
        shell.contains("let stylesheet = page_stylesheet();"),
        "Shell must take the catalog stylesheet, not drop it"
    );
    assert!(
        shell.contains("<style>{stylesheet}</style>"),
        "Shell must emit that string as the style element"
    );
    let page = views
        .split("fn page_stylesheet()")
        .nth(1)
        .expect("page_stylesheet");
    let page = page.split("\nfn ").next().expect("page_stylesheet body");
    assert!(
        page.contains("splora_frontend_shared::style_proof::leptos_stylesheet_css()"),
        "the shell style element must be leptos_stylesheet_css"
    );
    assert!(
        !page.contains("catalog_css"),
        "the shell style element must not be catalog_css"
    );
    assert!(
        !page.contains("#[cfg(any())]"),
        "leptos_stylesheet_css must not sit under cfg(any())"
    );
    assert!(
        !views.contains("let _catalog"),
        "the page must not discard catalog_css"
    );
}

/// `PATH` is only `/tmp/splora-web-style-no-sass`. That directory has no
/// `sass` or `sassc` file, so a PATH lookup cannot see the compiler.
struct RestorePath(Option<std::ffi::OsString>);

impl Drop for RestorePath {
    fn drop(&mut self) {
        unsafe {
            match self.0.take() {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }
    }
}

fn restrict_path_to_dir_without_sass() -> RestorePath {
    const DIR: &str = "/tmp/splora-web-style-no-sass";
    std::fs::create_dir_all(DIR).expect("create the no-sass PATH directory");
    for name in ["sass", "sassc"] {
        let candidate = std::path::Path::new(DIR).join(name);
        assert!(
            !candidate.exists(),
            "{} must not exist",
            candidate.display()
        );
    }
    let previous = std::env::var_os("PATH");
    unsafe {
        std::env::set_var("PATH", DIR);
    }
    let guard = RestorePath(previous);
    assert_eq!(std::env::var("PATH").expect("PATH"), DIR);
    for entry in DIR.split(':') {
        for name in ["sass", "sassc"] {
            let candidate = std::path::Path::new(entry).join(name);
            assert!(!candidate.is_file(), "PATH entry {entry} contains {name}");
        }
    }
    for name in ["sass", "sassc"] {
        match std::process::Command::new(name)
            .arg("--version")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
        {
            Err(error) => assert_eq!(
                error.kind(),
                std::io::ErrorKind::NotFound,
                "{name} must not be found on PATH {DIR}"
            ),
            Ok(status) => panic!("{name} ran on PATH {DIR}: {status}"),
        }
    }
    guard
}

#[test]
fn page_stylesheet_builds_without_sass_on_path() {
    let _path = restrict_path_to_dir_without_sass();
    let stylesheet = page_stylesheet();
    let catalog = all_rules()
        .iter()
        .map(|rule| rule.leptos_css)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!catalog.is_empty(), "catalog stylesheet text is empty");
    assert_eq!(stylesheet, catalog);
    assert!(stylesheet.contains("#000000"));
    let shell = format!(
        "{stylesheet}\n{}\n{}\n{}\n{}",
        shell_style(),
        link_style(),
        network_button_style(true),
        network_button_style(false)
    );
    assert!(shell.contains(&catalog));
    assert!(shell.contains("#000000"));
    assert!(shell.contains("#FFFFFF"));
    assert!(shell.contains("#FFD100"));
}
