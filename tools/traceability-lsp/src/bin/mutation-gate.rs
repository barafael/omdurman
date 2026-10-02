//! The mutation gate: the rulebook tests must pin the code they cite.
//!
//! For every function in `omdurman-rules` that an `implemented` mapping of
//! `docs/traceability.toml` cites, cargo-mutants mutates it and runs only the
//! engine tests of the sections that cite it. A mutant those tests all miss
//! means the cited code can change without any rulebook test noticing.
//!
//! ```sh
//! cargo run -p traceability-lsp --bin mutation-gate                    # every cited function
//! cargo run -p traceability-lsp --bin mutation-gate -- --in-diff pr.diff  # the gate on a change
//! cargo run -p traceability-lsp --bin mutation-gate -- --section §5.53 --list
//! ```
//!
//! Options: `--in-diff <file>` (only mutants on lines the diff touches, as CI
//! runs it), `--section <§N>` (repeatable: only the functions those sections
//! cite, still judged by every citing section's tests), `--list` (show the plan, run
//! nothing), `--in-place` (mutate the checkout itself -- CI only), `--jobs
//! <n>`, `--output <dir>` (default `target/mutation-gate`). Exits non-zero
//! when a mutant is missed or a cited function has no engine test to run.
//! Accepted equivalent mutants go in `.cargo/mutants.toml` (`exclude_re`),
//! each with a comment saying why.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use traceability_lsp::checks::read_traceability;
use traceability_lsp::{traceability_path, workspace_root};

const PACKAGE: &str = "omdurman-rules";
const PACKAGE_SRC: &str = "omdurman-rules/src/";

struct Options {
    in_diff: Option<PathBuf>,
    sections: BTreeSet<String>,
    list: bool,
    in_place: bool,
    jobs: usize,
    output: PathBuf,
}

