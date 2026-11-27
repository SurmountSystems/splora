//! Native paint checks against the shared style catalog.
//! This test does not open a window.

use splora_frontend_shared::style_rule::{GpuiOutcome, StyleRule};
use splora_native::style_paint::{
    CursorStyle, Display, LonghandPaint, Overflow, Style, catalog_rules_paint,
    catalog_slice_paints, first_catalog_paint_miss, gpui_paints, paint_longhand, paint_rule,
    paint_splora_style, recorded_theme_colors, style_matches_longhand, unhostable_style_names,
    unwritten_paintable_longhands,
};

#[test]
fn gpui_paints_paint_outcome_and_locks_theme_colors() {
    assert!(gpui_paints(GpuiOutcome::Paint, ""));
    assert!(gpui_paints(
        GpuiOutcome::NotAWidget {
            feature: "keyframes",
        },
        "keyframes",
    ));
    assert!(!gpui_paints(GpuiOutcome::NotAWidget { feature: "" }, "",));

    let colors = recorded_theme_colors();
    assert_eq!(colors.background, "#000000");
    assert_eq!(colors.background_rgb, 0x000000);
    assert_eq!(colors.text, "#FFFFFF");
    assert_eq!(colors.accent, "#FFD100");
}

#[test]
fn present_catalog_rules_match_paint_shape() {
    let rules = splora_frontend_shared::all_rules();
    if rules.is_empty() {
        assert!(!catalog_rules_paint());
        return;
    }
    let miss = first_catalog_paint_miss(&rules);
    assert!(
        catalog_rules_paint(),
        "{}",
        miss.unwrap_or_else(|| {
            "a catalog rule's paint_rule field does not equal the specified longhand".to_string()
        })
    );
}

/// The theme sample is absolute. `.block-face` specifies relative, so that
/// absolute value must not count as the rule's position.
#[test]
fn position_relative_is_not_the_theme_absolute() {
    use gpui::Position;
    let theme = paint_splora_style();
    assert_eq!(theme.position, Position::Absolute);
    assert!(
        !style_matches_longhand(&theme, "position", "relative"),
        "theme Position::Absolute counted as position:relative"
    );
    let rules = splora_frontend_shared::all_rules();
    let face = catalog_rule(&rules, "address-blocks", ".block-face");
    let painted = paint_rule(face);
    assert_eq!(painted.style.position, Position::Relative);
    assert_ne!(painted.style.position, theme.position);
    assert!(
        catalog_slice_paints(&[face]),
        "{}",
        first_catalog_paint_miss(&[face]).unwrap_or_else(|| "block-face".to_string())
    );
}

/// `position:fixed` is the feature `position:fixed`. The theme sample's
/// absolute position does not satisfy it. `position:absolute` has to copy
/// [`Position::Absolute`] onto a fresh style, because the default is relative.
#[test]
fn position_fixed_is_not_theme_absolute_and_absolute_is_copied() {
    use gpui::Position;
    let theme = paint_splora_style();
    assert_eq!(theme.position, Position::Absolute);
    let rules = splora_frontend_shared::all_rules();
    let fixed = catalog_rule(&rules, "api-docs", "#doc-nav-desktop.fixed");
    let painted = paint_rule(fixed);
    assert!(
        painted
            .features
            .iter()
            .any(|feature| *feature == "position:fixed"),
        "features={:?}",
        painted.features
    );
    assert_ne!(painted.style.position, Position::Absolute);
    assert!(!style_matches_longhand(&painted.style, "position", "fixed"));
    assert!(
        catalog_slice_paints(&[fixed]),
        "{}",
        first_catalog_paint_miss(&[fixed]).unwrap_or_else(|| "fixed".to_string())
    );

    let absolute = catalog_rule(&rules, "styles.scss", ".progress-text");
    let painted = paint_rule(absolute);
    assert_eq!(painted.style.position, Position::Absolute);
    assert_ne!(painted.style.position, Style::default().position);
    assert!(style_matches_longhand(
        &painted.style,
        "position",
        "absolute"
    ));
    assert!(!style_matches_longhand(&theme, "position", "relative"));
}

