//! Query fixture tests for prolog.
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
// Prolog
// ---------------------------------------------------------------------------

const PROLOG_SAMPLE: &str = include_str!("fixtures/prolog/sample.pl");

#[test]
fn prolog_tags_finds_predicates() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping prolog_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("prolog").ok() else {
        eprintln!("Skipping prolog_tags: prolog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("prolog")
        .expect("prolog tags query missing");
    let names = collect_captures(&lang, PROLOG_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "factorial" || n == "parent" || n == "ancestor"),
        "expected 'factorial', 'parent', or 'ancestor' in prolog tags, got: {names:?}"
    );
}

#[test]
fn prolog_calls_finds_predicate_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping prolog_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("prolog").ok() else {
        eprintln!("Skipping prolog_calls: prolog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("prolog")
        .expect("prolog calls query missing");
    let calls = collect_captures(&lang, PROLOG_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "factorial" || c == "parent" || c == "member"),
        "expected a predicate call in prolog sample, got: {calls:?}"
    );
}

#[test]
fn prolog_complexity_finds_clauses() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping prolog_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("prolog").ok() else {
        eprintln!("Skipping prolog_complexity: prolog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("prolog")
        .expect("prolog complexity query missing");
    let complexity = collect_captures(&lang, PROLOG_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in prolog sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn prolog_imports_finds_use_module() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping prolog_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("prolog").ok() else {
        eprintln!("Skipping prolog_imports: prolog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("prolog")
        .expect("prolog imports query missing");
    let paths = collect_captures(&lang, PROLOG_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("lists") || p.contains("apply")),
        "expected 'lists' or 'apply' in prolog import paths, got: {paths:?}"
    );
}

#[test]
fn prolog_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping prolog_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "prolog",
        PROLOG_SAMPLE,
        &["% Facts: family relationships"],
    );
}
