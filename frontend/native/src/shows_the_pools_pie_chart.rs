//! Mining-dashboard pools pie chart host attribute.
//!
//! The attribute comes from
//! `splora_frontend_shared::shows_the_pools_pie_chart`.
//! This module does not invent that attribute and does not invent a pool row.
//! It does not format a pool name, a share, a hashrate, or a block count.
//! The window is opened headless and closed before this function returns.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Bounds, Context, IntoElement, Pixels, Render, Window, WindowBounds, WindowOptions, canvas, div,
    point, prelude::*, px, size,
};
use gpui_platform::headless;

/// Width of the painted host row, in pixels.
const ROW_WIDTH: f32 = 720.0;
/// Height of the painted host row, in pixels.
const ROW_HEIGHT: f32 = 24.0;
/// Headless window width.
const WINDOW_WIDTH: f32 = 800.0;

struct PoolsPieChartHostView {
    line: String,
    text: Rc<RefCell<Option<String>>>,
    slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
}

impl Render for PoolsPieChartHostView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let line = self.line.clone();
        let text = self.text.clone();
        let slot = self.slot.clone();
        div()
            .id("pp-host")
            .w(px(WINDOW_WIDTH))
            .flex()
            .flex_col()
            .child(host_row(line, text, slot))
    }
}

fn host_row(
    line: String,
    text_slot: Rc<RefCell<Option<String>>>,
    bounds_slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
) -> impl IntoElement {
    let laid = line.clone();
    div()
        .id("pp-host-row")
        .relative()
        .flex_shrink_0()
        .w(px(ROW_WIDTH))
        .h(px(ROW_HEIGHT))
        .child(line)
        .child(host_probe(laid, text_slot, bounds_slot))
}

/// Records the host text and bounds while GPUI lays the row out.
fn host_probe(
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

/// Paint the mining-dashboard pools pie chart host attribute.
///
/// Opens one headless window with `show: false`, draws the shared attribute,
/// and calls `cx.quit()` before returning, including when the window fails
/// to open. The attribute stays in
/// `splora_frontend_shared::shows_the_pools_pie_chart`.
pub fn paint_shows_the_pools_pie_chart() -> &'static str {
    let host = splora_frontend_shared::shows_the_pools_pie_chart();
    let text = Rc::new(RefCell::new(None));
    let slot = Rc::new(RefCell::new(None));
    let line = host.to_string();
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
        let opened = cx.open_window(options, move |_window, cx| {
            cx.new(|_| PoolsPieChartHostView {
                line,
                text: text_view,
                slot: slot_view,
            })
        });
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless pools pie chart host window");
    });
    let laid = text
        .borrow()
        .clone()
        .unwrap_or_else(|| panic!("pools pie chart host text was not laid out"));
    let bounds = slot
        .borrow()
        .unwrap_or_else(|| panic!("pools pie chart host bounds were not laid out"));
    assert_eq!(
        laid.trim(),
        host,
        "laid-out pools pie chart host must match the shared attribute"
    );
    assert!(
        bounds.size.height.as_f32() > 0.0,
        "pools pie chart host row was not laid out"
    );
    splora_frontend_shared::shows_the_pools_pie_chart()
}
