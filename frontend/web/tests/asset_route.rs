//! Cypress opens `/assets/asset/{id}` (`liquid.spec.ts`).
//! That path selects the asset page. The short path `/asset/{id}` does too.
//! The page markup is the same string `asset_markup` paints, including the heading.

use splora_web::asset_page_markup;

const NATIVE: &str = "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
const ISSUED: &str = "ac3e0ff248c5051ffd61e00155b7122e5ebc04fd397a0ecbdd4f4e4a56232926";

fn asset_json(id: &str, name: &str, ticker: &str) -> String {
    format!(r#"{{"asset_id":"{id}","name":"{name}","ticker":"{ticker}"}}"#)
}

#[test]
fn assets_asset_path_selects_the_asset_page_and_its_heading() {
    let cypress = format!("/assets/asset/{NATIVE}");
    let markup = asset_page_markup(
        &cypress,
        &asset_json(NATIVE, "Liquid Bitcoin", "L-BTC"),
        "[]",
    )
    .expect("Cypress /assets/asset/{{id}} selects the asset page");
    assert!(
        markup.contains("<h2>Peg In/Out and Burn Transactions</h2>"),
        "{markup}"
    );
    assert_eq!(markup.matches("<h2>").count(), 1, "{markup}");
    assert!(markup.contains("Liquid Bitcoin"), "{markup}");
    assert!(markup.contains("L-BTC"), "{markup}");

    let short = format!("/asset/{NATIVE}");
    let short_markup =
        asset_page_markup(&short, &asset_json(NATIVE, "Liquid Bitcoin", "L-BTC"), "[]")
            .expect("short /asset/{{id}} selects the asset page");
    assert!(
        short_markup.contains("<h2>Peg In/Out and Burn Transactions</h2>"),
        "{short_markup}"
    );
    assert_eq!(short_markup.matches("<h2>").count(), 1, "{short_markup}");

    let issued = format!("/assets/asset/{ISSUED}");
    let issued_markup = asset_page_markup(&issued, &asset_json(ISSUED, "Test Asset", "TEST"), "[]")
        .expect("issued asset path selects the asset page");
    assert!(
        issued_markup.contains("<h2>Issuance and Burn Transactions</h2>"),
        "{issued_markup}"
    );
    assert!(
        !issued_markup.contains("Peg In/Out and Burn Transactions"),
        "{issued_markup}"
    );

    assert!(
        asset_page_markup(
            "/assets/registry",
            &asset_json(NATIVE, "Liquid Bitcoin", "L-BTC"),
            "[]",
        )
        .is_none(),
        "registry path selected the asset page"
    );
    assert!(
        asset_page_markup(&cypress, &asset_json(ISSUED, "Test Asset", "TEST"), "[]",).is_none(),
        "a path selected a different asset"
    );
}
