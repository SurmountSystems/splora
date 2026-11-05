//! Mining-dashboard mined block lines.
//!
//! The seven strings come from
//! `splora_frontend_shared::shows_the_mined_blocks`.
//! This module does not invent those strings.
//! The window is opened headless and closed before this function returns.

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
/// Headless window height. Tall enough for the seven mined-block rows.
const WINDOW_HEIGHT: f32 = 280.0;

struct MinedBlocksView {
    lines: Vec<String>,
    texts: Vec<Rc<RefCell<Option<String>>>>,
    slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
}

impl Render for MinedBlocksView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = div()
            .id("mined-blocks")
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
        .id(("mined-blocks-row", index))
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
) -> (Vec<String>, Vec<f32>) {
    let mut visible = Vec::with_capacity(texts.len());
    let mut heights = Vec::with_capacity(slots.len());
    for (index, (text_slot, bounds_slot)) in texts.iter().zip(slots).enumerate() {
        let bounds = bounds_slot
            .borrow()
            .unwrap_or_else(|| panic!("mined blocks row {index} bounds were not laid out"));
        let text = text_slot
            .borrow()
            .clone()
            .unwrap_or_else(|| panic!("mined blocks row {index} text was not laid out"));
        visible.push(text.trim().to_string());
        heights.push(bounds.size.height.as_f32());
    }
    (visible, heights)
}

/// Paint the seven mined-block lines for the fields the caller passes.
///
/// Opens one headless window with `show: false`, draws the shared lines,
/// and calls `cx.quit()` before returning, including when the window fails
/// to open. The lines stay in
/// `splora_frontend_shared::shows_the_mined_blocks`.
pub fn paint_shows_the_mined_blocks(
    height: u64,
    median_fee: u64,
    fee_range: &[u64],
    total_fee_sats: u64,
    transaction_count: u64,
    block_timestamp: u64,
    clock_unix: u64,
    pool_name: &str,
) -> [String; 7] {
    let lines = splora_frontend_shared::shows_the_mined_blocks(
        height,
        median_fee,
        fee_range,
        total_fee_sats,
        transaction_count,
        block_timestamp,
        clock_unix,
        pool_name,
    );
    let texts: Vec<Rc<RefCell<Option<String>>>> = (0..lines.len())
        .map(|_| Rc::new(RefCell::new(None)))
        .collect();
    let slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>> = (0..lines.len())
        .map(|_| Rc::new(RefCell::new(None)))
        .collect();
    let lines_view = lines.to_vec();
    let texts_view = texts.clone();
    let slots_view = slots.clone();
    headless().run(move |cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.), px(0.)),
                size: size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)),
            })),
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, move |_window, cx| {
            cx.new(|_| MinedBlocksView {
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
        opened.expect("headless mined blocks window");
    });
    let (visible, heights) = read_laid_out(&texts, &slots);
    assert_eq!(
        visible, lines,
        "laid-out mined block lines must match the shared lines"
    );
    assert!(
        heights.len() == lines.len() && heights.iter().all(|height| *height > 0.0),
        "mined block rows were not laid out"
    );
    splora_frontend_shared::shows_the_mined_blocks(
        height,
        median_fee,
        fee_range,
        total_fee_sats,
        transaction_count,
        block_timestamp,
        clock_unix,
        pool_name,
    )
}
