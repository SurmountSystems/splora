//! Static mining-dashboard host attribute for the pools pie chart element.
//!
//! Cypress only checks that the host exists. The attribute value is
//! `pool-distribution`. The paint function is not defined in this file.
//! This test does not check a pool name, a share, a hashrate, or a block count.

#[test]
fn shows_the_pools_pie_chart() {
    let host = splora_native::paint_shows_the_pools_pie_chart();
    let host = host.trim();
    assert!(
        host.contains("pool-distribution"),
        "static host attribute, got {host}"
    );
}
