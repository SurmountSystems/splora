//! Footer timezone option labels match the shared formatter.
//!
//! Offset `+0` and name `Greenwich Mean Time (GMT)` is
//! `UTC - Greenwich Mean Time (GMT)`. Offset `+9` and name
//! `Japan Standard Time (JST)` is `UTC+9 - Japan Standard Time (JST)`.

use splora_frontend_shared::timezone_option_label;

#[test]
fn paint_timezone_option_label_matches_shared() {
    let gmt = splora_web::paint_timezone_option_label("+0", "Greenwich Mean Time (GMT)");
    assert_eq!(
        gmt,
        timezone_option_label("+0", "Greenwich Mean Time (GMT)")
    );
    assert_eq!(gmt, "UTC - Greenwich Mean Time (GMT)");

    let jst = splora_web::paint_timezone_option_label("+9", "Japan Standard Time (JST)");
    assert_eq!(
        jst,
        timezone_option_label("+9", "Japan Standard Time (JST)")
    );
    assert_eq!(jst, "UTC+9 - Japan Standard Time (JST)");
}
