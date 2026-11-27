//! One catalogued Sass rule and whether GPUI can paint it.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleRule {
    pub source_path: &'static str,
    pub selector_path: &'static str,
    pub properties: &'static [(&'static str, &'static str)],
    pub leptos_css: &'static str,
    pub gpui: GpuiOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuiOutcome {
    Paint,
    NotAWidget { feature: &'static str },
}
