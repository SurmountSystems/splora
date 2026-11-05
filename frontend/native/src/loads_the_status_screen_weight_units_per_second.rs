//! Status-screen weight-units-per-second unit.
//!
//! The unit comes from the shared host function.
//! This module does not invent that unit, a rate, or a size.
//! The window is opened headless and closed before this function returns.

use gpui::{Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};
use gpui_platform::headless;

struct WeightUnitsPerSecondView;

impl Render for WeightUnitsPerSecondView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("weight-units-per-second-host")
    }
}

/// Paint the status-screen weight-units-per-second unit.
///
/// Opens one headless window with `show: false` and calls `cx.quit()`
/// before returning, including when the window fails to open.
/// The unit stays in the shared host function.
pub fn paint_loads_the_status_screen_weight_units_per_second() -> &'static str {
    headless().run(|cx| {
        let options = WindowOptions {
            show: false,
            focus: false,
            ..Default::default()
        };
        let opened = cx.open_window(options, |_window, cx| cx.new(|_| WeightUnitsPerSecondView));
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
        opened.expect("headless weight units per second window");
    });
    splora_frontend_shared::loads_the_status_screen_weight_units_per_second()
}
