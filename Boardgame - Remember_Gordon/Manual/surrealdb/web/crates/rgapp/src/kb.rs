//! Client for the corpus worker (crates/rgworker): starts it, tracks loading, and turns its replies
//! into futures. The worker and the pending replies live in a thread-local; the reactive state is a
//! `Copy` handle shared through context.

use std::cell::RefCell;
use std::collections::HashMap;

use futures::channel::oneshot;
use leptos::prelude::*;
pub use rgcore::QueryResult;
use rgcore::{Manifest, Progress, Reply, Request, Tables};
use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, Worker};

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Loading(Progress),
    Ready {
        manifest: Manifest,
        tables: Tables,
        ms: f64,
    },
    Failed(String),
}

#[derive(Clone, Copy)]
pub struct Kb {
    pub status: RwSignal<Status>,
    /// Whether the corpus is loaded; changes only once, so readers aren't re-run per file.
    pub ready: Memo<bool>,
    /// Record counts per table, once loaded (also: which strings are record ids).
    pub tables: RwSignal<Tables>,
}

struct Inner {
    worker: Worker,
    next_id: u64,
    pending: HashMap<u64, oneshot::Sender<QueryResult>>,
}

/// Set by web/build.sh (the dataset version); "dev" for local builds.
pub const BUILD: &str = match option_env!("RG_BUILD") {
    Some(b) => b,
    None => "dev",
};

thread_local! {
    static INNER: RefCell<Option<Inner>> = const { RefCell::new(None) };
}

impl Kb {
    pub fn start() -> Kb {
        let status = RwSignal::new(Status::Loading(Progress {
            file: "manifest.json".into(),
            done_bytes: 0,
            total_bytes: 1,
            ms: 0.0,
        }));
        let kb = Kb {
            status,
            ready: Memo::new(move |_| status.with(|s| matches!(s, Status::Ready { .. }))),
            tables: RwSignal::new(Tables::new()),
        };
        let worker = Worker::new(&format!("./rgworker_loader.js?v={BUILD}"))
            .expect("start the database worker");
        let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |ev: MessageEvent| {
            let Some(text) = ev.data().as_string() else {
                return;
            };
            let reply = match serde_json::from_str::<Reply>(&text) {
                Ok(r) => r,
                Err(e) => {
                    return kb
                        .status
                        .set(Status::Failed(format!("bad reply from the worker: {e}")));
                }
            };
            match reply {
                Reply::Progress(p) => kb.status.set(Status::Loading(p)),
                Reply::Ready {
                    manifest,
                    tables,
                    ms,
                } => {
                    kb.tables.set(tables.clone());
                    kb.status.set(Status::Ready {
                        manifest,
                        tables,
                        ms,
                    });
                }
                Reply::Failed { error } => kb.status.set(Status::Failed(error)),
                Reply::Result { id, result } => {
                    if let Some(tx) =
                        INNER.with(|i| i.borrow_mut().as_mut().and_then(|i| i.pending.remove(&id)))
                    {
                        let _ = tx.send(result);
                    }
                }
            }
        });
        worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
        INNER.with(|i| {
            *i.borrow_mut() = Some(Inner {
                worker,
                next_id: 1,
                pending: HashMap::new(),
            })
        });
        kb
    }

    pub async fn query(&self, q: String) -> QueryResult {
        let rx = INNER.with(|i| {
            let mut i = i.borrow_mut();
            let inner = i.as_mut()?;
            let id = inner.next_id;
            inner.next_id += 1;
            let (tx, rx) = oneshot::channel();
            inner.pending.insert(id, tx);
            let msg = serde_json::to_string(&Request { id, query: q }).ok()?;
            inner.worker.post_message(&JsValue::from_str(&msg)).ok()?;
            Some(rx)
        });
        match rx {
            Some(rx) => rx
                .await
                .unwrap_or_else(|_| QueryResult::failed("the worker stopped")),
            None => QueryResult::failed("the worker is not running"),
        }
    }

    /// The last statement's result, or the first error.
    pub async fn last(&self, q: impl Into<String>) -> Result<Value, String> {
        self.query(q.into()).await.into_last()
    }

    /// The table of a record id in a result ("unit:sultan" → "unit"), if the string is one.
    pub fn record_table(&self, s: &str) -> Option<String> {
        let (table, key) = s.split_once(':')?;
        (!key.is_empty()
            && !key.contains(' ')
            && self.tables.with_untracked(|t| t.contains_key(table)))
        .then(|| table.to_string())
    }
}
