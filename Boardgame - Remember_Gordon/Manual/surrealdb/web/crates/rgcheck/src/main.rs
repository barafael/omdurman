//! Load `crates/rgapp/public/data` exactly as the browser worker does, seal it, then:
//! run every benchmark question (as the guest a visitor is) against its expected answer, and check
//! that writes and schema statements change nothing.
//!
//!     cargo run --release -p rgcheck [-- -v]

use rgcore::{Kb, Source, matches};
use serde_json::Value;

struct Files(std::path::PathBuf);

impl Source for Files {
    async fn text(&self, path: &str) -> Result<String, String> {
        tokio::fs::read_to_string(self.0.join(path))
            .await
            .map_err(|e| format!("{path}: {e}"))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let verbose = std::env::args().any(|a| a == "-v");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../rgapp/public/data");
    let t0 = std::time::Instant::now();
    let mut kb = Kb::open().await.expect("open");
    let (manifest, tables) = kb
        .load(&Files(dir), |_| {})
        .await
        .unwrap_or_else(|e| panic!("load: {e}"));
    kb.seal().await.expect("seal");
    let rows: u64 = tables.values().sum();
    println!(
        "dataset {} loaded: {} tables, {rows} records, {:.0} ms",
        manifest.version,
        tables.len(),
        t0.elapsed().as_secs_f64() * 1e3
    );

    let qs = kb
        .query("SELECT record::id(id) AS id, query, expect, match FROM question ORDER BY id;")
        .await
        .into_last()
        .expect("questions");
    let qs = qs.as_array().cloned().unwrap_or_default();
    let (mut pass, mut fail) = (0, 0);
    let t1 = std::time::Instant::now();
    for q in &qs {
        let id = q["id"].as_str().unwrap_or("?");
        let Some(query) = q["query"].as_str() else {
            continue;
        };
        let got = kb.query(query).await.into_last();
        let ok = match &got {
            Ok(v) => matches(v, &q["expect"], q["match"].as_str().unwrap_or("exact")),
            Err(_) => false,
        };
        if ok {
            pass += 1
        } else {
            fail += 1;
            println!(
                "FAIL {id}\n  query:  {query}\n  expect: {}\n  got:    {}",
                q["expect"],
                match got {
                    Ok(v) => v.to_string(),
                    Err(e) => format!("ERROR {e}"),
                }
            );
        }
        if verbose && ok {
            println!("ok   {id}");
        }
    }
    println!(
        "questions: {pass} passed, {fail} failed ({:.0} ms)",
        t1.elapsed().as_secs_f64() * 1e3
    );

    // Write probes: none may change anything.
    let before = kb
        .query("RETURN [count(SELECT id FROM unit), unit:sultan.name];")
        .await
        .into_last()
        .unwrap_or(Value::Null);
    let probes = [
        "CREATE unit:intruder SET name = 'x';",
        "UPDATE unit:sultan SET name = 'Sunk';",
        "DELETE unit:sultan;",
        "RELATE unit:sultan->is_a->unit_class:fort;",
        "DEFINE TABLE intruder;",
        "REMOVE TABLE unit;",
    ];
    let mut refused = 0;
    for p in probes {
        let r = kb.query(p).await;
        if r.first_error().is_some() {
            refused += 1;
        }
    }
    let after = kb
        .query("RETURN [count(SELECT id FROM unit), unit:sultan.name];")
        .await
        .into_last()
        .unwrap_or(Value::Null);
    let intact = before == after && after != Value::Null;
    println!(
        "write probes: {refused}/{} refused outright, data intact: {intact} ({after})",
        probes.len()
    );
    if fail > 0 || !intact {
        std::process::exit(1);
    }
}
