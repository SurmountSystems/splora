//! Recent transaction list as a later batch arrives.
//!
//! Visible txids come from
//! `splora_frontend_shared::updates_the_transaction_list_over_time`.
//! Anchor text is the txid from that function, which copies the caller's
//! batch arguments. This module does not slice a txid to 10 characters.
//! The window is opened headless and closed before this function returns.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Bounds, Context, IntoElement, Pixels, Render, Window, WindowBounds, WindowOptions, canvas, div,
    point, prelude::*, px, size,
};
use gpui_platform::headless;

/// Width of one transaction row, in pixels.
const ROW_WIDTH: f32 = 720.0;
/// Height of one transaction row, in pixels.
const ROW_HEIGHT: f32 = 24.0;
/// Headless window width. Matches the other recent-list paints' desktop width.
const WINDOW_WIDTH: f32 = 800.0;
/// Headless window height. Tall enough for both batches stacked.
const WINDOW_HEIGHT: f32 = 800.0;

/// Laid-out anchor text and row heights after the first batch, then the later batch.
#[derive(Clone, Debug, PartialEq)]
pub struct TransactionListOverTimePaint {
    /// Joined anchor text of the first batch. Each anchor is the txid argument.
    pub first_visible_text: String,
    /// Joined anchor text after the later batch. The second batch comes first.
    pub later_visible_text: String,
    /// Laid-out row count after the first batch.
    pub first_row_count: usize,
    /// Laid-out row count after the later batch.
    pub later_row_count: usize,
    /// Height in pixels of each laid-out first-batch row.
    pub first_row_heights: Vec<f32>,
    /// Height in pixels of each laid-out later row.
    pub later_row_heights: Vec<f32>,
}

struct TransactionListView {
    first_txids: Vec<String>,
    later_txids: Vec<String>,
    first_text: Vec<Rc<RefCell<Option<String>>>>,
    later_text: Vec<Rc<RefCell<Option<String>>>>,
    first_slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
    later_slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
}

impl Render for TransactionListView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("transactions-list")
            .w(px(WINDOW_WIDTH))
            .flex()
            .flex_col()
            .child(tx_column(
                "after-first",
                self.first_txids.clone(),
                self.first_text.clone(),
                self.first_slots.clone(),
            ))
            .child(tx_column(
                "after-later",
                self.later_txids.clone(),
                self.later_text.clone(),
                self.later_slots.clone(),
            ))
    }
}

fn tx_column(
    list_key: &'static str,
    txids: Vec<String>,
    texts: Vec<Rc<RefCell<Option<String>>>>,
    slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
) -> impl IntoElement {
    let mut column = div().id(list_key).flex().flex_col();
    for (index, ((txid, text), slot)) in txids.into_iter().zip(texts).zip(slots).enumerate() {
        column = column.child(tx_row(list_key, index, txid, text, slot));
    }
    column
}

fn tx_row(
    list_key: &'static str,
    index: usize,
    txid: String,
    text_slot: Rc<RefCell<Option<String>>>,
    bounds_slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
) -> impl IntoElement {
    let laid_txid = txid.clone();
    div()
        .id((list_key, index))
        .relative()
        .flex_shrink_0()
        .w(px(ROW_WIDTH))
        .h(px(ROW_HEIGHT))
        .child(txid)
        .child(text_and_bounds_probe(laid_txid, text_slot, bounds_slot))
}

/// Records the row's txid and bounds while GPUI lays the row out.
fn text_and_bounds_probe(
    txid: String,
    text_slot: Rc<RefCell<Option<String>>>,
    bounds_slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
) -> impl IntoElement {
    canvas(
        move |bounds, _window, _app| {
            *bounds_slot.borrow_mut() = Some(bounds);
            *text_slot.borrow_mut() = Some(txid.clone());
        },
        |_bounds, _recorded, _window, _app| {},
    )
    .absolute()
    .inset_0()
}

fn empty_slots(
    count: usize,
) -> (
    Vec<Rc<RefCell<Option<String>>>>,
    Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
) {
    let texts = (0..count).map(|_| Rc::new(RefCell::new(None))).collect();
    let bounds = (0..count).map(|_| Rc::new(RefCell::new(None))).collect();
    (texts, bounds)
}

fn read_laid_out(
    label: &str,
    texts: &[Rc<RefCell<Option<String>>>],
    slots: &[Rc<RefCell<Option<Bounds<Pixels>>>>],
) -> (String, Vec<f32>) {
    let mut visible = Vec::with_capacity(texts.len());
    let mut heights = Vec::with_capacity(slots.len());
    for (index, (text_slot, bounds_slot)) in texts.iter().zip(slots).enumerate() {
        let bounds = bounds_slot
            .borrow()
            .unwrap_or_else(|| panic!("{label} row {index} bounds were not laid out"));
        let text = text_slot
            .borrow()
            .clone()
            .unwrap_or_else(|| panic!("{label} row {index} text was not laid out"));
        visible.push(text.trim().to_string());
        heights.push(bounds.size.height.as_f32());
    }
    (visible.join(" "), heights)
}

/// Paint the recent transaction list after the first batch and after the later batch.
///
/// `first` and `second` are the two batches of txid arguments. The shared
/// function chooses which ids are visible. Each anchor paints that id in
/// full. Opens one headless window, draws once, and quits before returning.
pub fn paint_updates_the_transaction_list_over_time(
    first: &[&str; 6],
    second: &[&str; 6],
) -> TransactionListOverTimePaint {
    let (after_first, after_second) =
        splora_frontend_shared::updates_the_transaction_list_over_time(first, second);
    let first_txids: Vec<String> = after_first.iter().copied().map(str::to_string).collect();
    let later_txids: Vec<String> = after_second.iter().copied().map(str::to_string).collect();
    let (first_text, first_slots) = empty_slots(first_txids.len());
    let (later_text, later_slots) = empty_slots(later_txids.len());
    let first_txids_view = first_txids.clone();
    let later_txids_view = later_txids.clone();
    let first_text_view = first_text.clone();
    let later_text_view = later_text.clone();
    let first_slots_view = first_slots.clone();
    let later_slots_view = later_slots.clone();
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
        cx.open_window(options, move |_window, cx| {
            cx.new(|_| TransactionListView {
                first_txids: first_txids_view,
                later_txids: later_txids_view,
                first_text: first_text_view,
                later_text: later_text_view,
                first_slots: first_slots_view,
                later_slots: later_slots_view,
            })
        })
        .expect("headless transaction list window");
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
    });
    let (first_visible_text, first_row_heights) =
        read_laid_out("first batch", &first_text, &first_slots);
    let (later_visible_text, later_row_heights) =
        read_laid_out("later batch", &later_text, &later_slots);
    TransactionListOverTimePaint {
        first_row_count: first_row_heights.len(),
        later_row_count: later_row_heights.len(),
        first_visible_text,
        later_visible_text,
        first_row_heights,
        later_row_heights,
    }
}
