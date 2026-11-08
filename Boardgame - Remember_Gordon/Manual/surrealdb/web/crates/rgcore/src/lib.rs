//! The archive's database core, shared by the browser worker, the native checker (`rgcheck`) and the app.
//!
//! Without features this crate is only the protocol between the app and the worker: the dataset
//! manifest, loading progress, query results and the messages ([`Request`], [`Reply`]). The `engine`
//! feature adds [`Kb`], the corpus itself in in-memory SurrealDB (see `engine.rs`); the `web` feature
//! adds the browser fetch both the app and the worker use (see `web.rs`).
//!
//! This follows the bavovniatko knowledge-base explorer (db/web there), scaled down to one corpus.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "engine")]
mod engine;
#[cfg(feature = "engine")]
pub use engine::{Error, Kb, Source};
#[cfg(feature = "web")]
pub mod web;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub bytes: u64,
}

/// `data/manifest.json`, written by `web/export.py`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub format: u32,
    pub version: String,
    pub load: Vec<String>,
    pub files: Vec<FileEntry>,
    pub attribution: String,
}

impl Manifest {
    pub fn bytes(&self, path: &str) -> u64 {
        self.files
            .iter()
            .find(|f| f.path == path)
            .map_or(0, |f| f.bytes)
    }
}

/// Loading progress, reported after each file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    pub file: String,
    pub done_bytes: u64,
    pub total_bytes: u64,
    pub ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementResult {
    pub result: Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub ms: f64,
    pub statements: Vec<StatementResult>,
    /// Set when the query as a whole failed (e.g. a parse error).
    pub error: Option<String>,
}

impl QueryResult {
    pub fn failed(error: impl Into<String>) -> Self {
        QueryResult {
            ms: 0.0,
            statements: vec![],
            error: Some(error.into()),
        }
    }

    pub fn loading() -> Self {
        Self::failed("the archive is still loading")
    }

    pub fn first_error(&self) -> Option<&str> {
        self.error
            .as_deref()
            .or_else(|| self.statements.iter().find_map(|s| s.error.as_deref()))
    }

    /// The last statement's result, or the first error. This is how every question is answered.
    pub fn into_last(self) -> Result<Value, String> {
        if let Some(e) = self.first_error() {
            return Err(e.to_string());
        }
        Ok(self
            .statements
            .into_iter()
            .last()
            .map(|s| s.result)
            .unwrap_or(Value::Null))
    }
}

/// App → worker: run a query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub query: String,
}

/// Worker → app.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Reply {
    Progress(Progress),
    Ready {
        manifest: Manifest,
        tables: BTreeMap<String, u64>,
        ms: f64,
    },
    Failed {
        error: String,
    },
    Result {
        id: u64,
        result: QueryResult,
    },
}

/// Normalise a JSON value for comparison: integral floats become integers, objects sort their keys.
pub fn normalize(v: &Value) -> Value {
    match v {
        Value::Array(a) => Value::Array(a.iter().map(normalize).collect()),
        Value::Object(o) => {
            Value::Object(o.iter().map(|(k, v)| (k.clone(), normalize(v))).collect())
        }
        Value::Number(n) => match n.as_f64() {
            Some(f) if n.is_f64() && f.fract() == 0.0 && f.abs() < 9e15 => Value::from(f as i64),
            _ => v.clone(),
        },
        _ => v.clone(),
    }
}

/// Does a question's result match its expected answer? `mode` is the benchmark's `match`:
/// "exact" (default), "set" (order-insensitive list) or "contains" (every expected item present).
pub fn matches(got: &Value, expect: &Value, mode: &str) -> bool {
    let (g, e) = (normalize(got), normalize(expect));
    match mode {
        "set" => match (g.as_array(), e.as_array()) {
            (Some(a), Some(b)) => {
                let key = |x: &Value| serde_json::to_string(x).unwrap_or_default();
                let mut a: Vec<String> = a.iter().map(key).collect();
                let mut b: Vec<String> = b.iter().map(key).collect();
                a.sort();
                b.sort();
                a == b
            }
            _ => false,
        },
        "contains" => match (g.as_array(), e.as_array()) {
            (Some(a), Some(b)) => b.iter().all(|x| a.contains(x)),
            _ => false,
        },
        _ => g == e,
    }
}

/// Counts per table, as the worker reports them once loaded.
pub type Tables = BTreeMap<String, u64>;
