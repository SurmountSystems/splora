//! Next Block and Previous Block labels and route after ArrowLeft on genesis.
//!
//! Labels and the route come from
//! `splora_frontend_shared::loads_genesis_block_and_keypress_arrow_left`.
//! Each icon label is included only when that return says it is visible.
//! This return has no heading. This module does not recompute the link
//! rule. The window is opened headless and closed before this function
//! returns.

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

/// Laid-out visible labels and route after ArrowLeft on genesis.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadsGenesisBlockArrowLeftPaint {
    /// Shared return this window drew.
    pub shown: splora_frontend_shared::GenesisBlockArrowLeft,
    /// One line per visible label, then the route.
    pub visible_text: String,
    /// Height in pixels of each laid-out row.
    pub row_heights: Vec<f32>,
}

struct GenesisBlockArrowLeftView {
    lines: Vec<String>,
    texts: Vec<Rc<RefCell<Option<String>>>>,
    slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
}

impl Render for GenesisBlockArrowLeftView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = div()
            .id("genesis-block-arrow-left")
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
        .id(("genesis-block-arrow-left-row", index))
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
            panic!("genesis block arrow left row {index} bounds were not laid out")
        });
        let text = text_slot.borrow().clone().unwrap_or_else(|| {
            panic!("genesis block arrow left row {index} text was not laid out")
        });
        visible.push(text.trim().to_string());
        heights.push(bounds.size.height.as_f32());
    }
    (visible.join("\n"), heights)
}

/// Visible icon labels, then the route. Hidden icons are omitted.
///
/// The integration test formats these lines the same way. This return has
/// no heading, so none is laid out.
fn icon_lines(shown: &splora_frontend_shared::GenesisBlockArrowLeft) -> Vec<String> {
    let icons = &shown.after_arrow_left;
    let mut lines = Vec::new();
    if icons.next_block_icon_visible {
        lines.push("Next Block".to_string());
    }
    if icons.previous_block_icon_visible {
        lines.push("Previous Block".to_string());
    }
    lines.push(format!("route {}", shown.route));
    lines
}

/// Paint the visible genesis labels and route after one ArrowLeft.
///
/// Opens one headless window, draws one row per shared visible label and
/// the route, and quits before returning.
pub fn paint_loads_genesis_block_and_keypress_arrow_left() -> LoadsGenesisBlockArrowLeftPaint {
    let shown = splora_frontend_shared::loads_genesis_block_and_keypress_arrow_left();
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
            cx.new(|_| GenesisBlockArrowLeftView {
                lines: lines_view,
                texts: texts_view,
                slots: slots_view,
            })
        })
        .expect("headless genesis block arrow left window");
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
    });
    let (visible_text, row_heights) = read_laid_out(&texts, &slots);
    LoadsGenesisBlockArrowLeftPaint {
        shown,
        visible_text,
        row_heights,
    }
}
