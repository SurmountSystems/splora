//! One clipboard button for each calculator input.
//!
//! The inputs and buttons come from
//! `splora_frontend_shared::shows_copy_buttons_for_each_input`. This module
//! does not invent that list. The window is opened headless and closed
//! before this function returns.

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

/// Laid-out clipboard rows for each calculator input.
#[derive(Clone, Debug, PartialEq)]
pub struct ShowsCopyButtonsForEachInputPaint {
    /// Shared clipboard buttons this window drew.
    pub shown: splora_frontend_shared::CopyButtonsForEachInput,
    /// One line per shared button, in input order.
    pub visible_text: String,
    /// Height in pixels of each laid-out row.
    pub row_heights: Vec<f32>,
}

struct CopyButtonsForEachInputView {
    lines: Vec<String>,
    texts: Vec<Rc<RefCell<Option<String>>>>,
    slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
}

impl Render for CopyButtonsForEachInputView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = div()
            .id("copy-buttons-for-each-input")
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
        .id(("copy-buttons-for-each-input-row", index))
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
            panic!("copy buttons for each input row {index} bounds were not laid out")
        });
        let text = text_slot.borrow().clone().unwrap_or_else(|| {
            panic!("copy buttons for each input row {index} text was not laid out")
        });
        visible.push(text.trim().to_string());
        heights.push(bounds.size.height.as_f32());
    }
    (visible.join("\n"), heights)
}

/// Paint one visible clipboard control for each calculator input.
///
/// Opens one headless window, draws one row per shared button, and quits
/// before returning. Each row is the input name, the button field, and
/// whether that button is visible.
pub fn paint_shows_copy_buttons_for_each_input() -> ShowsCopyButtonsForEachInputPaint {
    let shown = splora_frontend_shared::shows_copy_buttons_for_each_input();
    let lines: Vec<String> = shown
        .inputs
        .iter()
        .zip(shown.buttons.iter())
        .map(|(input, button)| format!("{input} {} {}", button.field, button.visible))
        .collect();
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
        cx.open_window(options, move |_window, cx| {
            cx.new(|_| CopyButtonsForEachInputView {
                lines: lines_view,
                texts: texts_view,
                slots: slots_view,
            })
        })
        .expect("headless copy buttons for each input window");
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
    });
    let (visible_text, row_heights) = read_laid_out(&texts, &slots);
    ShowsCopyButtonsForEachInputPaint {
        shown,
        visible_text,
        row_heights,
    }
}
