//! Cypress it() title: loads the api screen.
//!
//! Mainnet `ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts`
//! visits `/`, waits until the skeleton is gone, clicks `#btn-docs`,
//! and waits 1000 milliseconds. The shared function takes that control
//! and the docs child routes `(path, redirectTo)`. The destination is
//! the empty-path redirect. A different control does not get it.
//! `paint_loads_the_api_screen` is not defined for that input on
//! `splora_native`. This test does not open a window.

const CYPRESS_IT: &str = "loads the api screen";

fn line_value<'a>(painted: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}=");
    painted.lines().find_map(|line| line.strip_prefix(&prefix))
}

fn paint<'a>(clicked_control: &str, docs_routes: &'a [(&'a str, &'a str)]) -> String {
    splora_native::paint_loads_the_api_screen(clicked_control, docs_routes)
}

fn shared<'a>(
    clicked_control: &str,
    docs_routes: &'a [(&'a str, &'a str)],
) -> splora_frontend_shared::ApiScreen<'a> {
    splora_frontend_shared::after_docs_button_loads_the_api_screen(clicked_control, docs_routes)
}

#[test]
fn loads_the_api_screen() {
    let docs_button = "#btn-docs";
    let bitcoin = [("api", "api/rest"), ("", "faq"), ("**", "faq")];
    let view = shared(docs_button, &bitcoin);
    let painted = paint(docs_button, &bitcoin);

    assert!(
        view.api_screen_active,
        "{CYPRESS_IT}: clicking #btn-docs activates the api screen"
    );
    assert_eq!(
        view.destination, "faq",
        "{CYPRESS_IT}: the shared destination is the empty-path redirect target"
    );
    assert_eq!(
        line_value(&painted, "control"),
        Some(docs_button),
        "{CYPRESS_IT}: painted control must be #btn-docs; got {painted:?}"
    );
    assert_eq!(
        line_value(&painted, "api_screen_active"),
        Some("true"),
        "{CYPRESS_IT}: painted api_screen_active must equal the shared flag; got {painted:?}"
    );
    assert_eq!(
        line_value(&painted, "destination"),
        Some(view.destination),
        "{CYPRESS_IT}: painted lines must equal the shared destination {:?}, including the redirect target; got {painted:?}",
        view.destination
    );

    let other_button = "#btn-about";
    let other = shared(other_button, &bitcoin);
    let other_painted = paint(other_button, &bitcoin);
    assert!(
        !other.api_screen_active && other.destination.is_empty(),
        "{CYPRESS_IT}: a different control does not activate the api screen; got {other:?}"
    );
    assert_eq!(
        line_value(&other_painted, "control"),
        Some(other_button),
        "{CYPRESS_IT}: a different control stays on its own control line; got {other_painted:?}"
    );
    assert_eq!(
        line_value(&other_painted, "api_screen_active"),
        Some("false"),
        "{CYPRESS_IT}: a different control paints api_screen_active=false; got {other_painted:?}"
    );
    assert_eq!(
        line_value(&other_painted, "destination"),
        Some(other.destination),
        "{CYPRESS_IT}: a different control must not paint destination {:?}; got {other_painted:?}",
        view.destination
    );
    assert!(
        line_value(&other_painted, "destination") != Some(view.destination),
        "{CYPRESS_IT}: a different control must not paint the docs redirect {:?}; got {other_painted:?}",
        view.destination
    );

    let docs_id = "btn-docs";
    let by_id = shared(docs_id, &bitcoin);
    let painted_id = paint(docs_id, &bitcoin);
    assert!(by_id.api_screen_active);
    assert_eq!(by_id.destination, view.destination);
    assert_eq!(
        line_value(&painted_id, "control"),
        Some(docs_id),
        "{CYPRESS_IT}: the docs id without a hash is the clicked control; got {painted_id:?}"
    );
    assert_eq!(
        line_value(&painted_id, "destination"),
        Some(by_id.destination),
        "{CYPRESS_IT}: btn-docs paints the same shared destination; got {painted_id:?}"
    );
    assert_eq!(
        line_value(&painted_id, "api_screen_active"),
        Some("true"),
        "{CYPRESS_IT}: btn-docs paints api_screen_active=true; got {painted_id:?}"
    );

    let liquid = [("api", "api/rest"), ("**", "api/rest"), ("", "api/rest")];
    let liquid_view = shared(docs_button, &liquid);
    let liquid_painted = paint(docs_button, &liquid);
    assert!(liquid_view.api_screen_active);
    assert_ne!(
        liquid_view.destination, view.destination,
        "{CYPRESS_IT}: the liquid empty-path redirect is not the bitcoin one"
    );
    assert_eq!(
        liquid_view.destination, "api/rest",
        "{CYPRESS_IT}: the liquid shared destination is that table's empty-path redirect"
    );
    assert_eq!(
        line_value(&liquid_painted, "destination"),
        Some(liquid_view.destination),
        "{CYPRESS_IT}: painted destination must follow the shared route table, not a fixed path; got {liquid_painted:?}"
    );
    assert!(
        line_value(&liquid_painted, "destination") != Some(view.destination),
        "{CYPRESS_IT}: the liquid paint must not use the bitcoin redirect {:?}; got {liquid_painted:?}",
        view.destination
    );

    let no_empty = [("api", "api/rest"), ("**", "faq")];
    let missing = shared(docs_button, &no_empty);
    let missing_painted = paint(docs_button, &no_empty);
    assert!(missing.api_screen_active);
    assert_eq!(missing.destination, "");
    assert_eq!(
        line_value(&missing_painted, "destination"),
        Some(""),
        "{CYPRESS_IT}: no empty path paints an empty destination, not another row's redirect; got {missing_painted:?}"
    );
    assert!(
        !missing_painted.contains("destination=faq")
            && !missing_painted.contains("destination=api/rest"),
        "{CYPRESS_IT}: a missing empty path must not paint another row's redirect; got {missing_painted:?}"
    );
}
