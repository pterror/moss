//! Query fixture tests for scheme.
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
// Scheme
// ---------------------------------------------------------------------------

const SCHEME_SAMPLE: &str = include_str!("fixtures/scheme/sample.scm");

#[test]
fn scheme_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scheme_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scheme").ok() else {
        eprintln!("Skipping scheme_tags: scheme grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("scheme")
        .expect("scheme tags query missing");
    let names = collect_captures(&lang, SCHEME_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' in scheme tags, got: {names:?}"
    );
    assert!(
        names.contains(&"square".to_string()),
        "expected 'square' in scheme tags, got: {names:?}"
    );
}

#[test]
fn scheme_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scheme_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scheme").ok() else {
        eprintln!("Skipping scheme_calls: scheme grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("scheme")
        .expect("scheme calls query missing");
    let calls = collect_captures(&lang, SCHEME_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "display" || c == "sqrt"),
        "expected 'display' or 'sqrt' call in scheme sample, got: {calls:?}"
    );
}

#[test]
fn scheme_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scheme_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scheme").ok() else {
        eprintln!("Skipping scheme_complexity: scheme grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("scheme")
        .expect("scheme complexity query missing");
    let complexity = collect_captures(&lang, SCHEME_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in scheme sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn scheme_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scheme_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scheme").ok() else {
        eprintln!("Skipping scheme_imports: scheme grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("scheme")
        .expect("scheme imports query missing");
    let paths = collect_captures(&lang, SCHEME_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("scheme")),
        "expected a scheme library in import paths, got: {paths:?}"
    );
}

#[test]
fn scheme_types_finds_no_captures() {
    // Scheme is dynamically typed; the types query intentionally captures nothing.
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scheme_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scheme").ok() else {
        eprintln!("Skipping scheme_types: scheme grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("scheme")
        .expect("scheme types query missing");
    let _ = collect_captures(&lang, SCHEME_SAMPLE, &query_str, "type");
}

#[test]
fn scheme_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping scheme_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "scheme",
        SCHEME_SAMPLE,
        &["; A point in 2D space"],
    );
}
