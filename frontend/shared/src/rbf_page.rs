//! Opening view of the RBF page. Cypress opening state: no replacements yet.
//! A later websocket payload is a separate `rbfLatest` list.

use crate::parse::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RbfPageInitial {
    pub shows_no_replacements: bool,
    pub replacement_tree_len: Option<u32>,
    pub requests: Vec<&'static str>,
}

/// Replacement trees from a websocket body `{ "rbfLatest": [ ... ] }`.
/// Each array element is one `.tree`. This is not the opening GET list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RbfSocketTrees {
    pub tree_len: u32,
}

/// Cypress opening state: no replacements, a tree length of 0, and the two replacement GETs.
pub fn rbf_page_initial() -> RbfPageInitial {
    RbfPageInitial {
        shows_no_replacements: true,
        replacement_tree_len: Some(0),
        requests: vec![
            "GET /api/v1/replacements",
            "GET /api/v1/fullrbf/replacements",
        ],
    }
}

/// Tree count for the socket fixture the native page already delivers.
/// The count is `rbfLatest.len()`, not a fixed number.
pub fn rbf_socket_trees(body: &str) -> Result<RbfSocketTrees, ParseError> {
    #[derive(serde::Deserialize)]
    struct Payload {
        #[serde(rename = "rbfLatest")]
        rbf_latest: Vec<serde_json::Value>,
    }
    let payload: Payload = serde_json::from_str(body)?;
    let tree_len = u32::try_from(payload.rbf_latest.len())
        .map_err(|_| ParseError::new("rbfLatest longer than u32"))?;
    Ok(RbfSocketTrees { tree_len })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rbf_page_initial_matches_cypress_opening_state() {
        let page = rbf_page_initial();
        assert_eq!(
            (
                page.shows_no_replacements,
                page.replacement_tree_len,
                page.requests.as_slice(),
            ),
            (
                true,
                Some(0),
                [
                    "GET /api/v1/replacements",
                    "GET /api/v1/fullrbf/replacements"
                ]
                .as_slice(),
            ),
            "RBF page gets updated over websockets"
        );
    }
}
