//! Query fixture tests for tsx.
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
// TSX
// ---------------------------------------------------------------------------

const TSX_SAMPLE: &str = include_str!("fixtures/tsx/sample.tsx");

#[test]
fn tsx_tags_finds_components_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tsx_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tsx").ok() else {
        eprintln!("Skipping tsx_tags: tsx grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("tsx").expect("tsx tags query missing");
    let names = collect_captures(&lang, TSX_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "Counter" || n == "Button" || n == "classify"),
        "expected 'Counter'/'Button'/'classify' in tsx tags, got: {names:?}"
    );
}

#[test]
fn tsx_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tsx_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tsx").ok() else {
        eprintln!("Skipping tsx_calls: tsx grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("tsx").expect("tsx calls query missing");
    let calls = collect_captures(&lang, TSX_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "useState" || c == "useEffect" || c == "classify"),
        "expected a hook/function call in tsx sample, got: {calls:?}"
    );
}

#[test]
fn tsx_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tsx_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tsx").ok() else {
        eprintln!("Skipping tsx_complexity: tsx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("tsx")
        .expect("tsx complexity query missing");
    let complexity = collect_captures(&lang, TSX_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in tsx sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn tsx_imports_finds_react_imports() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tsx_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tsx").ok() else {
        eprintln!("Skipping tsx_imports: tsx grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("tsx")
        .expect("tsx imports query missing");
    let paths = collect_captures(&lang, TSX_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p == "react" || p == "react-native"),
        "expected 'react'/'react-native' in tsx import paths, got: {paths:?}"
    );
}

#[test]
fn tsx_types_finds_interface_and_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping tsx_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("tsx").ok() else {
        eprintln!("Skipping tsx_types: tsx grammar .so not found");
        return;
    };
    let query_str = loader.get_types("tsx").expect("tsx types query missing");
    let refs = collect_captures(&lang, TSX_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected at least one type reference in tsx sample, got: {refs:?}"
    );
}

#[test]
fn tsx_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping tsx_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "tsx",
        TSX_SAMPLE,
        &["// Classify"],
    );
}
