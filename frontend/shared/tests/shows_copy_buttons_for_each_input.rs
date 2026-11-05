//! Cypress it() title: shows copy buttons for each input.
//!
//! Spec: ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts line 52.
//! The it() counts `app-clipboard` and checks each one is visible. The
//! calculator template places one clipboard after each amount input.
//! This does not read a price feed, an indexer body, or a websocket body.

#[test]
fn shows_copy_buttons_for_each_input() {
    let shown = splora_frontend_shared::shows_copy_buttons_for_each_input();
    assert_eq!(
        shown.buttons.len(),
        shown.inputs.len(),
        "cy.get('app-clipboard') has one control for each calculator input"
    );
    assert!(
        !shown.inputs.is_empty(),
        "the calculator form has an input for each clipboard"
    );
    assert!(
        shown.buttons.iter().all(|button| button.visible),
        "cy.get('app-clipboard').each is visible"
    );
    assert!(
        shown
            .buttons
            .iter()
            .zip(shown.inputs.iter())
            .all(|(button, input)| button.field == *input),
        "each app-clipboard copies that input"
    );
    assert_eq!(
        shown.inputs.as_slice(),
        splora_frontend_shared::calculator_field_names().as_slice(),
        "the clipboard inputs are the calculator amount fields"
    );
}
