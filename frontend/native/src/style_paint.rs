//! Whether a catalog rule is something GPUI paints, and the `Style` fields it can set.
//! This module does not open a window.
//!
//! `TextStyle` has no `letter-spacing` or `text-transform`. `Fill` is only
//! `Fill::Color`. The paint path's background tags are solid, pattern, checkerboard,
//! and linear gradient, so a radial or conic gradient has no tag. `Style` has no
//! `::before`, `::after`, keyframes, animation, hover, media, or custom-property field.
//! `Position` is only `Relative` or `Absolute`.

use splora_frontend_shared::style_proof::{
    SpecifiedStyle, selector_feature_tokens, token_blocks_resting_paint, write_specified_style,
};
use splora_frontend_shared::style_rule::{GpuiOutcome, StyleRule};

pub use gpui::{CursorStyle, Display, Overflow, Style};

/// True for [`GpuiOutcome::Paint`].
/// True for [`GpuiOutcome::NotAWidget`] only when both feature strings are non-empty.
pub fn gpui_paints(outcome: GpuiOutcome, feature: &'static str) -> bool {
    match outcome {
        GpuiOutcome::Paint => true,
        GpuiOutcome::NotAWidget { feature: on_rule } => {
            on_rule != "pseudo" && feature != "pseudo" && !feature.is_empty() && !on_rule.is_empty()
        }
    }
}

/// True when the catalog is non-empty and every present rule is `Paint`
/// or a `NotAWidget` with a non-empty feature, and every paintable longhand
/// named by that catalog was written onto [`Style`]. An empty catalog is false.
pub fn catalog_rules_paint() -> bool {
    let rules = splora_frontend_shared::all_rules();
    catalog_slice_paints(&rules)
}

/// Same check as [`catalog_rules_paint`] for an explicit slice.
/// An empty slice is false.
pub fn catalog_slice_paints(rules: &[&StyleRule]) -> bool {
    if rules.is_empty() {
        return false;
    }
    rules.iter().all(|rule| rule_paints(rule))
}

/// The first catalog rule whose painted field does not equal the specified
/// longhand. Empty when [`catalog_slice_paints`] is true.
pub fn first_catalog_paint_miss(rules: &[&StyleRule]) -> Option<String> {
    rules.iter().find_map(|rule| rule_paint_miss(rule))
}

/// A paint rule matches when each paintable longhand on [`paint_rule`] equals
/// [`paint_longhand`] for that specified value. The theme sample is not that value.
fn rule_paints(rule: &StyleRule) -> bool {
    rule_paint_miss(rule).is_none()
}

fn rule_paint_miss(rule: &StyleRule) -> Option<String> {
    if resting_paint_blocked(rule.selector_path) {
        let painted = paint_rule(rule);
        if painted.wrote_display || painted.wrote_overflow {
            return Some(format!(
                "selector token painted as resting style source_path={} selector_path={}",
                rule.source_path, rule.selector_path
            ));
        }
        let tokens = selector_feature_tokens(rule.selector_path);
        // `:host` and `:not` do not block. `:hover`, `:focus`, `::before`,
        // scrollbars, `@media`, and `keyframes` do, so position stays default.
        if tokens.iter().copied().any(token_blocks_resting_paint)
            && painted.style.position != Style::default().position
        {
            return Some(format!(
                "selector token painted as resting style source_path={} selector_path={}",
                rule.source_path, rule.selector_path
            ));
        }
        for token in tokens {
            if token == "pseudo" || !painted.features.contains(&token) {
                return Some(format!(
                    "selector token is not a gpui feature source_path={} selector_path={} token={token}",
                    rule.source_path, rule.selector_path
                ));
            }
        }
        return None;
    }
    let feature = match rule.gpui {
        GpuiOutcome::Paint => "",
        GpuiOutcome::NotAWidget { feature } => feature,
    };
    if feature == "pseudo" {
        return Some(format!(
            "gpui feature pseudo is not a selector token source_path={} selector_path={}",
            rule.source_path, rule.selector_path
        ));
    }
    if !gpui_paints(rule.gpui, feature) {
        return Some(format!(
            "gpui outcome is not paintable source_path={} selector_path={}",
            rule.source_path, rule.selector_path
        ));
    }
    let painted = paint_rule(rule);
    if let GpuiOutcome::NotAWidget { feature } = rule.gpui {
        if let Some(miss) = feature_name_miss(rule, feature, &painted.features) {
            return Some(miss);
        }
    }
    for (index, &(name, value)) in rule.properties.iter().enumerate() {
        match paint_longhand(name, value) {
            LonghandPaint::Feature(feature) => {
                if let Some(miss) = feature_name_miss(rule, feature, &painted.features) {
                    return Some(format!("{miss} property={name}"));
                }
            }
            LonghandPaint::Written(_) => {
                if later_property_rewrites(rule, index, name) {
                    continue;
                }
                if !style_matches_longhand(&painted.style, name, value) {
                    return Some(format!(
                        "paint field differs source_path={} selector_path={} property={name} value={value}",
                        rule.source_path, rule.selector_path
                    ));
                }
            }
        }
    }
    None
}

