//! Next Block and Previous Block icons after the newest confirmed block,
//! then the heading and route after one ArrowLeft.
//!
//! Labels, the heading, and the route come from
//! `splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_left`.
//! This module does not recompute that return. The window is opened headless
//! and closed before this function returns.

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

/// Laid-out labels, heading, and route for the ArrowLeft step.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadsFirstBlockchainBlockArrowLeftPaint {
    /// Shared return this window drew.
    pub shown: splora_frontend_shared::FirstBlockchainBlockArrowLeft,
    /// One line per label, in Cypress check order.
    pub visible_text: String,
    /// Height in pixels of each laid-out row.
    pub row_heights: Vec<f32>,
}

struct FirstBlockchainBlockArrowLeftView {
    lines: Vec<String>,
    texts: Vec<Rc<RefCell<Option<String>>>>,
    slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
}

impl Render for FirstBlockchainBlockArrowLeftView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = div()
            .id("first-blockchain-block-arrow-left")
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
        .id(("first-blockchain-block-arrow-left-row", index))
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
            panic!("first blockchain block arrow left row {index} bounds were not laid out")
        });
        let text = text_slot.borrow().clone().unwrap_or_else(|| {
            panic!("first blockchain block arrow left row {index} text was not laid out")
        });
        visible.push(text.trim().to_string());
        heights.push(bounds.size.height.as_f32());
    }
    (visible.join("\n"), heights)
}

/// The integration test formats these four lines the same way.
fn icon_lines(shown: &splora_frontend_shared::FirstBlockchainBlockArrowLeft) -> Vec<String> {
    let newest = &shown.after_selecting_newest;
    vec![
        format!(
            "newest confirmed Next Block {}",
            newest.next_block_icon_visible
        ),
        format!(
            "newest confirmed Previous Block {}",
            newest.previous_block_icon_visible
        ),
        format!("after ArrowLeft heading {}", shown.heading),
        format!("after ArrowLeft route {}", shown.route),
    ]
}

/// Paint Next Block and Previous Block after the newest block, then ArrowLeft.
///
/// Opens one headless window, draws one row per shared label, and quits
/// before returning.
pub fn paint_loads_first_blockchain_block_visible_and_keypress_arrow_left()
-> LoadsFirstBlockchainBlockArrowLeftPaint {
    let shown =
        splora_frontend_shared::loads_first_blockchain_block_visible_and_keypress_arrow_left();
    let lines = icon_lines(&shown);
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
            cx.new(|_| FirstBlockchainBlockArrowLeftView {
                lines: lines_view,
                texts: texts_view,
                slots: slots_view,
            })
        })
        .expect("headless first blockchain block arrow left window");
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
    });
    let (visible_text, row_heights) = read_laid_out(&texts, &slots);
    LoadsFirstBlockchainBlockArrowLeftPaint {
        shown,
        visible_text,
        row_heights,
    }
}
