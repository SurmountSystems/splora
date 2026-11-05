//! Static host attribute for the pools pie chart element.
//!
//! Cypress only checks that the host exists. The attribute value is
//! `pool-distribution`. The function is not defined in this file.
//! This test does not invent a pool name, a share, a hashrate,
//! or a block count.

#[test]
fn shows_the_pools_pie_chart() {
    let host = splora_frontend_shared::shows_the_pools_pie_chart();
    assert_eq!(host, "pool-distribution");
}