/// A feature string is not a Style field match. An empty name, the catalog
/// word `pseudo`, or a paintable longhand name is a miss. Any other name has
/// to stay on the paint result. A missing name is a silent drop.
fn feature_name_miss(rule: &StyleRule, feature: &str, features: &[&str]) -> Option<String> {
    if feature.is_empty() || feature == "pseudo" || is_paintable_longhand(feature) {
        return Some(format!(
            "feature string is not a Style field match source_path={} selector_path={} feature={feature}",
            rule.source_path, rule.selector_path
        ));
    }
    if !features.contains(&feature) {
        return Some(format!(
            "unhostable longhand was dropped source_path={} selector_path={} feature={feature}",
            rule.source_path, rule.selector_path
        ));
    }
    None
}

/// `:host` and `:not` still select an element. Every other colon token,
/// `@media`, and `keyframes` is not resting paint. The token rule is the
/// shared `token_blocks_resting_paint` function.
fn resting_paint_blocked(selector: &str) -> bool {
    selector_feature_tokens(selector)
        .iter()
        .copied()
        .any(token_blocks_resting_paint)
}

/// A later declaration in the same rule replaced this longhand's field.
/// The resting value is that later declaration, not this one.
fn later_property_rewrites(rule: &StyleRule, index: usize, name: &str) -> bool {
    let LonghandPaint::Written(specified) = paint_longhand(name, rule.properties[index].1) else {
        return false;
    };
    for &(later_name, later_value) in &rule.properties[index + 1..] {
        let LonghandPaint::Written(later) = paint_longhand(later_name, later_value) else {
            continue;
        };
        let mut overwrote = specified.clone();
        let mut wrote_display = false;
        let mut wrote_overflow = false;
        apply_written(
            later_name,
            &mut overwrote,
            &later,
            &mut wrote_display,
            &mut wrote_overflow,
        );
        if !longhand_fields_equal(name, &specified, &overwrote) {
            return true;
        }
    }
    false
}

