//! The corpus runs as SurrealDB 4.0 in memory. [`Kb::open`] starts it with root authentication and a
//! random password that never leaves the process, [`Kb::load`] runs the corpus files as root, and
//! [`Kb::seal`] drops to an anonymous guest session for good. Every table grants only `select`
//! (schema.surql), so a visitor's writes are no-ops and schema statements are refused.

use std::time::Duration;

use surrealdb::Surreal;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::opt::Config;
use surrealdb::opt::auth::Root;
use surrealdb::opt::capabilities::Capabilities;
use surrealdb::types::Value;
use web_time::Instant;

use crate::{Manifest, Progress, QueryResult, StatementResult, Tables};

const NS: &str = "rg";
const DB: &str = "rg";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Db(#[from] surrealdb::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Source(String),
    #[error("statement {number}: {source}")]
    Statement {
        number: usize,
        source: surrealdb::Error,
    },
    #[error("{path}: {source}")]
    File { path: String, source: Box<Error> },
    #[error("the database is sealed")]
    Sealed,
}

/// Where the dataset files come from: `fetch` in the browser, the file system natively.
#[allow(async_fn_in_trait)]
pub trait Source {
    async fn text(&self, path: &str) -> Result<String, String>;
}

pub struct Kb {
    db: Surreal<Db>,
    sealed: bool,
}

fn random_password() -> String {
    let mut b = [0u8; 24];
    getrandom::fill(&mut b).expect("random bytes");
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn ms_since(t0: Instant) -> f64 {
    t0.elapsed().as_secs_f64() * 1e3
}

impl Kb {
    pub async fn open() -> Result<Self, Error> {
        let root = Root {
            username: "root".into(),
            password: random_password(),
        };
        let caps = Capabilities::default()
            .with_guest_access(true)
            .with_scripting(false);
        let cfg = Config::new()
            .user(root.clone())
            .capabilities(caps)
            .query_timeout(Duration::from_secs(30));
        let db = Surreal::new::<Mem>(cfg).await?;
        db.signin(root).await?;
        let kb = Kb { db, sealed: false };
        kb.exec(&format!(
            "DEFINE NAMESPACE IF NOT EXISTS {NS}; USE NS {NS}; DEFINE DATABASE IF NOT EXISTS {DB};"
        ))
        .await?;
        kb.db.use_ns(NS).use_db(DB).await?;
        Ok(kb)
    }

    /// Run the manifest's files in order, as root; `progress` after each file. Then count each table.
    pub async fn load<S: Source>(
        &self,
        src: &S,
        mut progress: impl FnMut(Progress),
    ) -> Result<(Manifest, Tables), Error> {
        if self.sealed {
            return Err(Error::Sealed);
        }
        let t0 = Instant::now();
        let manifest: Manifest =
            serde_json::from_str(&src.text("manifest.json").await.map_err(Error::Source)?)?;
        let total: u64 = manifest.load.iter().map(|p| manifest.bytes(p)).sum();
        let mut done = 0;
        for path in &manifest.load {
            let text = src.text(path).await.map_err(Error::Source)?;
            self.exec(&text).await.map_err(|e| Error::File {
                path: path.clone(),
                source: Box::new(e),
            })?;
            done += manifest.bytes(path);
            progress(Progress {
                file: path.clone(),
                done_bytes: done,
                total_bytes: total,
                ms: ms_since(t0),
            });
        }
        let mut r = self
            .db
            .query("RETURN object::keys((INFO FOR DB).tables);")
            .await?;
        let names: Vec<String> = r
            .take::<Value>(0)?
            .into_json_value()
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        let mut tables = Tables::new();
        for t in names {
            let mut r = self
                .db
                .query(format!("RETURN count(SELECT id FROM `{t}`);"))
                .await?;
            let n = r.take::<Value>(0)?.into_json_value().as_u64().unwrap_or(0);
            tables.insert(t, n);
        }
        Ok((manifest, tables))
    }

    /// Drop to an anonymous guest session for good: from here on the database is read-only.
    pub async fn seal(&mut self) -> Result<(), Error> {
        self.db.invalidate().await?;
        self.db.use_ns(NS).use_db(DB).await?;
        self.sealed = true;
        Ok(())
    }

    async fn exec(&self, q: &str) -> Result<(), Error> {
        let mut r = self.db.query(q).await?;
        match r.take_errors().into_iter().min_by_key(|(i, _)| *i) {
            Some((i, source)) => Err(Error::Statement {
                number: i + 1,
                source,
            }),
            None => Ok(()),
        }
    }

    /// Run a visitor's query (any number of statements). Refuses to run before [`Kb::seal`].
    pub async fn query(&self, q: &str) -> QueryResult {
        if !self.sealed {
            return QueryResult::loading();
        }
        let t0 = Instant::now();
        let mut r = match self.db.query(q).await {
            Ok(r) => r,
            Err(e) => {
                return QueryResult {
                    ms: ms_since(t0),
                    ..QueryResult::failed(e.to_string())
                };
            }
        };
        let n = r.num_statements();
        let mut errors = r.take_errors();
        let statements = (0..n)
            .map(|i| {
                let result = match errors.remove(&i) {
                    Some(e) => Err(e),
                    None => r.take::<Value>(i),
                };
                match result {
                    Ok(v) => StatementResult {
                        result: v.into_json_value(),
                        error: None,
                    },
                    Err(e) => StatementResult {
                        result: serde_json::Value::Null,
                        error: Some(e.to_string()),
                    },
                }
            })
            .collect();
        QueryResult {
            ms: ms_since(t0),
            statements,
            error: None,
        }
    }
}
