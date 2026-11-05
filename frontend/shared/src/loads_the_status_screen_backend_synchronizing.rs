//! Static English label in the status footer.
//!
//! The footer ng-container `dashboard.backend-is-synchronizing` is
//! `Backend is synchronizing`. The percent in parentheses is interpolated
//! and is not this string.

/// Static label `Backend is synchronizing` from the status footer.
pub fn loads_the_status_screen_backend_synchronizing() -> &'static str {
    "Backend is synchronizing"
}
