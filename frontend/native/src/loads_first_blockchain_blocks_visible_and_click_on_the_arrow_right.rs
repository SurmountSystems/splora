//! Newest confirmed block click, then one Previous Block click.
//!
//! Labels and the route come from
//! `splora_frontend_shared::loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right`.
//! Each icon label is included only when that return says the icon is visible.
//! This module does not invent a block hash. The window is opened headless
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

/// Laid-out labels and route after the newest block click, then Previous Block.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadsFirstBlockchainBlocksClickArrowRightPaint {
    /// Visible icon labels after the newest confirmed block click.
    pub after_selecting_newest: String,
    /// Visible icon labels after the Previous Block click, then the route.
    pub after_previous_block_click: String,
    /// Confirmed height after the Previous Block click.
    pub height: u32,
    /// Route after the Previous Block click.
    pub route: String,
    /// Height in pixels of each laid-out row.
    pub row_heights: Vec<f32>,
}

struct FirstBlockchainBlocksClickArrowRightView {
    lines: Vec<String>,
    texts: Vec<Rc<RefCell<Option<String>>>>,
    slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
}

impl Render for FirstBlockchainBlocksClickArrowRightView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = div()
            .id("first-blockchain-blocks-click-arrow-right")
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
        .id(("first-blockchain-blocks-click-arrow-right-row", index))
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
            panic!("first blockchain blocks click arrow right row {index} bounds were not laid out")
        });
        let text = text_slot.borrow().clone().unwrap_or_else(|| {
            panic!("first blockchain blocks click arrow right row {index} text was not laid out")
        });
        visible.push(text.trim().to_string());
        heights.push(bounds.size.height.as_f32());
    }
    (visible.join("\n"), heights)
}

/// Icon labels for one stage. A hidden icon contributes no label.
fn visible_icon_lines(icons: splora_frontend_shared::BlockNavIcons) -> Vec<String> {
    let mut lines = Vec::new();
    if icons.next_block_icon_visible {
        lines.push("Next Block".to_string());
    }
    if icons.previous_block_icon_visible {
        lines.push("Previous Block".to_string());
    }
    lines
}

/// Paint the newest confirmed block click, then the Previous Block click.
///
/// Opens one headless window, draws a row for each visible icon label and
/// for the route, and quits before returning.
pub fn paint_loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right()
-> LoadsFirstBlockchainBlocksClickArrowRightPaint {
    let page =
        splora_frontend_shared::loads_first_blockchain_blocks_visible_and_click_on_the_arrow_right(
        );
    let height = page.height;
    let route = page.route.clone();
    let newest_lines = visible_icon_lines(page.after_selecting_newest);
    let mut previous_lines = visible_icon_lines(page.after_previous_block_click);
    previous_lines.push(route.clone());
    let newest_count = newest_lines.len();
    let mut lines = newest_lines;
    lines.extend(previous_lines);
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
            cx.new(|_| FirstBlockchainBlocksClickArrowRightView {
                lines: lines_view,
                texts: texts_view,
                slots: slots_view,
            })
        })
        .expect("headless first blockchain blocks click arrow right window");
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
    });
    let (visible_text, row_heights) = read_laid_out(&texts, &slots);
    let laid: Vec<&str> = visible_text.lines().collect();
    let after_selecting_newest = laid[..newest_count].join("\n");
    let after_previous_block_click = laid[newest_count..].join("\n");
    LoadsFirstBlockchainBlocksClickArrowRightPaint {
        after_selecting_newest,
        after_previous_block_click,
        height,
        route,
        row_heights,
    }
}