/// `:host` display and flex-wrap are the host box. A colon in `:host` must not
/// leave display at the default or skip `flex-wrap: wrap`.
#[test]
fn host_box_longhands_are_resting_paint() {
    use gpui::FlexWrap;
    let rules = splora_frontend_shared::all_rules();
    let header = catalog_rule(&rules, "address-block-group-header", ":host");
    let painted = paint_rule(header);
    assert!(painted.wrote_display, "features={:?}", painted.features);
    assert_eq!(painted.style.display, Display::Flex);
    assert_ne!(painted.style.display, Style::default().display);
    assert_eq!(painted.style.flex_wrap, FlexWrap::Wrap);
    assert_ne!(painted.style.flex_wrap, Style::default().flex_wrap);
    assert!(
        catalog_slice_paints(&[header]),
        "{}",
        first_catalog_paint_miss(&[header]).unwrap_or_else(|| "host header".to_string())
    );

    let blocks = catalog_rule(&rules, "address-blocks.component", ":host");
    let painted = paint_rule(blocks);
    assert!(painted.wrote_display, "features={:?}", painted.features);
    assert_eq!(painted.style.display, Display::Block);
    // gpui Style::default sets display to Block, so block equals that default.
    // margin-bottom: 1.5rem is the longhand that is not the zero margin.
    assert_eq!(Style::default().display, Display::Block);
    assert_ne!(painted.style.margin.bottom, Style::default().margin.bottom);
    assert!(style_matches_longhand(
        &painted.style,
        "margin-bottom",
        "1.5rem"
    ));
    assert!(
        catalog_slice_paints(&[blocks]),
        "{}",
        first_catalog_paint_miss(&[blocks]).unwrap_or_else(|| "host blocks".to_string())
    );

    let focus_visible = catalog_rule(&rules, "address-block-group-header", ":host:focus-visible");
    let painted = paint_rule(focus_visible);
    assert!(
        painted.features.iter().any(|feature| *feature == ":host"),
        "features={:?}",
        painted.features
    );
    assert!(
        painted
            .features
            .iter()
            .any(|feature| *feature == ":focus-visible"),
        "features={:?}",
        painted.features
    );
    assert!(!painted.wrote_display, "features={:?}", painted.features);
    assert_eq!(painted.style.display, Style::default().display);
    assert_eq!(painted.style.flex_wrap, Style::default().flex_wrap);
    assert!(
        catalog_slice_paints(&[focus_visible]),
        "{}",
        first_catalog_paint_miss(&[focus_visible]).unwrap_or_else(|| "focus-visible".to_string())
    );

    let media = catalog_rule(
        &rules,
        "address-blocks.component",
        "@media (min-width: 768px) and (min-height: 700px) { :host }",
    );
    let painted = paint_rule(media);
    assert!(
        painted.features.iter().any(|feature| *feature == "@media"),
        "features={:?}",
        painted.features
    );
    assert!(
        painted.features.iter().any(|feature| *feature == ":host"),
        "features={:?}",
        painted.features
    );
    assert!(!painted.wrote_display, "features={:?}", painted.features);
    assert_eq!(painted.style.position, Style::default().position);
    assert!(
        catalog_slice_paints(&[media]),
        "{}",
        first_catalog_paint_miss(&[media]).unwrap_or_else(|| "media host".to_string())
    );
}

/// `:not(...)` still selects an element. Its longhands are resting paint.
/// `:hover` and `:focus` on the same selector stay off the Style.
#[test]
fn not_selector_longhands_are_resting_paint() {
    let rules = splora_frontend_shared::all_rules();
    let label = catalog_rule(
        &rules,
        "styles.scss",
        "form label:not(.form-check-label):not(.btn)",
    );
    let painted = paint_rule(label);
    assert!(
        painted.features.iter().all(|feature| *feature != ":not"),
        "features={:?}",
        painted.features
    );
    assert_ne!(painted.style.margin.bottom, Style::default().margin.bottom);
    assert!(style_matches_longhand(
        &painted.style,
        "margin-bottom",
        "0.5rem"
    ));
    assert!(
        catalog_slice_paints(&[label]),
        "{}",
        first_catalog_paint_miss(&[label]).unwrap_or_else(|| "not label".to_string())
    );

    let cell = catalog_rule(&rules, "styles.scss", ".table > :not(caption) > * > *");
    let painted = paint_rule(cell);
    assert!(
        painted.features.iter().all(|feature| *feature != ":not"),
        "features={:?}",
        painted.features
    );
    assert!(style_matches_longhand(
        &painted.style,
        "border-bottom-width",
        "0px"
    ));
    assert!(
        catalog_slice_paints(&[cell]),
        "{}",
        first_catalog_paint_miss(&[cell]).unwrap_or_else(|| "not cell".to_string())
    );
}

