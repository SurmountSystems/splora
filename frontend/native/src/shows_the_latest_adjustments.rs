//! Mining-dashboard latest adjustments host attribute.
//!
//! The attribute comes from the shared host function.
//! This module does not invent that attribute.
//! The window is opened headless and closed before this function returns.

use gpui::{Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
use gpui_platform::headless;

struct LatestHostView;

impl Render for LatestHostView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("lh-host")
    }
}

/// Paint the mining-dashboard latest adjustments host attribute.
///
/// Opens one headless window with `show: false` and calls `cx.quit()`
/// before returning, including when the window fails to open.
/// The attribute stays in the shared host function.
pub fn paint_shows_the_latest_adjustments() -> &'static str {
    headless().run(|cx| {
        let options = WindowOptions {
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, |_window, cx| cx.new(|_| LatestHostView));
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless latest adjustments window");
    });
    splora_frontend_shared::shows_the_latest_adjustments()
}
