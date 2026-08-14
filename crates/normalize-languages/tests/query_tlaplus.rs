//! Query fixture tests for tlaplus.
//!
//! Split out of the former monolithic `query_fixtures.rs`; see
//! `tests/common/mod.rs` for the shared helpers.
//!
//! These tests require compiled grammar `.so` files in `target/grammars/`.
//! Build them with `cargo xtask build-grammars`. Without grammars present the
//! tests skip gracefully.

mod common;

#[allow(unused_imports)]
use common::*;
use normalize_languages::GrammarLoader;

// ---------------------------------------------------------------------------
// TLA+
// ---------------------------------------------------------------------------

const TLAPLUS_SAMPLE: &str = include_str!("fixtures/tlaplus/sample.tla");

#[test]
fn tlaplus_tags_finds_module_and_operators() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tlaplus_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tlaplus").ok() else {
        eprintln!("Skipping tlaplus_tags: tlaplus grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("tlaplus")
        .expect("tlaplus tags query missing");
    let names = collect_captures(&lang, TLAPLUS_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "Sample"),
        "expected 'Sample' module in tlaplus tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "Init" || n == "Next" || n == "Safety"),
        "expected an operator definition in tlaplus tags, got: {names:?}"
    );
}

#[test]
fn tlaplus_complexity_finds_conditionals() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tlaplus_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tlaplus").ok() else {
        eprintln!("Skipping tlaplus_complexity: tlaplus grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("tlaplus")
        .expect("tlaplus complexity query missing");
    let complexity = collect_captures(&lang, TLAPLUS_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in tlaplus sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn tlaplus_imports_finds_extends() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tlaplus_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tlaplus").ok() else {
        eprintln!("Skipping tlaplus_imports: tlaplus grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("tlaplus")
        .expect("tlaplus imports query missing");
    let paths = collect_captures(&lang, TLAPLUS_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("Naturals") || p.contains("Sequences")),
        "expected 'Naturals' or 'Sequences' in tlaplus import paths, got: {paths:?}"
    );
}
