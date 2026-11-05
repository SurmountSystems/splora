//! Cypress it() title: load first mempool block after skeleton loads.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts.
//! The it() visits the dashboard, waits until `.skeleton-loader` is gone,
//! clicks `#mempool-block-0 > .blockLink`, and waits until the skeleton is
//! gone again. The dashboard does not pass `getHref`, so the click follows
//! `/mempool-block/0`. Liquid `relativeUrl` does not prefix that path.
//! This test calls `paint_load_first_mempool_block_after_skeleton_loads`
//! and reads the laid-out lines and row heights. It does not search source
//! text and it does not open a window.

const CYPRESS_IT: &str = "load first mempool block after skeleton loads";

#[test]
fn load_first_mempool_block_after_skeleton_loads() {
    let painted = splora_native::paint_load_first_mempool_block_after_skeleton_loads();
    let lines: Vec<&str> = painted.visible_text.lines().collect();
    assert_eq!(
        painted.row_heights.len(),
        lines.len(),
        "{CYPRESS_IT}: each laid-out line needs a row height, lines {lines:?} heights {:?}",
        painted.row_heights
    );
    assert!(
        !painted.row_heights.is_empty() && painted.row_heights.iter().all(|height| *height > 0.0),
        "{CYPRESS_IT}: rows were not laid out, heights {:?}",
        painted.row_heights
    );
    assert!(
        lines.len() >= 2 && lines[0] == "mempool-block-0" && lines[1] == "blockLink",
        "{CYPRESS_IT}: cy.get('#mempool-block-0 > .blockLink').click(); got {:?}",
        painted.visible_text
    );
    assert!(
        lines.iter().any(|line| *line == "/mempool-block/0"),
        "{CYPRESS_IT}: dashboard default getHref(0) is /mempool-block/0 with no liquid prefix; got {:?}",
        painted.visible_text
    );
    assert!(
        lines.iter().any(|line| *line == "0"),
        "{CYPRESS_IT}: Cypress.$('.skeleton-loader').length === 0 after the click; got {:?}",
        painted.visible_text
    );
    assert!(
        !painted.visible_text.contains("/liquid/mempool-block/0"),
        "{CYPRESS_IT}: liquid relativeUrl must not prefix /mempool-block/0; got {:?}",
        painted.visible_text
    );
}
