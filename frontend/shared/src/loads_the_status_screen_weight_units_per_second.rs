//! Static English unit in the status footer.
//!
//! The footer ng-container `shared.weight-per-second|WU/s` is exactly `WU/s`.
//! The number before it is interpolated (`vBytesPerSecond * 4`) and is not
//! this string.

/// Static unit `WU/s` from the status footer.
pub fn loads_the_status_screen_weight_units_per_second() -> &'static str {
    "WU/s"
}
