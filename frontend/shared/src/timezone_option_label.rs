//! Footer timezone option labels.
//!
//! Offset `+0` omits the numeric offset. Every other offset is written
//! after `UTC`. The name always follows ` - `.

/// One timezone option label.
///
/// `+0` becomes `UTC - ` plus `name`. Any other offset becomes `UTC`
/// plus that offset, then ` - `, then `name`.
pub fn timezone_option_label(offset: &str, name: &str) -> String {
    let shown = if offset == "+0" { "" } else { offset };
    format!("UTC{shown} - {name}")
}
