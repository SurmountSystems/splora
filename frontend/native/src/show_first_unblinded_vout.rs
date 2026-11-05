//! First `.table-tx-vout` row for one `#blinded` group.
//!
//! Class and amount come from
//! `splora_frontend_shared::show_first_unblinded_vout`. That return has
//! `row_class` and `amount` only. It has no height, no route, and no icon
//! visibility, so this module does not invent those and does not paint icon
//! labels. Fee stays 0 inside the shared function. The window is opened
//! headless and closed before this function returns.

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

/// Laid-out first vout row. Class and amount are the shared return.
#[derive(Clone, Debug, PartialEq)]
pub struct ShowFirstUnblindedVoutPaint {
    /// Class on `tr:nth-child(1)` inside `.table-tx-vout`.
    pub row_class: String,
    /// Text of that row's `.amount`.
    pub amount: String,
    /// Markup GPUI laid out for the vout row.
    pub vout_row: String,
    /// Height in pixels of each laid-out row.
    pub row_heights: Vec<f32>,
}

struct FirstUnblindedVoutView {
    lines: Vec<String>,
    texts: Vec<Rc<RefCell<Option<String>>>>,
    slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
}

impl Render for FirstUnblindedVoutView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = div()
            .id("show-first-unblinded-vout")
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
        .id(("show-first-unblinded-vout-row", index))
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
        let bounds = bounds_slot
            .borrow()
            .unwrap_or_else(|| panic!("first unblinded vout row {index} bounds were not laid out"));
        let text = text_slot
            .borrow()
            .clone()
            .unwrap_or_else(|| panic!("first unblinded vout row {index} text was not laid out"));
        visible.push(text.trim().to_string());
        heights.push(bounds.size.height.as_f32());
    }
    (visible.join("\n"), heights)
}

/// Markup for the first vout row. Class and amount are the shared fields.
fn vout_markup(row_class: &str, amount: &str) -> String {
    format!(
        "<table class=\"table-tx-vout\"><tbody><tr class=\"{row_class}\"><td class=\"amount\">{amount}</td></tr></tbody></table>"
    )
}

/// Paint the first unblinded vout row for `fragment`.
///
/// Opens one headless window, draws the shared class and amount inside
/// `.table-tx-vout`, and quits before returning. Icon labels are omitted
/// because the shared return does not say any icon is visible.
pub fn paint_show_first_unblinded_vout(fragment: &str) -> ShowFirstUnblindedVoutPaint {
    let row = splora_frontend_shared::show_first_unblinded_vout(fragment);
    let row_class = row.row_class.to_string();
    let amount = row.amount.clone();
    let mut lines = Vec::new();
    if !row_class.is_empty() || !amount.is_empty() {
        lines.push(vout_markup(&row_class, &amount));
    }
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
            cx.new(|_| FirstUnblindedVoutView {
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
        opened.expect("headless first unblinded vout window");
    });
    let (vout_row, row_heights) = read_laid_out(&texts, &slots);
    ShowFirstUnblindedVoutPaint {
        row_class,
        amount,
        vout_row,
        row_heights,
    }
}
