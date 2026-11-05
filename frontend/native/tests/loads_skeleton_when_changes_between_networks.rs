//! Cypress it() title: loads skeleton when changes between networks.
//!
//! `splora_frontend_shared::loads_skeleton_when_changes_between_networks`
//! takes `first` then `second` and returns peak, settled, and network.
//! The paint prints those three fields, one per line:
//! `peak=<u32>`, `settled=<u32>`, `network=<name>`.
//! This test does not open a window.

/// Painted text is three exact lines: peak, settled, then network.
fn painted_line_value<'a>(painted: &'a str, key: &str) -> &'a str {
    let prefix = format!("{key}=");
    painted
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or("")
}

#[test]
fn loads_skeleton_when_changes_between_networks() {
    let first = "signet";
    let second = "mainnet";
    let shared =
        splora_frontend_shared::loads_skeleton_when_changes_between_networks(first, second);
    let painted = splora_native::paint_loads_skeleton_when_changes_between_networks(first, second);
    let painted_settled: u32 = painted_line_value(&painted, "settled")
        .parse()
        .expect("painted settled count");
    let painted_peak: u32 = painted_line_value(&painted, "peak")
        .parse()
        .expect("painted peak");
    assert_eq!(
        painted_settled, shared.settled,
        "painted settled count must equal shared.settled, got painted {painted_settled} and shared {shared:?} in {painted:?}"
    );
    assert_eq!(
        painted_settled, 0,
        "settled count is 0 after changeNetwork('{first}') then changeNetwork('{second}'), got {painted_settled} in {painted:?}"
    );
    assert!(
        painted
            .lines()
            .any(|line| line == format!("peak={}", shared.peak)),
        "painted lines must include the shared peak peak={}, got {painted:?} from {shared:?}",
        shared.peak
    );
    assert!(
        shared.peak > 0,
        "peak is greater than 0 after changeNetwork('{first}') then changeNetwork('{second}'), got {} from {shared:?}",
        shared.peak
    );
    assert_eq!(
        painted_peak, shared.peak,
        "painted peak must equal shared.peak, got painted {painted_peak} and shared {shared:?} in {painted:?}"
    );
    assert!(
        painted
            .lines()
            .any(|line| line == format!("network={}", shared.network)),
        "painted lines must include the shared network network={}, got {painted:?} from {shared:?}",
        shared.network
    );
}
