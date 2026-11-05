//! Desktop block pagination.
//!
//! Cypress `shows blocks pagination with 5 pages (desktop)` visits at
//! viewport 760 by 800, then 669 by 800. The child count is
//! `splora_frontend_shared::shows_blocks_pagination_with_5_pages_desktop`.
//! This module does not invent that count. The window is opened headless
//! and closed before this function returns.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Bounds, Context, IntoElement, Pixels, Render, Window, WindowBounds, WindowOptions, canvas, div,
    point, prelude::*, px, size,
};
use gpui_platform::headless;

/// Width and height of each laid-out first-pager child, in pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct ShowsBlocksPaginationWith5PagesDesktop {
    pub child_widths: Vec<f32>,
    pub child_heights: Vec<f32>,
}

struct BlockPaginationView {
    slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>>,
}

impl Render for BlockPaginationView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let slots = self.slots.clone();
        let mut row = div().id("pagination").w_full().flex().flex_row();
        for (index, slot) in slots.into_iter().enumerate() {
            row = row.child(pagination_child(index, slot));
        }
        row
    }
}

fn pagination_child(index: usize, slot: Rc<RefCell<Option<Bounds<Pixels>>>>) -> impl IntoElement {
    div()
        .id(("pagination-child", index))
        .relative()
        .flex_shrink_0()
        .w(px(32.))
        .h(px(32.))
        .child((index + 1).to_string())
        .child(bounds_probe(slot))
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

/// Paint the first block pager and return each child's laid-out size.
///
/// Opens one headless window at `viewport_width` by 800, draws one child
/// per `first_pagination_children`, and quits before returning.
pub fn paint_shows_blocks_pagination_with_5_pages_desktop(
    viewport_width: u32,
    tx_count: u32,
) -> ShowsBlocksPaginationWith5PagesDesktop {
    let page = splora_frontend_shared::shows_blocks_pagination_with_5_pages_desktop(
        viewport_width,
        tx_count,
    );
    let slots: Vec<Rc<RefCell<Option<Bounds<Pixels>>>>> = (0..page.first_pagination_children)
        .map(|_| Rc::new(RefCell::new(None)))
        .collect();
    let slots_view = slots.clone();
    let width = viewport_width as f32;
    headless().run(move |cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.), px(0.)),
                size: size(px(width), px(800.)),
            })),
            show: false,
            focus: false,
            ..Default::default()
        };
        cx.open_window(options, move |_window, cx| {
            cx.new(|_| BlockPaginationView { slots: slots_view })
        })
        .expect("headless block pagination window");
        cx.quit();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.update(|app| app.quit());
        })
        .detach();
    });
    let mut child_widths = Vec::with_capacity(slots.len());
    let mut child_heights = Vec::with_capacity(slots.len());
    for slot in &slots {
        let bounds = slot
            .borrow()
            .expect("pagination child bounds were not laid out");
        child_widths.push(bounds.size.width.as_f32());
        child_heights.push(bounds.size.height.as_f32());
    }
    ShowsBlocksPaginationWith5PagesDesktop {
        child_widths,
        child_heights,
    }
}
