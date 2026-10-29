//! Row 2. Cypress it() title: check first mempool block after skeleton loads.
//!
//! `ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts` visits `/`,
//! waits until the skeleton is gone, then requires
//! `#mempool-block-0 > .blockLink`. The parent id is the ngFor render index
//! on a received `GET /mempool` backlog.
//!
//! `Dashboard` stores that body. The loaded mempool section is
//! `dashboard_loaded_markup` of it. This test calls that function with the
//! body and checks the link child in that return only. It does not call the
//! three mempool helpers, and it does not concatenate them.

use splora_frontend_shared::{MempoolBacklog, first_mempool_block_from_backlog};
use splora_web::dashboard_loaded_markup;

/// `#parent > .blockLink` is an anchor with class `blockLink` inside the
/// element whose id is the shared link id, before that element's close.
fn direct_child_block_link(markup: &str, link_id: &str) -> bool {
    let open = format!("id=\"{link_id}\"");
    let Some(start) = markup.find(&open) else {
        return false;
    };
    let after_id = &markup[start + open.len()..];
    let Some(gt) = after_id.find('>') else {
        return false;
    };
    let inside = &after_id[gt + 1..];
    let Some(close) = inside.find("</div>") else {
        return false;
    };
    inside[..close].trim().starts_with("<a class=\"blockLink\"")
}

#[test]
fn check_first_mempool_block_after_skeleton_loads() {
    let count: u32 = 4;
    let vsize: u32 = 500;
    let total_fee: u64 = 90;
    let histogram: [(u32, u32); 1] = [(2, 100)];
    let histogram_json = histogram
        .iter()
        .map(|(rate, size)| format!("[{rate},{size}]"))
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        r#"{{"count":{count},"vsize":{vsize},"total_fee":{total_fee},"fee_histogram":[{histogram_json}]}}"#
    );
    let view = first_mempool_block_from_backlog(MempoolBacklog {
        count,
        vsize,
        total_fee,
        fee_histogram_len: histogram.len(),
    });
    assert!(
        view.block_link_exists && !view.link_id.is_empty(),
        "a received GET /mempool backlog projects a first block link, got {view:?}"
    );

    let page = dashboard_loaded_markup(&body);
    let child = direct_child_block_link(&page, "mempool-block-0");
    assert!(
        child,
        "#mempool-block-0 > .blockLink missing from the loaded dashboard return\n{page}"
    );
}
