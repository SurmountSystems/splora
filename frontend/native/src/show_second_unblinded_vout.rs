//! Second `.table-tx-vout` row for one `#blinded` group.
//!
//! Child, class, and amount come from
//! `splora_frontend_shared::show_second_unblinded_vout`. That return is child
//! 2, not the second parsed line. This module does not invent a height, a
//! route, or an icon label. Fee stays 0 inside the shared function. The
//! window is opened headless and closed before this function returns.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Bounds, Context, IntoElement, Pixels, Render, Window, WindowBounds, WindowOptions, canvas, div,
    point, prelude::*, px, size,
};
use gpui_platform::headless;

/// Width of one painted row, in pixels.
const ROW_WIDTH: f32 = 720.0;
/// Height of one painted row, in pixels.
const ROW_HEIGHT: f32 = 24.0;
/// Headless window width.
const WINDOW_WIDTH: f32 = 800.0;

struct SecondUnblindedVoutView {
    lines: Vec<String>,
    texts: Vec<Rc<RefCell<Option<String>>>>,
    slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
}

impl Render for SecondUnblindedVoutView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = div()
            .id("show-second-unblinded-vout")
            .w(px(WINDOW_WIDTH))
            .flex()
            .flex_col();
        let lines = self.lines.clone();
        let texts = self.texts.clone();
        let slots = self.slots.clone();
        for (index, ((line, text), slot)) in lines.into_iter().zip(texts).zip(slots).enumerate() {
            column = column.child(text_row(index, line, text, slot));
        }
        column
    }
}

fn text_row(
    index: usize,
    line: String,
    text_slot: Rc<RefCell<Option<String>>>,
    bounds_slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
) -> impl IntoElement {
    let laid = line.clone();
    div()
        .id(("show-second-unblinded-vout-row", index))
        .relative()
        .flex_shrink_0()
        .w(px(ROW_WIDTH))
        .h(px(ROW_HEIGHT))
        .child(line)
        .child(text_and_bounds_probe(laid, text_slot, bounds_slot))
}

/// Records the row text and bounds while GPUI lays the row out.
fn text_and_bounds_probe(
    line: String,
    text_slot: Rc<RefCell<Option<String>>>,
    bounds_slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
) -> impl IntoElement {
    canvas(
        move |bounds, _window, _app| {
            *bounds_slot.borrow_mut() = Some(bounds);
            *text_slot.borrow_mut() = Some(line.clone());
        },
        |_bounds, _recorded, _window, _app| {},
    )
    .absolute()
    .inset_0()
}

fn read_laid_out(
    texts: &[Rc<RefCell<Option<String>>>],
    slots: &[Rc<RefCell<Option<Bounds<Pixels>>>>],
) -> (String, Vec<f32>) {
    let mut visible = Vec::with_capacity(texts.len());
    let mut heights = Vec::with_capacity(slots.len());
    for (index, (text_slot, bounds_slot)) in texts.iter().zip(slots).enumerate() {
        let bounds = bounds_slot.borrow().unwrap_or_else(|| {
            panic!("second unblinded vout row {index} bounds were not laid out")
        });
        let text = text_slot
            .borrow()
            .clone()
            .unwrap_or_else(|| panic!("second unblinded vout row {index} text was not laid out"));
        visible.push(text.trim().to_string());
        heights.push(bounds.size.height.as_f32());
    }
    (visible.join("\n"), heights)
}

/// Markup for child `child` inside `.table-tx-vout`.
///
/// Leading rows have no class, so the shared amount is `tr:nth-child(child)`.
fn vout_markup(row: &splora_frontend_shared::SecondUnblindedVout) -> String {
    if row.row_class.is_empty() && row.amount.is_empty() {
        return String::new();
    }
    let mut body = String::new();
    let lead = usize::from(row.child.saturating_sub(1));
    for _ in 0..lead {
        body.push_str("<tr></tr>");
    }
    body.push_str("<tr class=\"");
    body.push_str(row.row_class);
    body.push_str("\"><td class=\"amount\">");
    body.push_str(&row.amount);
    body.push_str("</td></tr>");
    format!("<table class=\"table-tx-vout\"><tbody>{body}</tbody></table>")
}

fn nth_tr_class(markup: &str, child: u8) -> String {
    let mut rest = markup;
    let mut seen = 0u8;
    while let Some(at) = rest.find("<tr") {
        seen = seen.saturating_add(1);
        let tag = &rest[at..];
        let end = tag.find('>').unwrap_or(tag.len());
        if seen == child {
            return class_from_open_tag(&tag[..end]);
        }
        rest = &tag[end..];
    }
    String::new()
}

fn class_from_open_tag(open: &str) -> String {
    let Some(class_at) = open.find("class=\"") else {
        return String::new();
    };
    let class_value = &open[class_at + "class=\"".len()..];
    let end = class_value.find('"').unwrap_or(class_value.len());
    class_value[..end].to_string()
}

/// Paint the second unblinded vout row for `fragment`.
///
/// Returns `splora_frontend_shared::show_second_unblinded_vout` for that
/// fragment. Opens one headless window with `show: false`, draws that row
/// as `tr:nth-child` of the shared child index, and quits before returning.
pub fn paint_show_second_unblinded_vout(
    fragment: &str,
) -> splora_frontend_shared::SecondUnblindedVout {
    let row = splora_frontend_shared::show_second_unblinded_vout(fragment);
    let markup = vout_markup(&row);
    let lines = if markup.is_empty() {
        Vec::new()
    } else {
        vec![markup]
    };
    let texts: Vec<Rc<RefCell<Option<String>>>> = (0..lines.len())
        .map(|_| Rc::new(RefCell::new(None)))
        .collect();
    let slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>> = (0..lines.len())
        .map(|_| Rc::new(RefCell::new(None)))
        .collect();
    let lines_view = lines.clone();
    let texts_view = texts.clone();
    let slots_view = slots.clone();
    let window_height = (lines.len().max(1) as f32) * ROW_HEIGHT + 80.0;
    headless().run(move |cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.), px(0.)),
                size: size(px(WINDOW_WIDTH), px(window_height)),
            })),
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, move |_window, cx| {
            cx.new(|_| SecondUnblindedVoutView {
                lines: lines_view,
                texts: texts_view,
                slots: slots_view,
            })
        });
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless second unblinded vout window");
    });
    let (vout_row, row_heights) = read_laid_out(&texts, &slots);
    if !lines.is_empty() {
        if row_heights.is_empty() || row_heights.iter().any(|height| *height <= 0.0) {
            panic!("second unblinded vout row was not laid out, heights {row_heights:?}");
        }
        let painted_class = nth_tr_class(&vout_row, row.child);
        if painted_class != row.row_class {
            panic!(
                "child {} class was {painted_class}, shared class was {}",
                row.child, row.row_class
            );
        }
        if !vout_row.contains(row.amount.as_str()) {
            panic!("laid-out child {} missing the shared amount", row.child);
        }
    }
    row
}
