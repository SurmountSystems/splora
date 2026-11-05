//! Calculator `.symbol` text after USD is selected.
//!
//! The symbol comes from
//! `splora_frontend_shared::displays_usd_price_with_dollar_symbol`.
//! This module does not invent that symbol and does not format a price.
//! The window is opened headless and closed before this function returns.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Bounds, Context, IntoElement, Pixels, Render, Window, WindowBounds, WindowOptions, canvas, div,
    point, prelude::*, px, size,
};
use gpui_platform::headless;

/// Width of the painted symbol row, in pixels.
const ROW_WIDTH: f32 = 720.0;
/// Height of the painted symbol row, in pixels.
const ROW_HEIGHT: f32 = 24.0;
/// Headless window width.
const WINDOW_WIDTH: f32 = 800.0;

struct UsdPriceSymbolView {
    line: String,
    text: Rc<RefCell<Option<String>>>,
    slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
}

impl Render for UsdPriceSymbolView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let line = self.line.clone();
        let text = self.text.clone();
        let slot = self.slot.clone();
        div()
            .id("displays-usd-price-with-dollar-symbol")
            .w(px(WINDOW_WIDTH))
            .flex()
            .flex_col()
            .child(symbol_row(line, text, slot))
    }
}

fn symbol_row(
    line: String,
    text_slot: Rc<RefCell<Option<String>>>,
    bounds_slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
) -> impl IntoElement {
    let laid = line.clone();
    div()
        .id("displays-usd-price-with-dollar-symbol-row")
        .relative()
        .flex_shrink_0()
        .w(px(ROW_WIDTH))
        .h(px(ROW_HEIGHT))
        .child(line)
        .child(symbol_probe(laid, text_slot, bounds_slot))
}

/// Records the symbol text and bounds while GPUI lays the row out.
fn symbol_probe(
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

/// Paint the calculator USD symbol.
///
/// Opens one headless window with `show: false`, draws the shared symbol,
/// and calls `cx.quit()` before returning. Formatting stays in
/// `splora_frontend_shared::displays_usd_price_with_dollar_symbol`.
pub fn paint_displays_usd_price_with_dollar_symbol() -> String {
    let symbol = splora_frontend_shared::displays_usd_price_with_dollar_symbol();
    let text = Rc::new(RefCell::new(None));
    let slot = Rc::new(RefCell::new(None));
    let line = symbol.clone();
    let text_view = text.clone();
    let slot_view = slot.clone();
    headless().run(move |cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.), px(0.)),
                size: size(px(WINDOW_WIDTH), px(ROW_HEIGHT + 80.0)),
            })),
            show: false,
            focus: false,
            ..Default::default()
        };
        cx.open_window(options, move |_window, cx| {
            cx.new(|_| UsdPriceSymbolView {
                line,
                text: text_view,
                slot: slot_view,
            })
        })
        .expect("headless USD price symbol window");
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
    });
    let laid = text
        .borrow()
        .clone()
        .unwrap_or_else(|| panic!("calculator USD symbol text was not laid out"));
    let bounds = slot
        .borrow()
        .unwrap_or_else(|| panic!("calculator USD symbol bounds were not laid out"));
    assert_eq!(
        laid.trim(),
        symbol,
        "laid-out calculator symbol must match the shared symbol"
    );
    assert!(
        bounds.size.height.as_f32() > 0.0,
        "calculator USD symbol row was not laid out"
    );
    symbol
}
