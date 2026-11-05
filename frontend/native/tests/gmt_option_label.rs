//! Native paint of the footer timezone option label.
//!
//! Offset `+0` and name `Greenwich Mean Time (GMT)` is the fixed sentence
//! `UTC - Greenwich Mean Time (GMT)`. Offset `+9` and name
//! `Japan Standard Time (JST)` is `UTC+9 - Japan Standard Time (JST)`.
//! `paint_timezone_option_label` is not defined on `splora_native`.
//! This test does not add it. No window.

#[test]
fn paint_timezone_option_label_matches_shared() {
    let offset = "+0";
    let name = "Greenwich Mean Time (GMT)";
    let expected = "UTC - Greenwich Mean Time (GMT)";
    let painted = splora_native::paint_timezone_option_label(offset, name);
    assert_eq!(
        painted,
        splora_frontend_shared::timezone_option_label(offset, name)
    );
    assert_eq!(painted, expected);

    let offset = "+9";
    let name = "Japan Standard Time (JST)";
    let expected = "UTC+9 - Japan Standard Time (JST)";
    let painted = splora_native::paint_timezone_option_label(offset, name);
    assert_eq!(
        painted,
        splora_frontend_shared::timezone_option_label(offset, name)
    );
    assert_eq!(painted, expected);
}
