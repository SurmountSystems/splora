//! Dashboard `.table-cell-fiat` cells after JPY, then after USD.
//!
//! The cell strings come from
//! `splora_frontend_shared::correctly_formats_when_switching_from_jpy_back_to_usd`.
//! This module does not invent those strings and does not read a price.
//! One headless window is opened and closed before this function returns.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Bounds, Context, IntoElement, Pixels, Render, Window, WindowBounds, WindowOptions, canvas, div,
    point, prelude::*, px, size,
};
use gpui_platform::headless;

/// Width of one painted fiat cell, in pixels.
const ROW_WIDTH: f32 = 720.0;
/// Height of one painted fiat cell, in pixels.
const ROW_HEIGHT: f32 = 24.0;
/// Headless window width.
const WINDOW_WIDTH: f32 = 800.0;

struct JpyBackToUsdSwitchView {
    lines: Vec<String>,
    texts: Vec<Rc<RefCell<Option<String>>>>,
    slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
}

impl Render for JpyBackToUsdSwitchView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = div()
            .id("correctly-formats-when-switching-from-jpy-back-to-usd")
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
        .id(("correctly-formats-jpy-back-to-usd-row", index))
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
            panic!("JPY back to USD fiat cell row {index} bounds were not laid out")
        });
        let text = text_slot.borrow().clone().unwrap_or_else(|| {
            panic!("JPY back to USD fiat cell row {index} text was not laid out")
        });
        visible.push(text.trim().to_string());
        heights.push(bounds.size.height.as_f32());
    }
    (visible.join("\n"), heights)
}

/// Paint the second fiat cell after JPY, then after USD.
///
/// Opens one headless window, draws one row per shared cell, and quits
/// before returning. The return value is the shared call.
pub fn paint_correctly_formats_when_switching_from_jpy_back_to_usd() -> [Vec<String>; 2] {
    let cells = splora_frontend_shared::correctly_formats_when_switching_from_jpy_back_to_usd();
    let lines: Vec<String> = cells.iter().flatten().cloned().collect();
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
            cx.new(|_| JpyBackToUsdSwitchView {
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
        opened.expect("headless JPY back to USD switch window");
    });
    let (visible_text, row_heights) = read_laid_out(&texts, &slots);
    assert_eq!(
        visible_text,
        lines.join("\n"),
        "laid-out JPY then USD fiat cells must match the shared cells"
    );
    assert!(
        row_heights.len() == lines.len() && row_heights.iter().all(|height| *height > 0.0),
        "JPY back to USD fiat cell rows were not laid out, heights {row_heights:?}"
    );
    cells
}
