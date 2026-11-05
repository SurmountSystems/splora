//! Empty graphs child redirect label.
//!
//! The label comes from the shared host function.
//! This module does not invent that label.
//! The window is opened headless and closed before this function returns.

use gpui::{Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
use gpui_platform::headless;

struct GraphsEmptyChildRedirectView;

impl Render for GraphsEmptyChildRedirectView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("graphs-empty-child-redirect-host")
    }
}

/// Paint the empty graphs child redirect label.
///
/// Opens one headless window with `show: false` and calls `cx.quit()`
/// before returning, including when the window fails to open.
/// The label stays in the shared host function.
pub fn paint_graphs_empty_child_redirects_to_mempool() -> &'static str {
    headless().run(|cx| {
        let options = WindowOptions {
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, |_window, cx| {
            cx.new(|_| GraphsEmptyChildRedirectView)
        });
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless graphs empty child redirect window");
    });
    splora_frontend_shared::graphs_empty_child_redirects_to_mempool()
}