/// One longhand painted onto a fresh [`Style`], or a named feature that
/// does not write that field.
#[derive(Clone, Debug)]
pub enum LonghandPaint {
    Written(Style),
    Feature(&'static str),
}

/// Per-rule paint. Display, overflow, and cursor use the specified value.
/// The theme sample is not copied onto this style.
#[derive(Clone, Debug)]
pub struct RulePaint {
    pub style: Style,
    pub wrote_display: bool,
    pub wrote_overflow: bool,
    pub features: Vec<&'static str>,
}

/// Paints `name: value` onto a fresh [`Style`]. The mapping lives in the
/// shared writer. `overflow-x: auto` is the feature `overflow-x:auto` and
/// does not assign [`Overflow`].
pub fn paint_longhand(name: &str, value: &str) -> LonghandPaint {
    match write_specified_style(name, value) {
        SpecifiedStyle::Feature(feature) => LonghandPaint::Feature(feature),
        SpecifiedStyle::Written(style) => LonghandPaint::Written(style),
    }
}

/// Paints the display, overflow, and cursor longhands of one catalog rule
/// onto a fresh [`Style`].
pub fn paint_rule(rule: &StyleRule) -> RulePaint {
    let mut style = Style::default();
    let mut wrote_display = false;
    let mut wrote_overflow = false;
    let mut features = Vec::new();
    if resting_paint_blocked(rule.selector_path) {
        for token in selector_feature_tokens(rule.selector_path) {
            if token != "pseudo" && !features.contains(&token) {
                features.push(token);
            }
        }
        return RulePaint {
            style,
            wrote_display,
            wrote_overflow,
            features,
        };
    }
    if let GpuiOutcome::NotAWidget { feature } = rule.gpui {
        if !feature.is_empty() && feature != "pseudo" && !features.contains(&feature) {
            features.push(feature);
        }
    }
    let paint_longhands = match rule.gpui {
        GpuiOutcome::Paint => true,
        GpuiOutcome::NotAWidget { feature } => !feature.is_empty() && feature != "pseudo",
    };
    if paint_longhands {
        for &(name, value) in rule.properties {
            match paint_longhand(name, value) {
                LonghandPaint::Feature(feature) => {
                    if feature != "pseudo" && !features.contains(&feature) {
                        features.push(feature);
                    }
                }
                LonghandPaint::Written(painted) => {
                    apply_written(
                        name,
                        &mut style,
                        &painted,
                        &mut wrote_display,
                        &mut wrote_overflow,
                    );
                }
            }
        }
    }
    RulePaint {
        style,
        wrote_display,
        wrote_overflow,
        features,
    }
}

/// True when `style`'s field equals the specified longhand.
/// A named feature, including `overflow-x:auto`, is not a field match.
/// `cursor: grab` matches only [`CursorStyle::OpenHand`], not the theme
/// sample's [`CursorStyle::PointingHand`].
pub fn style_matches_longhand(style: &Style, name: &str, value: &str) -> bool {
    let painted = match paint_longhand(name, value) {
        LonghandPaint::Feature(_) => return false,
        LonghandPaint::Written(painted) => painted,
    };
    longhand_fields_equal(name, style, &painted)
}

fn apply_written(
    name: &str,
    style: &mut Style,
    painted: &Style,
    wrote_display: &mut bool,
    wrote_overflow: &mut bool,
) {
    let name = name.trim();
    match name {
        "display" => {
            style.display = painted.display;
            *wrote_display = true;
        }
        "visibility" => style.visibility = painted.visibility,
        "overflow" => {
            style.overflow = painted.overflow;
            *wrote_overflow = true;
        }
        "overflow-x" => {
            style.overflow.x = painted.overflow.x;
            *wrote_overflow = true;
        }
        "overflow-y" => {
            style.overflow.y = painted.overflow.y;
            *wrote_overflow = true;
        }
        "position" => style.position = painted.position,
        "opacity" => style.opacity = painted.opacity,
        "cursor" => style.mouse_cursor = painted.mouse_cursor,
        "color" => style.text.color = painted.text.color,
        "font-size" => style.text.font_size = painted.text.font_size,
        "font-weight" => style.text.font_weight = painted.text.font_weight,
        "font-style" => style.text.font_style = painted.text.font_style,
        "line-height" => style.text.line_height = painted.text.line_height,
        "text-align" => style.text.text_align = painted.text.text_align,
        "white-space" => style.text.white_space = painted.text.white_space,
        "flex-direction" => style.flex_direction = painted.flex_direction,
        "flex-wrap" => style.flex_wrap = painted.flex_wrap,
        "flex-grow" => style.flex_grow = painted.flex_grow,
        "flex-shrink" => style.flex_shrink = painted.flex_shrink,
        "flex-basis" => style.flex_basis = painted.flex_basis,
        "aspect-ratio" => style.aspect_ratio = painted.aspect_ratio,
        "background" | "background-color" => style.background = painted.background.clone(),
        "box-shadow" => style.box_shadow = painted.box_shadow.clone(),
        "border" => {
            style.border_widths = painted.border_widths;
            style.border_style = painted.border_style;
            style.border_color = painted.border_color;
        }
        "border-style" => style.border_style = painted.border_style,
        "grid-template-columns" => style.grid_cols = painted.grid_cols,
        "grid-template-rows" => style.grid_rows = painted.grid_rows,
        "width" => style.size.width = painted.size.width,
        "height" => style.size.height = painted.size.height,
        "min-width" => style.min_size.width = painted.min_size.width,
        "min-height" => style.min_size.height = painted.min_size.height,
        "max-width" => style.max_size.width = painted.max_size.width,
        "max-height" => style.max_size.height = painted.max_size.height,
        "row-gap" => style.gap.height = painted.gap.height,
        "column-gap" => style.gap.width = painted.gap.width,
        "gap" => style.gap = painted.gap,
        "margin" => style.margin = painted.margin,
        "margin-top" => style.margin.top = painted.margin.top,
        "margin-right" => style.margin.right = painted.margin.right,
        "margin-bottom" => style.margin.bottom = painted.margin.bottom,
        "margin-left" => style.margin.left = painted.margin.left,
        "padding" => style.padding = painted.padding,
        "padding-top" => style.padding.top = painted.padding.top,
        "padding-right" => style.padding.right = painted.padding.right,
        "padding-bottom" => style.padding.bottom = painted.padding.bottom,
        "padding-left" => style.padding.left = painted.padding.left,
        "inset" | "top" | "right" | "bottom" | "left" => copy_inset(name, style, painted),
        "inset-inline-end" => style.inset.right = painted.inset.right,
        "inset-inline-start" => style.inset.left = painted.inset.left,
        "inset-block-start" => style.inset.top = painted.inset.top,
        "inset-block-end" => style.inset.bottom = painted.inset.bottom,
        "inset-block" => {
            style.inset.top = painted.inset.top;
            style.inset.bottom = painted.inset.bottom;
        }
        "inset-inline" => {
            style.inset.left = painted.inset.left;
            style.inset.right = painted.inset.right;
        }
        "border-width"
        | "border-top-width"
        | "border-right-width"
        | "border-bottom-width"
        | "border-left-width" => copy_border_width(name, style, painted),
        "border-color"
        | "border-top-color"
        | "border-right-color"
        | "border-bottom-color"
        | "border-left-color" => style.border_color = painted.border_color,
        "border-radius"
        | "border-top-left-radius"
        | "border-top-right-radius"
        | "border-bottom-right-radius"
        | "border-bottom-left-radius" => copy_radius(name, style, painted),
        "grid-column" | "grid-row" => copy_grid_axis(name, style, painted),
        _ => {}
    }
}

fn copy_inset(name: &str, style: &mut Style, painted: &Style) {
    match name {
        "top" => style.inset.top = painted.inset.top,
        "right" => style.inset.right = painted.inset.right,
        "bottom" => style.inset.bottom = painted.inset.bottom,
        "left" => style.inset.left = painted.inset.left,
        _ => style.inset = painted.inset,
    }
}

fn copy_border_width(name: &str, style: &mut Style, painted: &Style) {
    match name {
        "border-top-width" => style.border_widths.top = painted.border_widths.top,
        "border-right-width" => style.border_widths.right = painted.border_widths.right,
        "border-bottom-width" => style.border_widths.bottom = painted.border_widths.bottom,
        "border-left-width" => style.border_widths.left = painted.border_widths.left,
        _ => style.border_widths = painted.border_widths,
    }
}

fn copy_radius(name: &str, style: &mut Style, painted: &Style) {
    match name {
        "border-top-left-radius" => style.corner_radii.top_left = painted.corner_radii.top_left,
        "border-top-right-radius" => style.corner_radii.top_right = painted.corner_radii.top_right,
        "border-bottom-right-radius" => {
            style.corner_radii.bottom_right = painted.corner_radii.bottom_right;
        }
        "border-bottom-left-radius" => {
            style.corner_radii.bottom_left = painted.corner_radii.bottom_left;
        }
        _ => style.corner_radii = painted.corner_radii,
    }
}

fn copy_grid_axis(name: &str, style: &mut Style, painted: &Style) {
    let Some(source) = painted.grid_location.clone() else {
        return;
    };
    let mut location = style.grid_location.clone().unwrap_or_default();
    if name == "grid-column" {
        location.column = source.column;
    } else {
        location.row = source.row;
    }
    style.grid_location = Some(location);
}

fn longhand_fields_equal(name: &str, left: &Style, right: &Style) -> bool {
    let name = name.trim();
    match name {
        "display" => left.display == right.display,
        "visibility" => left.visibility == right.visibility,
        "overflow" => left.overflow == right.overflow,
        "overflow-x" => left.overflow.x == right.overflow.x,
        "overflow-y" => left.overflow.y == right.overflow.y,
        "position" => left.position == right.position,
        "opacity" => left.opacity == right.opacity,
        "cursor" => left.mouse_cursor == right.mouse_cursor,
        "color" => left.text.color == right.text.color,
        "font-size" => left.text.font_size == right.text.font_size,
        "font-weight" => left.text.font_weight == right.text.font_weight,
        "font-style" => left.text.font_style == right.text.font_style,
        "line-height" => left.text.line_height == right.text.line_height,
        "text-align" => left.text.text_align == right.text.text_align,
        "white-space" => left.text.white_space == right.text.white_space,
        "flex-direction" => left.flex_direction == right.flex_direction,
        "flex-wrap" => left.flex_wrap == right.flex_wrap,
        "flex-grow" => left.flex_grow == right.flex_grow,
        "flex-shrink" => left.flex_shrink == right.flex_shrink,
        "flex-basis" => left.flex_basis == right.flex_basis,
        "aspect-ratio" => left.aspect_ratio == right.aspect_ratio,
        "background" | "background-color" => left.background == right.background,
        "box-shadow" => left.box_shadow == right.box_shadow,
        "border" => {
            left.border_widths == right.border_widths
                && left.border_style == right.border_style
                && left.border_color == right.border_color
        }
        "border-style" => left.border_style == right.border_style,
        "border-color"
        | "border-top-color"
        | "border-right-color"
        | "border-bottom-color"
        | "border-left-color" => left.border_color == right.border_color,
        "grid-template-columns" => left.grid_cols == right.grid_cols,
        "grid-template-rows" => left.grid_rows == right.grid_rows,
        "grid-column" => match (&left.grid_location, &right.grid_location) {
            (Some(left), Some(right)) => left.column == right.column,
            _ => false,
        },
        "grid-row" => match (&left.grid_location, &right.grid_location) {
            (Some(left), Some(right)) => left.row == right.row,
            _ => false,
        },
        "width" => left.size.width == right.size.width,
        "height" => left.size.height == right.size.height,
        "min-width" => left.min_size.width == right.min_size.width,
        "min-height" => left.min_size.height == right.min_size.height,
        "max-width" => left.max_size.width == right.max_size.width,
        "max-height" => left.max_size.height == right.max_size.height,
        "row-gap" => left.gap.height == right.gap.height,
        "column-gap" => left.gap.width == right.gap.width,
        "gap" => left.gap == right.gap,
        "margin" => left.margin == right.margin,
        "margin-top" => left.margin.top == right.margin.top,
        "margin-right" => left.margin.right == right.margin.right,
        "margin-bottom" => left.margin.bottom == right.margin.bottom,
        "margin-left" => left.margin.left == right.margin.left,
        "padding" => left.padding == right.padding,
        "padding-top" => left.padding.top == right.padding.top,
        "padding-right" => left.padding.right == right.padding.right,
        "padding-bottom" => left.padding.bottom == right.padding.bottom,
        "padding-left" => left.padding.left == right.padding.left,
        "inset" => left.inset == right.inset,
        "top" | "inset-block-start" => left.inset.top == right.inset.top,
        "right" | "inset-inline-end" => left.inset.right == right.inset.right,
        "bottom" | "inset-block-end" => left.inset.bottom == right.inset.bottom,
        "left" | "inset-inline-start" => left.inset.left == right.inset.left,
        "inset-block" => {
            left.inset.top == right.inset.top && left.inset.bottom == right.inset.bottom
        }
        "inset-inline" => {
            left.inset.left == right.inset.left && left.inset.right == right.inset.right
        }
        "border-width" => left.border_widths == right.border_widths,
        "border-top-width" => left.border_widths.top == right.border_widths.top,
        "border-right-width" => left.border_widths.right == right.border_widths.right,
        "border-bottom-width" => left.border_widths.bottom == right.border_widths.bottom,
        "border-left-width" => left.border_widths.left == right.border_widths.left,
        "border-radius" => left.corner_radii == right.corner_radii,
        "border-top-left-radius" => left.corner_radii.top_left == right.corner_radii.top_left,
        "border-top-right-radius" => left.corner_radii.top_right == right.corner_radii.top_right,
        "border-bottom-right-radius" => {
            left.corner_radii.bottom_right == right.corner_radii.bottom_right
        }
        "border-bottom-left-radius" => {
            left.corner_radii.bottom_left == right.corner_radii.bottom_left
        }
        _ => false,
    }
}

/// Theme colors taken from [`crate::theme`], not a second palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordedThemeColors {
    pub background: &'static str,
    pub background_rgb: u32,
    pub text: &'static str,
    pub accent: &'static str,
}

