//! Query fixture tests for rescript.
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
// ReScript
// ---------------------------------------------------------------------------

const RESCRIPT_SAMPLE: &str = include_str!("fixtures/rescript/sample.res");

#[test]
fn rescript_tags_finds_let_bindings_and_types() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rescript_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rescript").ok() else {
        eprintln!("Skipping rescript_tags: rescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("rescript")
        .expect("rescript tags query missing");
    let names = collect_captures(&lang, RESCRIPT_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' in rescript tags, got: {names:?}"
    );
    assert!(
        names.contains(&"point".to_string()),
        "expected 'point' type in rescript tags, got: {names:?}"
    );
}

#[test]
fn rescript_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rescript_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rescript").ok() else {
        eprintln!("Skipping rescript_calls: rescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("rescript")
        .expect("rescript calls query missing");
    let calls = collect_captures(&lang, RESCRIPT_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "square" || c == "classify"),
        "expected 'square' or 'classify' call in rescript sample, got: {calls:?}"
    );
}

#[test]
fn rescript_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rescript_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rescript").ok() else {
        eprintln!("Skipping rescript_complexity: rescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("rescript")
        .expect("rescript complexity query missing");
    let complexity = collect_captures(&lang, RESCRIPT_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in rescript sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn rescript_imports_finds_open_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rescript_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rescript").ok() else {
        eprintln!("Skipping rescript_imports: rescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("rescript")
        .expect("rescript imports query missing");
    let paths = collect_captures(&lang, RESCRIPT_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Belt")),
        "expected 'Belt' in rescript import paths, got: {paths:?}"
    );
}

#[test]
fn rescript_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping rescript_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("rescript").ok() else {
        eprintln!("Skipping rescript_types: rescript grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("rescript")
        .expect("rescript types query missing");
    let refs = collect_captures(&lang, RESCRIPT_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "float" || r == "int" || r == "point"),
        "expected a type reference in rescript sample, got: {refs:?}"
    );
}

#[test]
fn rescript_decorations_finds_decorator_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping rescript_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "rescript",
        RESCRIPT_SAMPLE,
        &["@inline"],
    );
}
