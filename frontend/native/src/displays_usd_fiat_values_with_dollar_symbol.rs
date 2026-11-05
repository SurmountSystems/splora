//! Transaction-details and transactions-list USD fiat cells.
//!
//! Both strings come from
//! `splora_frontend_shared::displays_usd_fiat_values_with_dollar_symbol`.
//! This module does not format the amounts. The window is opened headless
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

struct DisplaysUsdFiatValuesWithDollarSymbolView {
    lines: [String; 2],
    texts: [Rc<RefCell<Option<String>>>; 2],
    slots: [Rc<RefCell<Option<Bounds<Pixels>>>>; 2],
}

impl Render for DisplaysUsdFiatValuesWithDollarSymbolView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("displays-usd-fiat-values-with-dollar-symbol")
            .w(px(WINDOW_WIDTH))
            .flex()
            .flex_col()
            .child(text_row(
                0,
                "transaction-details",
                self.lines[0].clone(),
                Rc::clone(&self.texts[0]),
                Rc::clone(&self.slots[0]),
            ))
            .child(text_row(
                1,
                "transactions-list",
                self.lines[1].clone(),
                Rc::clone(&self.texts[1]),
                Rc::clone(&self.slots[1]),
            ))
    }
}

fn text_row(
    index: usize,
    row_id: &'static str,
    line: String,
    text_slot: Rc<RefCell<Option<String>>>,
    bounds_slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
) -> impl IntoElement {
    let laid = line.clone();
    div()
        .id((row_id, index))
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
            *text_slot.borrow_mut() = Some(line);
        },
        |_bounds, _recorded, _window, _app| {},
    )
    .absolute()
    .inset_0()
}

fn read_laid_out(
    texts: &[Rc<RefCell<Option<String>>>; 2],
    slots: &[Rc<RefCell<Option<Bounds<Pixels>>>>; 2],
) -> [String; 2] {
    let mut painted = [String::new(), String::new()];
    for index in 0..2 {
        let bounds = slots[index].borrow().unwrap_or_else(|| {
            panic!("USD fiat dollar-symbol row {index} bounds were not laid out")
        });
        assert!(
            bounds.size.height.as_f32() > 0.0,
            "USD fiat dollar-symbol row {index} height must be positive, got {bounds:?}"
        );
        painted[index] = texts[index]
            .borrow()
            .clone()
            .unwrap_or_else(|| panic!("USD fiat dollar-symbol row {index} text was not laid out"));
    }
    painted
}

/// Paint the transaction-details fiat cell, then the transactions-list cell.
///
/// Opens one headless window with `show: false`, draws the two shared USD
/// strings, and calls `cx.quit()` before returning. Formatting stays in
/// `splora_frontend_shared::displays_usd_fiat_values_with_dollar_symbol`.
pub fn paint_displays_usd_fiat_values_with_dollar_symbol() -> [String; 2] {
    let cells = splora_frontend_shared::displays_usd_fiat_values_with_dollar_symbol();
    let lines = cells.clone();
    let texts = [Rc::new(RefCell::new(None)), Rc::new(RefCell::new(None))];
    let slots = [Rc::new(RefCell::new(None)), Rc::new(RefCell::new(None))];
    let texts_view = texts.clone();
    let slots_view = slots.clone();
    let window_height = 2.0 * ROW_HEIGHT + 80.0;
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
            cx.new(|_| DisplaysUsdFiatValuesWithDollarSymbolView {
                lines,
                texts: texts_view,
                slots: slots_view,
            })
        })
        .expect("headless USD fiat dollar-symbol window");
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
    });
    let painted = read_laid_out(&texts, &slots);
    assert_eq!(
        painted, cells,
        "laid-out USD fiat cells must match the shared dollar-symbol strings"
    );
    painted
}
