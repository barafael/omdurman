//! The archive's Web Worker: loads the corpus (`data/manifest.json` and the files it lists) into
//! in-memory SurrealDB, seals it read-only, then answers queries. Messages are JSON strings: a
//! [`Request`] in, a [`Reply`] out (see rgcore).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use js_sys::Promise;
use rgcore::web::response_text;
use rgcore::{Kb, Manifest, QueryResult, Reply, Request, Source};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

thread_local! {
    static KB: RefCell<Option<Rc<Kb>>> = const { RefCell::new(None) };
}

fn scope() -> DedicatedWorkerGlobalScope {
    js_sys::global().unchecked_into()
}

fn post(reply: &Reply) {
    let json = serde_json::to_string(reply).expect("replies serialise");
    let _ = scope().post_message(&JsValue::from_str(&json));
}

/// Fetches dataset files relative to the worker script; once the manifest is in, every listed file
/// is requested at once so downloads overlap with loading.
struct Fetcher {
    base: String,
    query: String,
    pending: RefCell<HashMap<String, Promise>>,
}

impl Fetcher {
    fn start(&self, path: &str) -> Promise {
        scope().fetch_with_str(&format!("{}{}{}", self.base, path, self.query))
    }
}

impl Source for Fetcher {
    async fn text(&self, path: &str) -> Result<String, String> {
        let p = self
            .pending
            .borrow_mut()
            .remove(path)
            .unwrap_or_else(|| self.start(path));
        let text = response_text(p, path).await?;
        if path == "manifest.json" {
            let m: Manifest = serde_json::from_str(&text).map_err(|e| e.to_string())?;
            let mut pending = self.pending.borrow_mut();
            for f in &m.load {
                pending.insert(f.clone(), self.start(f));
            }
        }
        Ok(text)
    }
}

async fn boot() {
    let t0 = js_sys::Date::now();
    let query = scope().location().search();
    let fetcher = Fetcher {
        base: "data/".into(),
        query,
        pending: RefCell::new(HashMap::new()),
    };
    let result = async {
        let mut kb = Kb::open().await?;
        let (manifest, tables) = kb.load(&fetcher, |p| post(&Reply::Progress(p))).await?;
        kb.seal().await?;
        Ok::<_, rgcore::Error>((kb, manifest, tables))
    }
    .await;
    match result {
        Ok((kb, manifest, tables)) => {
            KB.with(|k| *k.borrow_mut() = Some(Rc::new(kb)));
            post(&Reply::Ready {
                manifest,
                tables,
                ms: js_sys::Date::now() - t0,
            });
        }
        Err(e) => post(&Reply::Failed {
            error: e.to_string(),
        }),
    }
}

fn main() {
    console_error_panic_hook::set_once();
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(|ev: MessageEvent| {
        let Some(text) = ev.data().as_string() else {
            return;
        };
        let Ok(req) = serde_json::from_str::<Request>(&text) else {
            return;
        };
        spawn_local(async move {
            let result = match KB.with(|k| k.borrow().clone()) {
                Some(kb) => kb.query(&req.query).await,
                None => QueryResult::loading(),
            };
            post(&Reply::Result { id: req.id, result });
        });
    });
    scope().set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
    spawn_local(boot());
}
