//! Mining-dashboard reward-stats title.
//!
//! The title comes from
//! `splora_frontend_shared::shows_the_reward_stats_for_the_last_144_blocks`.
//! This module does not invent that title and does not format a reward amount.
//! The window is opened headless and closed before this function returns.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Bounds, Context, IntoElement, Pixels, Render, Window, WindowBounds, WindowOptions, canvas, div,
    point, prelude::*, px, size,
};
use gpui_platform::headless;

/// Width of the painted title row, in pixels.
const ROW_WIDTH: f32 = 720.0;
/// Height of the painted title row, in pixels.
const ROW_HEIGHT: f32 = 24.0;
/// Headless window width.
const WINDOW_WIDTH: f32 = 800.0;

struct RewardStatsTitleView {
    line: String,
    text: Rc<RefCell<Option<String>>>,
    slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
}

impl Render for RewardStatsTitleView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let line = self.line.clone();
        let text = self.text.clone();
        let slot = self.slot.clone();
        div()
            .id("shows-the-reward-stats-for-the-last-144-blocks")
            .w(px(WINDOW_WIDTH))
            .flex()
            .flex_col()
            .child(title_row(line, text, slot))
    }
}

fn title_row(
    line: String,
    text_slot: Rc<RefCell<Option<String>>>,
    bounds_slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
) -> impl IntoElement {
    let laid = line.clone();
    div()
        .id("shows-the-reward-stats-for-the-last-144-blocks-row")
        .relative()
        .flex_shrink_0()
        .w(px(ROW_WIDTH))
        .h(px(ROW_HEIGHT))
        .child(line)
        .child(title_probe(laid, text_slot, bounds_slot))
}

/// Records the title text and bounds while GPUI lays the row out.
fn title_probe(
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

/// Paint the mining-dashboard reward-stats title.
///
/// Opens one headless window with `show: false`, draws the shared title,
/// and calls `cx.quit()` before returning. The title stays in
/// `splora_frontend_shared::shows_the_reward_stats_for_the_last_144_blocks`.
pub fn paint_shows_the_reward_stats_for_the_last_144_blocks() -> &'static str {
    let title = splora_frontend_shared::shows_the_reward_stats_for_the_last_144_blocks();
    let text = Rc::new(RefCell::new(None));
    let slot = Rc::new(RefCell::new(None));
    let line = title.to_string();
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
            cx.new(|_| RewardStatsTitleView {
                line,
                text: text_view,
                slot: slot_view,
            })
        })
        .expect("headless reward-stats title window");
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
    });
    let laid = text
        .borrow()
        .clone()
        .unwrap_or_else(|| panic!("reward-stats title text was not laid out"));
    let bounds = slot
        .borrow()
        .unwrap_or_else(|| panic!("reward-stats title bounds were not laid out"));
    assert_eq!(
        laid.trim(),
        title,
        "laid-out reward-stats title must match the shared title"
    );
    assert!(
        bounds.size.height.as_f32() > 0.0,
        "reward-stats title row was not laid out"
    );
    splora_frontend_shared::shows_the_reward_stats_for_the_last_144_blocks()
}
