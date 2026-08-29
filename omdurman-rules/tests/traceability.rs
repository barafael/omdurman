//! Validate the bijective rulebook <-> implementation traceability matrix.
//!
//! Thin runner over the shared checks in the `traceability-lsp` crate, which
//! also feed the `traceability-lsp` editor server's live diagnostics — the test
//! and the editor always run the same logic.
//!
//! The matrix itself lives in `docs/traceability.toml`.
//!
//! Run: `cargo test -p omdurman-rules --test traceability`

#[test]
fn traceability_matrix_is_bijective() {
    use traceability_lsp::checks::check_matrix;

    let report = check_matrix(&traceability_lsp::workspace_root());
    if report.failures.is_empty() {
        eprintln!(
            "traceability OK: {} mappings, {} impl sites, {} source files with § refs checked, {} manual sections",
            report.num_mappings,
            report.num_impls,
            report.num_source_files,
            report.num_manual_sections,
        );
    } else {
        eprintln!("\n=== TRACEABILITY MATRIX ISSUES ===\n");
        for f in &report.failures {
            eprintln!("  [ ] {f}");
        }
        eprintln!(
            "\n{} failure(s) -- fix the TOML or the code to restore bijectivity.\n",
            report.failures.len()
        );
        panic!("traceability matrix check failed");
    }
}

/// Validate that the `tests = [...]` field in each [[mapping]] is bijective
/// with the `#[rulebook]` annotations in source code.
///
/// `omdurman-rules` tests are collected via `inventory` (from the
/// `#[rulebook]` proc-macro). `omdurman-app` tests are collected via source
/// scanning (they still use `// §` comments).
#[test]
fn test_coverage_mapping_is_bijective() {
    use traceability_lsp::checks::check_coverage;

    let report = check_coverage(&traceability_lsp::workspace_root());
    if report.failures.is_empty() {
        eprintln!(
            "test coverage bijectivity OK: {} test functions annotated across {} sections",
            report.num_tests, report.num_sections,
        );
    } else {
        eprintln!("\n=== TEST COVERAGE BIJECTIVITY ISSUES ===\n");
        for f in &report.failures {
            eprintln!("  [ ] {f}");
        }
        eprintln!(
            "\n{} failure(s) -- fix the TOML tests arrays or the #[rulebook] annotations.\n",
            report.failures.len()
        );
        panic!("test coverage bijectivity check failed");
    }
}

/// Every `implemented` mapping must list at least one test that (a) exists,
/// (b) is annotated with that section, and (c) is not `#[ignore]`d. This is
/// the hard form of the coverage-gap warning: an implemented rule without a
/// behavior proof fails the build.
#[test]
fn implemented_mappings_are_tested() {
    use traceability_lsp::checks::check_semantic_gap;

    let issues = check_semantic_gap(&traceability_lsp::workspace_root());
    if !issues.is_empty() {
        eprintln!(
            "\n=== IMPLEMENTED MAPPINGS WITHOUT TESTS ({}) ===\n",
            issues.len()
        );
        for g in &issues {
            eprintln!("  [ ] {} \"{}\" has no annotated test", g.section, g.title);
        }
        eprintln!(
            "\nWrite/list a #[rulebook]-annotated test for each, or downgrade the \
             mapping's status if no rule is enforced.\n"
        );
        panic!("{} implemented mapping(s) lack test coverage", issues.len());
    }
}