/// Returns the real theme constants for background, text, and accent.
pub fn recorded_theme_colors() -> RecordedThemeColors {
    RecordedThemeColors {
        background: crate::theme::BACKGROUND,
        background_rgb: crate::theme::BACKGROUND_RGB,
        text: crate::theme::TEXT,
        accent: crate::theme::ACCENT,
    }
}

/// Names GPUI `Style` cannot host. These are not paintable longhands.
pub fn unhostable_style_names() -> &'static [&'static str] {
    &[
        "::before",
        "::after",
        "keyframes",
        "animation",
        ":hover",
        "@media",
        "custom-property",
        "radial-gradient",
        "conic-gradient",
        "letter-spacing",
        "text-transform",
        "position:fixed",
        "position:sticky",
    ]
}

const PAINTABLE_LONGHANDS: &[&str] = &[
    "display",
    "visibility",
    "overflow",
    "position",
    "inset",
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "aspect-ratio",
    "margin",
    "padding",
    "border-width",
    "border-color",
    "border-style",
    "border-radius",
    "box-shadow",
    "color",
    "font-weight",
    "font-style",
    "line-height",
    "text-align",
    "white-space",
    "font-size",
    "cursor",
    "opacity",
    "flex-direction",
    "flex-wrap",
    "flex-basis",
    "flex-grow",
    "flex-shrink",
    "gap",
    "grid-template-columns",
    "grid-template-rows",
    "grid-column",
    "grid-row",
    "background-color",
];