fn parse_args(root: &Path) -> Result<Options, String> {
    let mut opts = Options {
        in_diff: None,
        sections: BTreeSet::new(),
        list: false,
        in_place: false,
        jobs: 1,
        output: root.join("target/mutation-gate"),
    };
    let mut args = std::env::args().skip(1);
    let value = |args: &mut std::iter::Skip<std::env::Args>, name: &str| {
        args.next().ok_or(format!("{name} needs a value"))
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--in-diff" => opts.in_diff = Some(PathBuf::from(value(&mut args, "--in-diff")?)),
            "--section" => {
                opts.sections.insert(value(&mut args, "--section")?);
            }
            "--list" => opts.list = true,
            "--in-place" => opts.in_place = true,
            "--jobs" => {
                opts.jobs = value(&mut args, "--jobs")?
                    .parse()
                    .map_err(|e| format!("--jobs: {e}"))?;
            }
            "--output" => opts.output = PathBuf::from(value(&mut args, "--output")?),
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    Ok(opts)
}

/// One cited function and everything that vouches for it.
#[derive(Default)]
struct Group {
    sections: BTreeSet<String>,
    /// libtest names (`--exact`), from the citing sections' engine tests.
    tests: BTreeSet<String>,
    mutants: Vec<String>,
}

/// The libtest names of a matrix test. Matrix names are file paths
/// (`omdurman-rules::src::effects::tests::foo`) and libtest names are module
/// paths, which add inline modules (`effects::tests::tests::foo`), so the
/// engine's real test list (`cargo test -- --list`) decides: a name matches
/// when it is the file's module path plus the fn, with any inline modules
/// between. `None` for other crates' tests, which do not run against the
/// engine's mutants.
fn engine_test_names(qualified: &str, listed: &BTreeSet<String>) -> Option<Vec<String>> {
    let (prefix, name) = if let Some(path) = qualified.strip_prefix("omdurman-rules::src::") {
        let path = path.strip_prefix("lib::").unwrap_or(path);
        let (prefix, name) = path.rsplit_once("::").unwrap_or(("", path));
        (prefix.to_string(), name.to_string())
    } else {
        // Integration tests: `omdurman-rules::tests::<file>::<fn>`; the
        // binary's names start at its own root.
        let rest = qualified.strip_prefix("omdurman-rules::tests::")?;
        let (_file, name) = rest.split_once("::")?;
        (String::new(), name.to_string())
    };
    let tail = format!("::{name}");
    Some(
        listed
            .iter()
            .filter(|t| {
                let in_module = prefix.is_empty()
                    || *t == &format!("{prefix}::{name}")
                    || t.starts_with(&format!("{prefix}::"));
                in_module && (*t == &name || t.ends_with(&tail))
            })
            .cloned()
            .collect(),
    )
}

/// Every test in the engine crate's lib and integration test binaries.
fn engine_tests(root: &Path) -> Result<BTreeSet<String>, String> {
    let out = Command::new("cargo")
        .current_dir(root)
        .args(["test", "-p", PACKAGE, "--lib", "--tests", "--", "--list"])
        .output()
        .map_err(|e| format!("cannot run cargo test --list: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo test --list failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.strip_suffix(": test"))
        .map(str::to_string)
        .collect())
}

fn escape_regex(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        if "\\.+*?()|[]{}^$".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Rewrite a unified diff's file headers to git's default `a/`/`b/` prefixes
/// (the only ones cargo-mutants understands), whatever prefixes -- mnemonic
/// (`c/`, `i/`, `w/`, `o/`), none -- it was written with.
fn normalize_diff_prefixes(diff: &str) -> String {
    let strip = |path: &str| -> String {
        match path.split_once('/') {
            Some((p, rest)) if p.len() == 1 => rest.to_string(),
            _ => path.to_string(),
        }
    };
    let mut out = String::with_capacity(diff.len());
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("--- ").filter(|p| *p != "/dev/null") {
            out.push_str(&format!("--- a/{}", strip(path)));
        } else if let Some(path) = line.strip_prefix("+++ ").filter(|p| *p != "/dev/null") {
            out.push_str(&format!("+++ b/{}", strip(path)));
        } else if let Some(rest) = line.strip_prefix("diff --git ") {
            match rest.split_once(' ') {
                Some((old, new)) => {
                    out.push_str(&format!("diff --git a/{} b/{}", strip(old), strip(new)))
                }
                None => out.push_str(line),
            }
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn cargo_mutants(root: &Path) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(root).arg("mutants").args(["-p", PACKAGE]);
    // Each scratch copy must build into its own target dir: an inherited
    // CARGO_TARGET_DIR makes `--jobs` copies overwrite each other's test
    // binaries, and a mutant is then judged by another copy's (unmutated)
    // build -- a false "missed".
    cmd.env_remove("CARGO_TARGET_DIR");
    cmd
}

fn main() -> ExitCode {
    let root = workspace_root();
    let mut opts = match parse_args(&root) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("mutation-gate: {e}");
            return ExitCode::from(2);
        }
    };
    // cargo-mutants reads only `a/`/`b/` path prefixes: a diff written under
    // `diff.mnemonicPrefix` (`c/`, `w/`, `i/` ...) or `--no-prefix` matched no
    // file and the gate silently found nothing. Hand it a normalized copy.
    if let Some(diff) = &opts.in_diff {
        let text = match std::fs::read_to_string(diff) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("mutation-gate: {}: {e}", diff.display());
                return ExitCode::from(2);
            }
        };
        let normalized = opts.output.join("in.diff");
        let written = std::fs::create_dir_all(&opts.output)
            .and_then(|()| std::fs::write(&normalized, normalize_diff_prefixes(&text)));
        if let Err(e) = written {
            eprintln!("mutation-gate: {}: {e}", normalized.display());
            return ExitCode::from(2);
        }
        opts.in_diff = Some(normalized);
    }
    let table = match read_traceability(&traceability_path()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("mutation-gate: {e}");
            return ExitCode::from(2);
        }
    };

    let listed = match engine_tests(&root) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("mutation-gate: {e}");
            return ExitCode::from(2);
        }
    };
    let mut unresolved: BTreeSet<String> = BTreeSet::new();

    // Cited engine functions, keyed by (file, name) -- the name as
    // cargo-mutants reports it, last path segment only.
    let mut groups: BTreeMap<(String, String), Group> = BTreeMap::new();
    for m in &table.mappings {
        if m.status != "implemented" {
            continue;
        }
        for imp in m.impls.iter().filter(|i| i.file.starts_with(PACKAGE_SRC)) {
            let name = imp.symbol.rsplit("::").next().unwrap_or(&imp.symbol);
            let group = groups
                .entry((imp.file.clone(), name.to_string()))
                .or_default();
            group.sections.insert(m.section.clone());
            for test in &m.tests {
                match engine_test_names(test, &listed) {
                    Some(names) if names.is_empty() => {
                        unresolved.insert(test.clone());
                    }
                    Some(names) => group.tests.extend(names),
                    None => {}
                }
            }
        }
    }

    // Every mutant cargo-mutants would make (on the diff's lines, if given).
    let mut list = cargo_mutants(&root);
    list.args(["--list", "--json"]);
    if let Some(diff) = &opts.in_diff {
        list.arg("--in-diff").arg(diff);
    }
    let output = match list.output() {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            eprintln!(
                "mutation-gate: cargo mutants --list failed:\n{}",
                String::from_utf8_lossy(&o.stderr)
            );
            return ExitCode::from(2);
        }
        Err(e) => {
            eprintln!(
                "mutation-gate: cannot run cargo mutants ({e}); `cargo install cargo-mutants`"
            );
            return ExitCode::from(2);
        }
    };
    // An empty listing (nothing to mutate on the diff's lines) is no mutants.
    let listing = String::from_utf8_lossy(&output.stdout);
    let listing = if listing.trim().is_empty() {
        "[]"
    } else {
        listing.as_ref()
    };
    let mutants: Vec<serde_json::Value> = match serde_json::from_str(listing) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("mutation-gate: unreadable mutant list: {e}");
            return ExitCode::from(2);
        }
    };
    for mutant in &mutants {
        let (Some(file), Some(function), Some(name)) = (
            mutant["file"].as_str(),
            mutant["function"]["function_name"].as_str(),
            mutant["name"].as_str(),
        ) else {
            continue;
        };
        let function = function.rsplit("::").next().unwrap_or(function);
        if let Some(group) = groups.get_mut(&(file.to_string(), function.to_string())) {
            group.mutants.push(name.to_string());
        }
    }
    // `--section` picks which cited functions to run; each keeps the tests
    // of every section citing it, as the full gate does.
    groups.retain(|_, g| {
        !g.mutants.is_empty()
            && (opts.sections.is_empty() || !g.sections.is_disjoint(&opts.sections))
    });

    let total: usize = groups.values().map(|g| g.mutants.len()).sum();
    println!(
        "mutation-gate: {total} mutant(s) in {} cited function(s){}",
        groups.len(),
        if opts.in_diff.is_some() {
            " on changed lines"
        } else {
            ""
        }
    );
    for ((file, name), g) in &groups {
        println!(
            "  {file}::{name}  [{}]  {} mutant(s), {} test(s)",
            g.sections.iter().cloned().collect::<Vec<_>>().join(" "),
            g.mutants.len(),
            g.tests.len()
        );
    }
    if opts.list {
        return ExitCode::SUCCESS;
    }

    let mut failures: Vec<String> = unresolved
        .iter()
        .map(|t| format!("{t}: listed in the matrix but not among the engine's tests"))
        .collect();
    for ((file, name), g) in &groups {
        let sections = g.sections.iter().cloned().collect::<Vec<_>>().join(" ");
        if g.tests.is_empty() {
            failures.push(format!(
                "{file}::{name} [{sections}]: {} mutant(s) but no engine test cites these sections",
                g.mutants.len()
            ));
            continue;
        }
        let out_dir = opts.output.join(format!(
            "{}__{name}",
            file.trim_end_matches(".rs").replace('/', "_")
        ));
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            failures.push(format!("{}: {e}", out_dir.display()));
            continue;
        }
        let names = g
            .mutants
            .iter()
            .map(|n| escape_regex(n))
            .collect::<Vec<_>>()
            .join("|");
        let mut run = cargo_mutants(&root);
        run.args([
            "--test-workspace",
            "false",
            "--no-shuffle",
            "--gitignore",
            "true",
        ])
        .arg("--jobs")
        .arg(opts.jobs.to_string())
        .arg("--file")
        .arg(file)
        .arg("--re")
        .arg(format!("^(?:{names})$"))
        .arg("--output")
        .arg(&out_dir);
        if let Some(diff) = &opts.in_diff {
            run.arg("--in-diff").arg(diff);
        }
        if opts.in_place {
            run.arg("--in-place");
        } else {
            // The scratch copies of the tree go on disk next to the results,
            // not in a tmpfs /tmp.
            let scratch = opts.output.join("scratch");
            if let Err(e) = std::fs::create_dir_all(&scratch) {
                failures.push(format!("{}: {e}", scratch.display()));
                continue;
            }
            run.env("TMPDIR", scratch);
        }
        run.args(["--", "--lib", "--tests", "--", "--exact"])
            .args(&g.tests);
        println!("\n== {file}::{name} [{sections}]");
        // cargo-mutants exits 0 when every mutant was caught, 2 when some
        // were missed, 3 when some timed out (a hang counts as caught); any
        // other status -- a failed baseline (4), a usage or internal error --
        // means nothing was verified.
        let code = match run.status() {
            Ok(status) => status.code(),
            Err(e) => {
                failures.push(format!("{file}::{name}: cannot run cargo mutants: {e}"));
                continue;
            }
        };
        let results = out_dir.join("mutants.out");
        let outcome = |f: &str| -> BTreeSet<String> {
            std::fs::read_to_string(results.join(f))
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect()
        };
        match code {
            Some(0 | 2 | 3) => {
                // Judge exactly the planned mutants: cargo-mutants' `--re`
                // does not filter some genres (struct field deletions), so it
                // may test more than asked; those extras are ignored.
                let planned: BTreeSet<&String> = g.mutants.iter().collect();
                let [caught, missed, unviable, timeout] =
                    ["caught.txt", "missed.txt", "unviable.txt", "timeout.txt"].map(outcome);
                let count = |set: &BTreeSet<String>| set.iter().filter(|m| planned.contains(m)).count();
                println!(
                    "   caught {}, missed {}, unviable {}, timed out {}",
                    count(&caught),
                    count(&missed),
                    count(&unviable),
                    count(&timeout)
                );
                for mutant in &g.mutants {
                    if missed.contains(mutant) {
                        failures.push(format!(
                            "MISSED [{sections}] {mutant}  (tests: {})",
                            g.tests.iter().cloned().collect::<Vec<_>>().join(", ")
                        ));
                    } else if !caught.contains(mutant)
                        && !unviable.contains(mutant)
                        && !timeout.contains(mutant)
                    {
                        failures.push(format!("NOT TESTED [{sections}] {mutant}"));
                    }
                }
            }
            Some(4) => failures.push(format!(
                "{file}::{name} [{sections}]: the tests fail on the unmutated code (baseline)"
            )),
            code => failures.push(format!(
                "{file}::{name} [{sections}]: cargo mutants failed (exit {code:?}); nothing was verified"
            )),
        }
    }

    if failures.is_empty() {
        println!("\nmutation-gate: every cited mutant is caught by its sections' tests");
        ExitCode::SUCCESS
    } else {
        println!("\nmutation-gate: {} failure(s)", failures.len());
        for f in &failures {
            println!("  {f}");
        }
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_diff_prefixes;

    #[test]
    fn mnemonic_prefixes_become_a_and_b() {
        let diff = "diff --git c/src/x.rs w/src/x.rs\n--- c/src/x.rs\n+++ w/src/x.rs\n@@ -1 +1 @@\n-old\n+new\n";
        assert_eq!(
            normalize_diff_prefixes(diff),
            "diff --git a/src/x.rs b/src/x.rs\n--- a/src/x.rs\n+++ b/src/x.rs\n@@ -1 +1 @@\n-old\n+new\n"
        );
    }

    #[test]
    fn default_prefixes_and_dev_null_are_kept() {
        let diff = "--- /dev/null\n+++ b/src/new.rs\n";
        assert_eq!(normalize_diff_prefixes(diff), diff);
    }
}
