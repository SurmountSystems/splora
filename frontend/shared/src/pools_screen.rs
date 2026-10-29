//! Pools screen after the pools control is activated.
//! Non-rendering state only. Not a mining, price, or Lightning response.

/// View after the pools control is activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolsScreen {
    pub pools_screen_active: bool,
}

/// No argument: this call is the pools control already activated, so the screen is active.
/// It is not a success flag the caller can set either way.
pub fn pools_screen_after_pools_control() -> PoolsScreen {
    PoolsScreen {
        pools_screen_active: true,
    }
}
