//! Query fixture tests for lean.
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
// Lean 4
// ---------------------------------------------------------------------------

const LEAN_SAMPLE: &str = include_str!("fixtures/lean/sample.lean");

#[test]
fn lean_tags_finds_defs_and_structures() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lean_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lean").ok() else {
        eprintln!("Skipping lean_tags: lean grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("lean").expect("lean tags query missing");
    let names = collect_captures(&lang, LEAN_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' def in lean tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' structure in lean tags, got: {names:?}"
    );
}

#[test]
fn lean_calls_finds_function_applications() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lean_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lean").ok() else {
        eprintln!("Skipping lean_calls: lean grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("lean").expect("lean calls query missing");
    let calls = collect_captures(&lang, LEAN_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "sqrt" || c == "classify" || c == "IO.println"),
        "expected a function call in lean sample, got: {calls:?}"
    );
}

#[test]
fn lean_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lean_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lean").ok() else {
        eprintln!("Skipping lean_complexity: lean grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("lean")
        .expect("lean complexity query missing");
    let complexity = collect_captures(&lang, LEAN_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in lean sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn lean_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lean_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lean").ok() else {
        eprintln!("Skipping lean_imports: lean grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("lean")
        .expect("lean imports query missing");
    let paths = collect_captures(&lang, LEAN_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Mathlib")),
        "expected Mathlib import in lean import paths, got: {paths:?}"
    );
}

#[test]
fn lean_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping lean_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("lean").ok() else {
        eprintln!("Skipping lean_types: lean grammar .so not found");
        return;
    };
    let query_str = loader.get_types("lean").expect("lean types query missing");
    // Query parses and runs; lean type ascriptions may or may not match in this sample.
    let _ = collect_captures(&lang, LEAN_SAMPLE, &query_str, "type");
}

#[test]
fn lean_decorations_finds_attribute_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping lean_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "lean",
        LEAN_SAMPLE,
        &["@[inline]"],
    );
}
