//! Details open state after a click.
//!
//! The next open state comes from the shared host function.
//! This module does not invent that state.
//! The window is opened headless and closed before this function returns.

use gpui::{Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
use gpui_platform::headless;

struct DetailsOpenAfterClickView;

impl Render for DetailsOpenAfterClickView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("details-open-after-click-host")
    }
}

/// Paint whether details are open after a click.
///
/// Opens one headless window with `show: false` and `focus: false`,
/// and calls `cx.quit()` before the failed-open expect.
/// The next open state stays in the shared host function.
pub fn paint_details_open_after_click(open: bool) -> bool {
    headless().run(|cx| {
        let options = WindowOptions {
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, |_window, cx| cx.new(|_| DetailsOpenAfterClickView));
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless details open after click window");
    });
    splora_frontend_shared::details_open_after_click(open)
}
