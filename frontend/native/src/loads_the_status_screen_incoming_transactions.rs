//! Status-screen incoming transactions label.
//!
//! The label comes from the shared host function.
//! This module does not invent that label, a rate, or a mempool size.
//! The window is opened headless and closed before this function returns.

use gpui::{Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
use gpui_platform::headless;

struct IncomingTransactionsView;

impl Render for IncomingTransactionsView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("it-host")
    }
}

/// Paint the status-screen incoming transactions label.
///
/// Opens one headless window with `show: false` and calls `cx.quit()`
/// before returning, including when the window fails to open.
/// The label stays in the shared host function.
pub fn paint_loads_the_status_screen_incoming_transactions() -> &'static str {
    headless().run(|cx| {
        let options = WindowOptions {
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, |_window, cx| cx.new(|_| IncomingTransactionsView));
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless incoming transactions window");
    });
    splora_frontend_shared::loads_the_status_screen_incoming_transactions()
}
