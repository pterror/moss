//! Query fixture tests for idris.
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
// Idris
// ---------------------------------------------------------------------------

const IDRIS_SAMPLE: &str = include_str!("fixtures/idris/sample.idr");

#[test]
fn idris_tags_finds_functions_and_types() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping idris_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("idris").ok() else {
        eprintln!("Skipping idris_tags: idris grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("idris").expect("idris tags query missing");
    let names = collect_captures(&lang, IDRIS_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' function in idris tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' data type in idris tags, got: {names:?}"
    );
}

#[test]
fn idris_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping idris_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("idris").ok() else {
        eprintln!("Skipping idris_calls: idris grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("idris")
        .expect("idris calls query missing");
    let calls = collect_captures(&lang, IDRIS_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "sqrt" || c == "printLn"),
        "expected 'sqrt' or 'printLn' call in idris sample, got: {calls:?}"
    );
}

#[test]
fn idris_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping idris_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("idris").ok() else {
        eprintln!("Skipping idris_complexity: idris grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("idris")
        .expect("idris complexity query missing");
    let complexity = collect_captures(&lang, IDRIS_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in idris sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn idris_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping idris_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("idris").ok() else {
        eprintln!("Skipping idris_imports: idris grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("idris")
        .expect("idris imports query missing");
    let paths = collect_captures(&lang, IDRIS_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Data")),
        "expected Data.* module in idris import paths, got: {paths:?}"
    );
}

#[test]
fn idris_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping idris_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("idris").ok() else {
        eprintln!("Skipping idris_types: idris grammar .so not found");
        return;
    };
    let query_str = loader
        .get_types("idris")
        .expect("idris types query missing");
    let refs = collect_captures(&lang, IDRIS_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "String" || r == "Int" || r == "Double"),
        "expected a type reference in idris sample, got: {refs:?}"
    );
}

#[test]
fn idris_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping idris_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // In tree-sitter-idris, ||| doc comments are parsed as (comment) by the external scanner —
    // there is no separate doc_comment node type.
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "idris",
        IDRIS_SAMPLE,
        &["||| Compute Euclidean distance between two points"],
    );
}
