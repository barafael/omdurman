//! Fetching text in the browser: the app fetches from its page, the worker from its script.

use js_sys::Promise;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;

/// The body of a `fetch` once it answers, as text. Errors start with `what` (the path or URL).
pub async fn response_text(fetch: Promise, what: &str) -> Result<String, String> {
    let resp: Response = JsFuture::from(fetch)
        .await
        .map_err(|e| format!("{what}: {e:?}"))?
        .unchecked_into();
    if !resp.ok() {
        return Err(format!("{what}: HTTP {}", resp.status()));
    }
    let text = JsFuture::from(resp.text().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?;
    text.as_string().ok_or_else(|| format!("{what}: not text"))
}
