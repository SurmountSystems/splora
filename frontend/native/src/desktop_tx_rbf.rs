//! Desktop transaction replace-by-fee layout.
//!
//! Cypress `shows RBF transactions properly (desktop)` compares the css width
//! of `.alert` with `.container-xl > :nth-child(3)`, and the bounding rects of
//! `.btn-warning` and `.alert`. Viewport name is `macbook-16`. Pixel size of
//! that Cypress preset is not in this tree.
//!
//! Container width is the `max-width: 1140px` rule in
//! `ref/mempool/frontend/src/styles.scss` under `@media (min-width: 1400px)`.
//! Horizontal padding is `15px` on `.container-xl` in that file.
//! The window is opened headless and closed before this function returns.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Bounds, Context, IntoElement, Pixels, Render, Window, WindowBounds, WindowOptions, canvas, div,
    point, prelude::*, px, rems, size,
};
use gpui_platform::headless;

/// Visited tx in the Cypress test.
const VISITED_TXID: &str = "21518a98d1aa9df524865d2f88c578499f524eb1d0c4d3e70312ab863508692f";
/// `txReplaced.txid` in `cypress/fixtures/mainnet_rbf.json`, sent as `rbfTransaction`.
const REPLACEMENT_TXID: &str = "8913ec7ba0ede285dbd120e46f6d61a28f2903c10814a6f6c4f97d0edf3e1f46";

/// `styles.scss` `.container-xl` max-width when the viewport is at least 1400px.
const CONTAINER_MAX_WIDTH: f32 = 1140.0;
/// Same file: `.container-xl` padding-left and padding-right.
const CONTAINER_PADDING: f32 = 15.0;
/// Media query that turns that max-width on. Not the macbook-16 preset.
const DESKTOP_MEDIA_MIN_WIDTH: f32 = 1400.0;

/// One laid-out rectangle. Origin is the top left, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaidOutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Bounds read from the desktop transaction replace-by-fee paint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopTxRbfBounds {
    /// `.alert`
    pub alert: LaidOutRect,
    /// `.container-xl > :nth-child(3)` (transaction details)
    pub details: LaidOutRect,
    /// `.btn-warning` ("Replaced")
    pub replaced_button: LaidOutRect,
}

struct DesktopTxRbfView {
    alert: Rc<RefCell<Option<Bounds<Pixels>>>>,
    details: Rc<RefCell<Option<Bounds<Pixels>>>>,
    replaced_button: Rc<RefCell<Option<Bounds<Pixels>>>>,
}

impl Render for DesktopTxRbfView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let alert = self.alert.clone();
        let details = self.details.clone();
        let replaced_button = self.replaced_button.clone();
        div()
            .id("container-xl")
            .w(px(CONTAINER_MAX_WIDTH))
            .flex()
            .flex_col()
            .px(px(CONTAINER_PADDING))
            .child(title_block(alert, replaced_button))
            .child(div().id("clearfix").w_full().h(px(0.)))
            .child(full_width_region(
                "transaction-details",
                "Details",
                details,
                false,
            ))
    }
}

fn title_block(
    alert: Rc<RefCell<Option<Bounds<Pixels>>>>,
    replaced_button: Rc<RefCell<Option<Bounds<Pixels>>>>,
) -> impl IntoElement {
    // `.title-block` is a wrapping row from 650px. `.alert-mempool` is
    // `width: 100%`, so the alert is its own line above the Replaced button.
    div()
        .id("title-block")
        .w_full()
        .flex()
        .flex_col()
        .child(full_width_region(
            "alert",
            format!("This transaction has been replaced by: {REPLACEMENT_TXID}"),
            alert,
            true,
        ))
        .child(
            div()
                .id("title-row")
                .w_full()
                .flex()
                .flex_row()
                .justify_between()
                .items_center()
                .child("Transaction")
                .child(div().flex_grow_1().child(VISITED_TXID))
                .child(
                    div()
                        .id("btn-warning")
                        .relative()
                        .px_2()
                        .py_1()
                        .child("Replaced")
                        .child(bounds_probe(replaced_button)),
                ),
        )
}

fn full_width_region(
    id: &'static str,
    text: impl Into<String>,
    slot: Rc<RefCell<Option<Bounds<Pixels>>>>,
    alert_margin: bool,
) -> impl IntoElement {
    let mut region = div().id(id).relative().w_full().child(text.into());
    if alert_margin {
        // Bootstrap ~5.3.8 `.alert` uses margin-bottom 1rem. That scss is not
        // vendored here. Cypress treats a shared edge as overlap.
        region = region.mb(rems(1.));
    }
    region.child(bounds_probe(slot))
}

fn bounds_probe(slot: Rc<RefCell<Option<Bounds<Pixels>>>>) -> impl IntoElement {
    canvas(
        move |bounds, _window, _app| {
            *slot.borrow_mut() = Some(bounds);
        },
        |_bounds, _recorded, _window, _app| {},
    )
    .absolute()
    .inset_0()
}

fn rect_from(bounds: Bounds<Pixels>) -> LaidOutRect {
    LaidOutRect {
        x: bounds.origin.x.as_f32(),
        y: bounds.origin.y.as_f32(),
        width: bounds.size.width.as_f32(),
        height: bounds.size.height.as_f32(),
    }
}

/// Paint the desktop transaction replace-by-fee regions and return laid-out bounds.
///
/// Opens one headless window, draws once, and quits before returning.
pub fn measure_desktop_transaction_rbf_bounds() -> DesktopTxRbfBounds {
    let alert = Rc::new(RefCell::new(None));
    let details = Rc::new(RefCell::new(None));
    let replaced_button = Rc::new(RefCell::new(None));
    let alert_view = alert.clone();
    let details_view = details.clone();
    let button_view = replaced_button.clone();
    headless().run(move |cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.), px(0.)),
                size: size(px(DESKTOP_MEDIA_MIN_WIDTH), px(800.)),
            })),
            show: false,
            focus: false,
            ..Default::default()
        };
        cx.open_window(options, move |_window, cx| {
            cx.new(|_| DesktopTxRbfView {
                alert: alert_view,
                details: details_view,
                replaced_button: button_view,
            })
        })
        .expect("headless desktop replace-by-fee window");
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
    });
    DesktopTxRbfBounds {
        alert: rect_from(alert.borrow().expect("alert bounds were not laid out")),
        details: rect_from(details.borrow().expect("details bounds were not laid out")),
        replaced_button: rect_from(
            replaced_button
                .borrow()
                .expect("replaced button bounds were not laid out"),
        ),
    }
}
