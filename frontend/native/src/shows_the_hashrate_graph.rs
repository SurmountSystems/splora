//! Mining-dashboard hashrate graph host attribute.
//!
//! The attribute comes from the shared host function.
//! This module does not invent that attribute and does not invent a hashrate.
//! The window is opened headless and closed before this function returns.

use gpui::{Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
use gpui_platform::headless;

struct HashrateGraphView;

impl Render for HashrateGraphView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("hg-host")
    }
}

/// Paint the mining-dashboard hashrate graph host attribute.
///
/// Opens one headless window with `show: false` and calls `cx.quit()`
/// before returning, including when the window fails to open.
/// The attribute stays in the shared host function.
pub fn paint_shows_the_hashrate_graph() -> &'static str {
    headless().run(|cx| {
        let options = WindowOptions {
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, |_window, cx| cx.new(|_| HashrateGraphView));
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless hashrate graph window");
    });
    splora_frontend_shared::shows_the_hashrate_graph()
}