/// Paintable longhands in [`PAINTABLE_LONGHANDS`] whose witness value is missing.
pub fn unwritten_paintable_longhands(style: &Style) -> Vec<&'static str> {
    PAINTABLE_LONGHANDS
        .iter()
        .copied()
        .filter(|name| !paintable_longhand_field_was_written(style, name))
        .collect()
}

fn is_paintable_longhand(name: &str) -> bool {
    paintable_arm(name).is_some()
}

/// Writes every paintable longhand onto a [`Style`] using the theme colors.
pub fn paint_splora_style() -> Style {
    let mut style = Style::default();
    write_paintable_longhands(&mut style);
    style
}

fn write_paintable_longhands(style: &mut Style) {
    let background = theme_hsla(crate::theme::BACKGROUND_RGB);
    let text = theme_hsla(crate::theme::TEXT_RGB);
    let accent = theme_hsla(crate::theme::ACCENT_RGB);

    style.display = gpui::Display::Flex;
    style.visibility = gpui::Visibility::Hidden;
    style.overflow = gpui::point(gpui::Overflow::Hidden, gpui::Overflow::Hidden);
    style.position = gpui::Position::Absolute;
    style.inset = edges_len(4.0);
    style.size = gpui::Size {
        width: px_len(32.0),
        height: px_len(32.0),
    };
    style.min_size = gpui::Size {
        width: px_len(1.0),
        height: px_len(1.0),
    };
    style.max_size = gpui::Size {
        width: px_len(64.0),
        height: px_len(64.0),
    };
    style.aspect_ratio = Some(1.5);
    style.margin = edges_len(8.0);
    style.padding = edges_def(8.0);
    style.border_widths = edges_abs(1.0);
    style.border_color = Some(accent);
    style.border_style = gpui::BorderStyle::Dashed;
    style.corner_radii = gpui::Corners {
        top_left: px_abs(2.0),
        top_right: px_abs(2.0),
        bottom_right: px_abs(2.0),
        bottom_left: px_abs(2.0),
    };
    style.box_shadow = vec![gpui::BoxShadow::new(gpui::px(0.0), gpui::px(1.0), accent)];
    style.text.color = Some(text);
    style.text.font_size = Some(px_abs(16.0));
    style.text.font_weight = Some(gpui::FontWeight(400.0));
    style.text.font_style = Some(gpui::FontStyle::Italic);
    style.text.line_height = Some(px_def(20.0));
    style.text.text_align = Some(gpui::TextAlign::Left);
    style.text.white_space = Some(gpui::WhiteSpace::Normal);
    style.mouse_cursor = Some(gpui::CursorStyle::PointingHand);
    style.opacity = Some(1.0);
    style.flex_direction = gpui::FlexDirection::Column;
    style.flex_wrap = gpui::FlexWrap::Wrap;
    style.flex_basis = px_len(8.0);
    style.flex_grow = 1.0;
    style.flex_shrink = 0.0;
    style.gap = gpui::Size {
        width: px_def(8.0),
        height: px_def(8.0),
    };
    style.grid_cols = Some(gpui::GridTemplate {
        repeat: 12,
        min_size: gpui::GridTemplateMinSize::MinContent,
    });
    style.grid_rows = Some(gpui::GridTemplate {
        repeat: 4,
        min_size: gpui::GridTemplateMinSize::MaxContent,
    });
    style.grid_location = Some(gpui::GridLocation::default());
    style.background = Some(background.into());
}

