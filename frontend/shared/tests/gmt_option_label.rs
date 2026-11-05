//! Timezone option label for the footer selector.
//!
//! Offset `+0` and name `Greenwich Mean Time (GMT)` is the fixed sentence
//! `UTC - Greenwich Mean Time (GMT)`. Offset `+9` and name
//! `Japan Standard Time (JST)` is `UTC+9 - Japan Standard Time (JST)`.
//! The function is not defined in this file.

#[test]
fn gmt_option_label_is_utc_greenwich_mean_time() {
    assert_eq!(
        splora_frontend_shared::timezone_option_label("+0", "Greenwich Mean Time (GMT)"),
        "UTC - Greenwich Mean Time (GMT)"
    );
    assert_eq!(
        splora_frontend_shared::timezone_option_label("+9", "Japan Standard Time (JST)"),
        "UTC+9 - Japan Standard Time (JST)"
    );
}
