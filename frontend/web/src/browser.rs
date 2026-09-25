use crate::auth::ClientError;
use splora_api::Network;

#[cfg(not(target_arch = "wasm32"))]
pub fn browser_signer_present() -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn signed_get(
    _origin: &str,
    _network: Network,
    _indexer_path: &str,
) -> Result<String, ClientError> {
    Err(ClientError::MissingSigner)
}

#[cfg(target_arch = "wasm32")]
pub fn browser_signer_present() -> bool {
    use js_sys::Reflect;
    use wasm_bindgen::JsValue;
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(nostr) = Reflect::get(&window, &JsValue::from_str("nostr")) else {
        return false;
    };
    nostr.is_object()
}

#[cfg(target_arch = "wasm32")]
pub fn page_origin() -> String {
    web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .filter(|origin| !origin.is_empty())
        .unwrap_or_else(|| splora_api::BASE_URL.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn page_origin() -> String {
    splora_api::BASE_URL.to_string()
}

#[cfg(target_arch = "wasm32")]
pub async fn signed_get(
    origin: &str,
    network: Network,
    indexer_path: &str,
) -> Result<String, ClientError> {
    signed_fetch(origin, network, indexer_path, "GET", None).await
}

#[cfg(target_arch = "wasm32")]
pub async fn signed_post(
    origin: &str,
    network: Network,
    method: &str,
    indexer_path: &str,
    payload: &str,
) -> Result<String, ClientError> {
    if method != "POST" || !crate::paths::allowed_post_path(indexer_path) {
        return Err(ClientError::RejectedEvent);
    }
    signed_fetch(origin, network, indexer_path, method, Some(payload)).await
}

#[cfg(target_arch = "wasm32")]
async fn signed_fetch(
    origin: &str,
    network: Network,
    indexer_path: &str,
    method: &str,
    payload: Option<&str>,
) -> Result<String, ClientError> {
    use crate::auth::{browser_fetch_url, unsigned_method};
    use js_sys::{Array, Function, JSON, Object, Promise, Reflect};
    use splora_api::authorization_header;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

    if payload.is_some_and(|body| body.contains("nsec1")) {
        return Err(ClientError::RejectedEvent);
    }
    if !browser_signer_present() {
        return Err(ClientError::MissingSigner);
    }
    let window = web_sys::window().ok_or(ClientError::MissingSigner)?;
    let nostr = Reflect::get(&window, &JsValue::from_str("nostr"))
        .map_err(|_| ClientError::MissingSigner)?;
    let get_public_key = Reflect::get(&nostr, &JsValue::from_str("getPublicKey"))
        .map_err(|_| ClientError::MissingSigner)?
        .dyn_into::<Function>()
        .map_err(|_| ClientError::MissingSigner)?;
    let pk_promise = get_public_key
        .call0(&nostr)
        .map_err(|_| ClientError::MissingSigner)?;
    let pk_val: JsValue = JsFuture::from(Promise::unchecked_from_js(pk_promise))
        .await
        .map_err(|_| ClientError::MissingSigner)?;
    let pubkey = pk_val
        .as_string()
        .filter(|key| !key.is_empty())
        .ok_or(ClientError::MissingSigner)?;

    let created_at = (js_sys::Date::now() / 1000.0) as u64;
    let unsigned = unsigned_method(&pubkey, created_at, origin, indexer_path, method);
    let event = Object::new();
    Reflect::set(
        &event,
        &JsValue::from_str("pubkey"),
        &JsValue::from_str(&unsigned.pubkey),
    )
    .map_err(|_| ClientError::RejectedEvent)?;
    Reflect::set(
        &event,
        &JsValue::from_str("created_at"),
        &JsValue::from_f64(unsigned.created_at as f64),
    )
    .map_err(|_| ClientError::RejectedEvent)?;
    Reflect::set(
        &event,
        &JsValue::from_str("kind"),
        &JsValue::from(unsigned.kind),
    )
    .map_err(|_| ClientError::RejectedEvent)?;
    Reflect::set(
        &event,
        &JsValue::from_str("content"),
        &JsValue::from_str(""),
    )
    .map_err(|_| ClientError::RejectedEvent)?;
    let tags = Array::new();
    for tag in &unsigned.tags {
        let row = Array::new();
        for part in tag {
            row.push(&JsValue::from_str(part));
        }
        tags.push(&row);
    }
    Reflect::set(&event, &JsValue::from_str("tags"), &tags)
        .map_err(|_| ClientError::RejectedEvent)?;

    let sign_event = Reflect::get(&nostr, &JsValue::from_str("signEvent"))
        .map_err(|_| ClientError::MissingSigner)?
        .dyn_into::<Function>()
        .map_err(|_| ClientError::MissingSigner)?;
    let signed_promise = sign_event
        .call1(&nostr, &event)
        .map_err(|_| ClientError::MissingSigner)?;
    let signed_val: JsValue = JsFuture::from(Promise::unchecked_from_js(signed_promise))
        .await
        .map_err(|_| ClientError::MissingSigner)?;
    let signed_json = JSON::stringify(&signed_val)
        .map_err(|_| ClientError::RejectedEvent)?
        .as_string()
        .ok_or(ClientError::RejectedEvent)?;
    if signed_json.contains("nsec1") {
        return Err(ClientError::RejectedEvent);
    }
    let header = authorization_header(&signed_json).map_err(|_| ClientError::RejectedEvent)?;
    let url = browser_fetch_url(origin, network, indexer_path);

    let init = web_sys::RequestInit::new();
    init.set_method(method);
    if let Some(body) = payload {
        init.set_body_opt_str(Some(body));
    }
    let headers = web_sys::Headers::new().map_err(|_| ClientError::RejectedEvent)?;
    headers
        .set("Authorization", &header)
        .map_err(|_| ClientError::RejectedEvent)?;
    if payload.is_some() {
        headers
            .set("Content-Type", "text/plain")
            .map_err(|_| ClientError::RejectedEvent)?;
    }
    init.set_headers(headers.as_ref());
    let request = web_sys::Request::new_with_str_and_init(&url, &init)
        .map_err(|_| ClientError::RejectedEvent)?;
    let response_val = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| ClientError::RejectedEvent)?;
    let response: web_sys::Response = response_val
        .dyn_into()
        .map_err(|_| ClientError::RejectedEvent)?;
    let text_promise = response.text().map_err(|_| ClientError::RejectedEvent)?;
    let text: JsValue = JsFuture::from(text_promise)
        .await
        .map_err(|_| ClientError::RejectedEvent)?;
    text.as_string().ok_or(ClientError::RejectedEvent)
}
