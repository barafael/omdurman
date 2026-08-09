//! Deterministic rules vignette suite runner.
//!
//! Each script in [`omdurman_rules::tactics::all_scripts`] is a small,
//! hand-built [`GameState`] plus an ordered list of legal steps, illegal
//! probes, and state asserts. This test replays every script from a fresh
//! clone of its initial state and reports the first misbehaving step per
//! script, naming the script, step, and rulebook citation.
//!
//! Run: `cargo test -p omdurman-rules --test tactics`

use omdurman_rules::tactics::{all_scripts, run_step, ScriptStep};

/// The human-readable note attached to a scripted step.
fn step_note(step: &ScriptStep) -> &'static str {
    match step {
        ScriptStep::Legal { note, .. }
        | ScriptStep::Illegal { note, .. }
        | ScriptStep::Assert { note, .. } => note,
    }
}

#[test]
fn all_scripts_replay() {
    let mut failures: Vec<String> = Vec::new();

    for script in all_scripts() {
        let mut state = script.state.clone();
        let mut script_ok = true;

        for (i, step) in script.steps.iter().enumerate() {
            if let Some(err) = run_step(&mut state, step) {
                failures.push(format!(
                    "[{} §{}] step {} ({}) -- {}",
                    script.name,
                    script.citation,
                    i + 1,
                    step_note(step),
                    err,
                ));
                script_ok = false;
                break;
            }
        }

        if script_ok {
            eprintln!("PASS {}", script.name);
        }
    }

    if failures.is_empty() {
        eprintln!(
            "tactics OK: {} scripts, all steps replayed",
            all_scripts().len()
        );
    } else {
        eprintln!("\n=== TACTICS SUITE FAILURES ===\n");
        for f in &failures {
            eprintln!("  [ ] {f}");
        }
        eprintln!(
            "\n{} failure(s) -- fix the vignette or the engine before proceeding.\n",
            failures.len()
        );
        panic!("tactics suite failed");
    }
}
