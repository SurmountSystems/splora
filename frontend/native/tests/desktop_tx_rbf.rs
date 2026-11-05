//! Desktop transaction page replace-by-fee width and overlap.
//!
//! Cypress title: `shows RBF transactions properly (desktop)`.
//! Source: `ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts`.
//! Viewport name `macbook-16` is `cy.viewport('macbook-16')` in that test.
//! Viewport pixels were not found in the vendored Cypress config, spec, or
//! `node_modules` (that directory is not present).
//!
//! The test paints through `measure_desktop_transaction_rbf_bounds` and reads
//! laid-out bounds. It does not leave a window open.

/// Cypress `areOverlapping`: touching edges count as overlap (`<`, not `<=`).
fn rects_overlap(first: &splora_native::LaidOutRect, second: &splora_native::LaidOutRect) -> bool {
    let first_right = first.x + first.width;
    let first_bottom = first.y + first.height;
    let second_right = second.x + second.width;
    let second_bottom = second.y + second.height;
    if first_right < second.x || second_right < first.x {
        return false;
    }
    if first_bottom < second.y || second_bottom < first.y {
        return false;
    }
    true
}

#[test]
fn shows_rbf_transactions_properly_desktop() {
    // Visit `/tx/21518a98d1aa9df524865d2f88c578499f524eb1d0c4d3e70312ab863508692f`,
    // then the `rbfTransaction` socket (`mainnet_rbf.json` `txReplaced.txid`).
    let laid = splora_native::measure_desktop_transaction_rbf_bounds();
    let bounds_line = format!(
        "alert x={} y={} w={} h={} details x={} y={} w={} h={} btn-warning x={} y={} w={} h={}",
        laid.alert.x,
        laid.alert.y,
        laid.alert.width,
        laid.alert.height,
        laid.details.x,
        laid.details.y,
        laid.details.width,
        laid.details.height,
        laid.replaced_button.x,
        laid.replaced_button.y,
        laid.replaced_button.width,
        laid.replaced_button.height
    );
    eprintln!("{bounds_line}");
    let _ = std::fs::write(
        "/tmp/grok-goal-c3cba9323e99/implementer/native-desktop-rbf-bounds.txt",
        bounds_line,
    );

    // `.container-xl > :nth-child(3)` css width equals `.alert` css width.
    assert_eq!(
        laid.alert.width, laid.details.width,
        "alert css width and the third container-xl child css width differ: alert {} details {}",
        laid.alert.width, laid.details.width
    );
    assert!(
        laid.alert.width > 0.0 && laid.details.width > 0.0,
        "measured widths must come from layout, not a zero placeholder"
    );
    assert!(
        laid.alert.height > 0.0
            && laid.replaced_button.width > 0.0
            && laid.replaced_button.height > 0.0,
        "alert and .btn-warning must have laid-out size"
    );

    // `.btn-warning` getBoundingClientRect vs `.alert` getBoundingClientRect.
    assert!(
        !rects_overlap(&laid.replaced_button, &laid.alert),
        "Confirmations box and RBF alert are overlapping"
    );
}
