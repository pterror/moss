//! Query fixture tests for starlark.
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
// Starlark
// ---------------------------------------------------------------------------

const STARLARK_SAMPLE: &str = include_str!("fixtures/starlark/sample.star");

#[test]
fn starlark_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping starlark_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("starlark").ok() else {
        eprintln!("Skipping starlark_tags: starlark grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("starlark")
        .expect("starlark tags query missing");
    let names = collect_captures(&lang, STARLARK_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "make_cc_library"),
        "expected 'make_cc_library' in starlark tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "make_test_suite" || n == "filter_srcs"),
        "expected another function in starlark tags, got: {names:?}"
    );
}

#[test]
fn starlark_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping starlark_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("starlark").ok() else {
        eprintln!("Skipping starlark_calls: starlark grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("starlark")
        .expect("starlark calls query missing");
    let calls = collect_captures(&lang, STARLARK_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "cc_library" || c == "cc_binary" || c == "make_cc_library"),
        "expected a function call in starlark sample, got: {calls:?}"
    );
}

#[test]
fn starlark_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping starlark_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("starlark").ok() else {
        eprintln!("Skipping starlark_complexity: starlark grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("starlark")
        .expect("starlark complexity query missing");
    let complexity = collect_captures(&lang, STARLARK_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in starlark sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn starlark_imports_finds_load_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping starlark_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("starlark").ok() else {
        eprintln!("Skipping starlark_imports: starlark grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("starlark")
        .expect("starlark imports query missing");
    let paths = collect_captures(&lang, STARLARK_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("rules_cc") || p.contains("rules_python")),
        "expected a load path in starlark sample, got: {paths:?}"
    );
}
