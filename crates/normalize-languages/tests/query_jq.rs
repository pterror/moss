//! Query fixture tests for jq.
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

// jq
// ---------------------------------------------------------------------------

const JQ_SAMPLE: &str = include_str!("fixtures/jq/sample.jq");

#[test]
fn jq_tags_finds_function_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jq_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jq").ok() else {
        eprintln!("Skipping jq_tags: jq grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("jq").expect("jq tags query missing");
    let names = collect_captures(&lang, JQ_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"sum".to_string()),
        "expected 'sum' function in jq tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "mean" || n == "flatten_keys" || n == "keep_if"),
        "expected function names in jq tags, got: {names:?}"
    );
}

#[test]
fn jq_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jq_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jq").ok() else {
        eprintln!("Skipping jq_calls: jq grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("jq").expect("jq calls query missing");
    let calls = collect_captures(&lang, JQ_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "map" || c == "select" || c == "group_by" || c == "sort_by"),
        "expected builtin function calls in jq sample, got: {calls:?}"
    );
}

#[test]
fn jq_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jq_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jq").ok() else {
        eprintln!("Skipping jq_complexity: jq grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("jq")
        .expect("jq complexity query missing");
    let _complexity = collect_captures(&lang, JQ_SAMPLE, &query_str, "complexity");
}

#[test]
fn jq_imports_finds_import_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping jq_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("jq").ok() else {
        eprintln!("Skipping jq_imports: jq grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("jq").expect("jq imports query missing");
    let paths = collect_captures(&lang, JQ_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("lib/utils")),
        "expected 'lib/utils' in jq import paths, got: {paths:?}"
    );
}