fn paintable_longhand_field_was_written(style: &Style, name: &str) -> bool {
    let Some(arm) = paintable_arm(name) else {
        return false;
    };
    let background = theme_hsla(crate::theme::BACKGROUND_RGB);
    let text = theme_hsla(crate::theme::TEXT_RGB);
    let accent = theme_hsla(crate::theme::ACCENT_RGB);
    match arm {
        "display" => style.display == gpui::Display::Flex,
        "visibility" => style.visibility == gpui::Visibility::Hidden,
        "overflow" => {
            style.overflow.x == gpui::Overflow::Hidden && style.overflow.y == gpui::Overflow::Hidden
        }
        "position" => style.position == gpui::Position::Absolute,
        "inset" => style.inset.top == px_len(4.0),
        "size" => style.size.width == px_len(32.0) && style.size.height == px_len(32.0),
        "min-size" => style.min_size.width == px_len(1.0) && style.min_size.height == px_len(1.0),
        "max-size" => style.max_size.width == px_len(64.0) && style.max_size.height == px_len(64.0),
        "aspect-ratio" => style.aspect_ratio == Some(1.5),
        "margin" => style.margin.top == px_len(8.0),
        "padding" => style.padding.top == px_def(8.0),
        "border-width" => style.border_widths.top == px_abs(1.0),
        "border-color" => style.border_color == Some(accent),
        "border-style" => style.border_style == gpui::BorderStyle::Dashed,
        "border-radius" => style.corner_radii.top_left == px_abs(2.0),
        "box-shadow" => {
            style.box_shadow == vec![gpui::BoxShadow::new(gpui::px(0.0), gpui::px(1.0), accent)]
        }
        "color" => style.text.color == Some(text),
        "font-weight" => style.text.font_weight == Some(gpui::FontWeight(400.0)),
        "font-style" => style.text.font_style == Some(gpui::FontStyle::Italic),
        "line-height" => style.text.line_height == Some(px_def(20.0)),
        "text-align" => style.text.text_align == Some(gpui::TextAlign::Left),
        "white-space" => style.text.white_space == Some(gpui::WhiteSpace::Normal),
        "font-size" => style.text.font_size == Some(px_abs(16.0)),
        "cursor" => style.mouse_cursor == Some(gpui::CursorStyle::PointingHand),
        "opacity" => style.opacity == Some(1.0),
        "flex-direction" => style.flex_direction == gpui::FlexDirection::Column,
        "flex-wrap" => style.flex_wrap == gpui::FlexWrap::Wrap,
        "flex-basis" => style.flex_basis == px_len(8.0),
        "flex-grow" => style.flex_grow == 1.0,
        "flex-shrink" => style.flex_shrink == 0.0,
        "gap" => style.gap.width == px_def(8.0) && style.gap.height == px_def(8.0),
        "grid-template-columns" => style.grid_cols.is_some_and(|cols| {
            cols.repeat == 12 && cols.min_size == gpui::GridTemplateMinSize::MinContent
        }),
        "grid-template-rows" => style.grid_rows.is_some_and(|rows| {
            rows.repeat == 4 && rows.min_size == gpui::GridTemplateMinSize::MaxContent
        }),
        "grid-location" => style.grid_location.is_some(),
        "background" => style.background == Some(background.into()),
        _ => false,
    }
}