/// `::-webkit-scrollbar` and `:focus` are those tokens. The catalog word
/// `pseudo` is not a feature, and `outline` / `width` are not resting paint.
#[test]
fn scrollbar_and_focus_are_named_tokens_not_resting_paint() {
    let rules = splora_frontend_shared::all_rules();
    let bar = catalog_rule(&rules, "api-docs", "::-webkit-scrollbar");
    let painted = paint_rule(bar);
    assert!(
        painted
            .features
            .iter()
            .any(|feature| *feature == "::-webkit-scrollbar"),
        "features={:?}",
        painted.features
    );
    assert!(
        painted.features.iter().all(|feature| *feature != "pseudo"),
        "features={:?}",
        painted.features
    );
    assert_eq!(
        painted.style.size.width,
        Style::default().size.width,
        "scrollbar width was resting paint"
    );
    assert!(
        catalog_slice_paints(&[bar]),
        "{}",
        first_catalog_paint_miss(&[bar]).unwrap_or_else(|| "scrollbar".to_string())
    );

    let focus = catalog_rule(&rules, "styles.scss", ":focus");
    let painted = paint_rule(focus);
    assert!(
        painted.features.iter().any(|feature| *feature == ":focus"),
        "features={:?}",
        painted.features
    );
    assert!(
        painted
            .features
            .iter()
            .all(|feature| *feature != "pseudo" && *feature != "outline" && *feature != "unmapped"),
        "features={:?}",
        painted.features
    );
    assert_eq!(painted.style.position, Style::default().position);
    assert_eq!(painted.style.display, Style::default().display);
    assert!(
        catalog_slice_paints(&[focus]),
        "{}",
        first_catalog_paint_miss(&[focus]).unwrap_or_else(|| "focus".to_string())
    );
}

#[test]
fn empty_catalog_fails() {
    assert!(!catalog_slice_paints(&[]));
}

#[test]
fn paint_splora_style_writes_display_and_opacity() {
    let style = paint_splora_style();
    assert_eq!(style.display, Display::Flex);
    assert_eq!(style.opacity, Some(1.0));
    assert!(unwritten_paintable_longhands(&style).is_empty());
}

/// Text color does not stand in for the other text longhands.
#[test]
fn font_weight_is_not_accepted_as_the_color_arm() {
    let mut style = paint_splora_style();
    assert_eq!(style.display, Display::Flex);
    assert!(style.text.color.is_some(), "text color stays written");
    style.text.font_weight = None;
    style.text.font_style = None;
    style.text.line_height = None;
    style.text.text_align = None;
    style.text.white_space = None;
    let missing = unwritten_paintable_longhands(&style);
    for name in [
        "font-weight",
        "font-style",
        "line-height",
        "text-align",
        "white-space",
    ] {
        assert!(
            missing.contains(&name),
            "{name} was accepted as the color arm: {missing:?}"
        );
    }
    assert!(
        !missing.contains(&"color"),
        "clearing the text fields must not clear the color witness: {missing:?}"
    );
}

/// One sample, not a matrix of every display and every overflow.
/// That sample is Flex, Hidden, PointingHand, `repeat(12, min-content)`,
/// and Absolute, with background `#000000`, text `#FFFFFF`, and accent `#FFD100`
/// on the Style color fields.
#[test]
fn style_fields_use_the_named_variant() {
    use gpui::{GridTemplate, GridTemplateMinSize, Position};
    let painted = paint_splora_style();
    assert_eq!(painted.display, Display::Flex);
    assert_eq!(painted.overflow.x, Overflow::Hidden);
    assert_eq!(painted.overflow.y, Overflow::Hidden);
    assert_eq!(painted.mouse_cursor, Some(CursorStyle::PointingHand));
    assert_eq!(
        painted.grid_cols,
        Some(GridTemplate {
            repeat: 12,
            min_size: GridTemplateMinSize::MinContent,
        })
    );
    assert_eq!(painted.position, Position::Absolute);

    let colors = recorded_theme_colors();
    assert_eq!(colors.background, "#000000");
    assert_eq!(colors.text, "#FFFFFF");
    assert_eq!(colors.accent, "#FFD100");
    let background: gpui::Hsla = gpui::rgb(splora_native::BACKGROUND_RGB).into();
    let text: gpui::Hsla = gpui::rgb(splora_native::TEXT_RGB).into();
    let accent: gpui::Hsla = gpui::rgb(splora_native::ACCENT_RGB).into();
    assert_eq!(
        painted.background,
        Some(background.into()),
        "background field is #000000"
    );
    assert_eq!(
        painted.text.color,
        Some(text),
        "text color field is #FFFFFF"
    );
    assert_eq!(
        painted.border_color,
        Some(accent),
        "accent field is #FFD100"
    );

    let mut color_only = painted;
    color_only.text.font_weight = None;
    color_only.text.font_style = None;
    color_only.text.line_height = None;
    color_only.text.text_align = None;
    color_only.text.white_space = None;
    let missing = unwritten_paintable_longhands(&color_only);
    for name in [
        "font-weight",
        "font-style",
        "line-height",
        "text-align",
        "white-space",
    ] {
        assert!(
            missing.contains(&name),
            "a text color write counted as {name}: {missing:?}"
        );
    }
    assert!(
        !missing.contains(&"color"),
        "clearing the text fields must not clear the color witness: {missing:?}"
    );
}

