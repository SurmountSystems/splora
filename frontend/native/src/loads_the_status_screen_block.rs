//! Status-screen block label.
//!
//! The label comes from the shared host function.
//! This module does not invent that label, a count, or a size.
//! The window is opened headless and closed before this function returns.

use gpui::{Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
use gpui_platform::headless;

struct BlockView;

impl Render for BlockView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("block-host")
    }
}

/// Paint the status-screen block label.
///
/// Opens one headless window with `show: false` and calls `cx.quit()`
/// before returning, including when the window fails to open.
/// The label stays in the shared host function.
pub fn paint_loads_the_status_screen_block() -> &'static str {
    headless().run(|cx| {
        let options = WindowOptions {
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, |_window, cx| cx.new(|_| BlockView));
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless block window");
    });
    splora_frontend_shared::loads_the_status_screen_block()
}
