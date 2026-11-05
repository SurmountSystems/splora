//! Theme selector option labels.
//!
//! The contrast option text is BlueMatt. The softsimon option text is softsimon.
//! Any other value is an empty string.

/// One theme option label.
///
/// `"contrast"` is `BlueMatt`. `"softsimon"` is `softsimon`.
/// Any other value is an empty string.
pub fn theme_option_label(value: &str) -> String {
    match value {
        "contrast" => "BlueMatt".to_string(),
        "softsimon" => "softsimon".to_string(),
        "nymkappa" => "nymkappa".to_string(),
        _ => String::new(),
    }
}