fn paintable_arm(name: &str) -> Option<&'static str> {
    match name.trim() {
        "display" => Some("display"),
        "visibility" => Some("visibility"),
        "overflow" | "overflow-x" | "overflow-y" => Some("overflow"),
        "position" => Some("position"),
        "inset" | "top" | "right" | "bottom" | "left" => Some("inset"),
        "width" | "height" | "size" => Some("size"),
        "min-width" | "min-height" | "min-size" => Some("min-size"),
        "max-width" | "max-height" | "max-size" => Some("max-size"),
        "aspect-ratio" => Some("aspect-ratio"),
        "margin" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => {
            Some("margin")
        }
        "padding" | "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => {
            Some("padding")
        }
        "border-width"
        | "border-top-width"
        | "border-right-width"
        | "border-bottom-width"
        | "border-left-width" => Some("border-width"),
        "border-color"
        | "border-top-color"
        | "border-right-color"
        | "border-bottom-color"
        | "border-left-color" => Some("border-color"),
        "border-style" => Some("border-style"),
        "border-radius"
        | "border-top-left-radius"
        | "border-top-right-radius"
        | "border-bottom-right-radius"
        | "border-bottom-left-radius" => Some("border-radius"),
        "box-shadow" => Some("box-shadow"),
        "color" => Some("color"),
        "font-weight" => Some("font-weight"),
        "font-style" => Some("font-style"),
        "line-height" => Some("line-height"),
        "text-align" => Some("text-align"),
        "white-space" => Some("white-space"),
        "font-size" => Some("font-size"),
        "cursor" => Some("cursor"),
        "opacity" => Some("opacity"),
        "flex-direction" => Some("flex-direction"),
        "flex-wrap" => Some("flex-wrap"),
        "flex-basis" => Some("flex-basis"),
        "flex-grow" => Some("flex-grow"),
        "flex-shrink" => Some("flex-shrink"),
        "gap" | "row-gap" | "column-gap" => Some("gap"),
        "grid-template-columns" | "grid-cols" => Some("grid-template-columns"),
        "grid-template-rows" | "grid-rows" => Some("grid-template-rows"),
        "grid-column" | "grid-row" | "grid-area" | "grid-location" => Some("grid-location"),
        "background" | "background-color" | "background-fill" => Some("background"),
        _ => None,
    }
}

fn theme_hsla(rgb: u32) -> gpui::Hsla {
    gpui::rgb(rgb).into()
}

fn px_abs(value: f32) -> gpui::AbsoluteLength {
    gpui::AbsoluteLength::from(gpui::px(value))
}

fn px_def(value: f32) -> gpui::DefiniteLength {
    gpui::DefiniteLength::from(px_abs(value))
}

fn px_len(value: f32) -> gpui::Length {
    gpui::Length::from(px_def(value))
}

fn edges_len(value: f32) -> gpui::Edges<gpui::Length> {
    gpui::Edges {
        top: px_len(value),
        right: px_len(value),
        bottom: px_len(value),
        left: px_len(value),
    }
}

fn edges_def(value: f32) -> gpui::Edges<gpui::DefiniteLength> {
    gpui::Edges {
        top: px_def(value),
        right: px_def(value),
        bottom: px_def(value),
        left: px_def(value),
    }
}

fn edges_abs(value: f32) -> gpui::Edges<gpui::AbsoluteLength> {
    gpui::Edges {
        top: px_abs(value),
        right: px_abs(value),
        bottom: px_abs(value),
        left: px_abs(value),
    }
}