fn catalog_rule<'a>(
    rules: &'a [&'a StyleRule],
    source_part: &str,
    selector: &str,
) -> &'a StyleRule {
    rules
        .iter()
        .copied()
        .find(|rule| rule.source_path.contains(source_part) && rule.selector_path == selector)
        .unwrap_or_else(|| panic!("missing rule {source_part} {selector}"))
}

/// A display:grid rule writes Display::Grid. The theme sample stays Flex.
#[test]
fn display_grid_rule_writes_display_grid() {
    let rules = splora_frontend_shared::all_rules();
    let face = catalog_rule(&rules, "address-blocks", ".block-face");
    let painted = paint_rule(face);
    assert!(painted.wrote_display);
    assert_eq!(painted.style.display, Display::Grid);
    let theme = paint_splora_style();
    assert_eq!(theme.display, Display::Flex);
    assert_ne!(painted.style.display, theme.display);
}

/// overflow:visible writes Overflow::Visible on both axes. The theme sample stays Hidden.
#[test]
fn overflow_visible_rule_writes_overflow_visible() {
    let rules = splora_frontend_shared::all_rules();
    let face = catalog_rule(&rules, "address-blocks", ".block-face");
    let painted = paint_rule(face);
    assert!(painted.wrote_overflow);
    assert_eq!(painted.style.overflow.x, Overflow::Visible);
    assert_eq!(painted.style.overflow.y, Overflow::Visible);
    let theme = paint_splora_style();
    assert_eq!(theme.overflow.x, Overflow::Hidden);
    assert_ne!(painted.style.overflow.x, theme.overflow.x);
}

/// overflow-x:auto is the named feature. It does not write Overflow, and the
/// theme sample's Hidden overflow does not count as that write.
#[test]
fn overflow_x_auto_is_a_named_feature_and_does_not_write_overflow() {
    let rules = splora_frontend_shared::all_rules();
    let strip = catalog_rule(&rules, "address-blocks", ".activity-strip");
    let painted = paint_rule(strip);
    assert!(!painted.wrote_overflow);
    assert!(
        painted
            .features
            .iter()
            .any(|feature| *feature == "overflow-x:auto"),
        "features={:?}",
        painted.features
    );
    let theme = paint_splora_style();
    assert_eq!(theme.overflow.x, Overflow::Hidden);
    assert!(!style_matches_longhand(&theme, "overflow-x", "auto"));
    match paint_longhand("overflow-x", "auto") {
        LonghandPaint::Feature(name) => assert_eq!(name, "overflow-x:auto"),
        LonghandPaint::Written(_) => panic!("overflow-x:auto wrote Overflow"),
    }
}

/// cursor:grab is OpenHand on a fresh Style. The theme sample's PointingHand
/// does not count as that write.
#[test]
fn cursor_grab_is_open_hand_not_the_theme_pointing_hand() {
    let theme = paint_splora_style();
    assert_eq!(theme.mouse_cursor, Some(CursorStyle::PointingHand));
    assert!(!style_matches_longhand(&theme, "cursor", "grab"));
    match paint_longhand("cursor", "grab") {
        LonghandPaint::Written(style) => {
            assert_eq!(style.mouse_cursor, Some(CursorStyle::OpenHand));
            assert_ne!(style.mouse_cursor, theme.mouse_cursor);
        }
        LonghandPaint::Feature(name) => panic!("cursor:grab was feature {name}, not OpenHand"),
    }
    let rules = splora_frontend_shared::all_rules();
    let strip = catalog_rule(&rules, "address-blocks", ".activity-strip");
    let painted = paint_rule(strip);
    assert_eq!(painted.style.mouse_cursor, Some(CursorStyle::OpenHand));
}

#[test]
fn unhostable_names_are_recorded() {
    let names = unhostable_style_names();
    for required in [
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
    ] {
        assert!(names.contains(&required), "{required}");
    }
}
