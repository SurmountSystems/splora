//! Surmount DOGE colors. One accent, no second accent.

/// Background `#000000`.
pub const BACKGROUND: &str = "#000000";

/// Body text `#FFFFFF`.
pub const TEXT: &str = "#FFFFFF";

/// The one accent `#FFD100`.
pub const ACCENT: &str = "#FFD100";

/// Background as `0xRRGGBB`.
pub const BACKGROUND_RGB: u32 = 0x000000;

/// Body text as `0xRRGGBB`.
pub const TEXT_RGB: u32 = 0x00FF_FFFF;

/// Accent as `0xRRGGBB`.
pub const ACCENT_RGB: u32 = 0x00FF_D100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doge_colors_are_black_white_and_one_gold() {
        assert_eq!(BACKGROUND, "#000000");
        assert_eq!(TEXT, "#FFFFFF");
        assert_eq!(ACCENT, "#FFD100");
        assert_eq!(BACKGROUND_RGB, 0x000000);
        assert_eq!(TEXT_RGB, 0x00FF_FFFF);
        assert_eq!(ACCENT_RGB, 0x00FF_D100);
    }
}
