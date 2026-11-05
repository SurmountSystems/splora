//! Unblinded-transaction invalid-blinding sentence.
//!
//! The sentence comes from the shared host function.
//! This module does not invent that sentence.
//! The window is opened headless and closed before this function returns.

use gpui::{Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
use gpui_platform::headless;

struct InvalidBlindingView;

impl Render for InvalidBlindingView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("invalid-blinding-host")
    }
}

/// Paint the unblinded-transaction invalid-blinding sentence.
///
/// Opens one headless window with `show: false` and calls `cx.quit()`
/// before returning, including when the window fails to open.
/// The sentence stays in the shared host function.
pub fn paint_show_invalid_error_unblinded_tx() -> &'static str {
    headless().run(|cx| {
        let options = WindowOptions {
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, |_window, cx| cx.new(|_| InvalidBlindingView));
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless invalid blinding window");
    });
    splora_frontend_shared::show_invalid_error_unblinded_tx()
}
