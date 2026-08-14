//! Query fixture tests for elm.
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
// Elm
// ---------------------------------------------------------------------------

const ELM_SAMPLE: &str = include_str!("fixtures/elm/sample.elm");

#[test]
fn elm_tags_finds_functions_and_types() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elm_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elm").ok() else {
        eprintln!("Skipping elm_tags: elm grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("elm").expect("elm tags query missing");
    let names = collect_captures(&lang, ELM_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' function in elm tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Shape".to_string()),
        "expected 'Shape' type in elm tags, got: {names:?}"
    );
}

#[test]
fn elm_calls_finds_function_applications() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elm_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elm").ok() else {
        eprintln!("Skipping elm_calls: elm grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("elm").expect("elm calls query missing");
    let calls = collect_captures(&lang, ELM_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "sqrt" || c == "classify" || c == "area"),
        "expected a function call in elm sample, got: {calls:?}"
    );
}

#[test]
fn elm_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elm_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elm").ok() else {
        eprintln!("Skipping elm_complexity: elm grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("elm")
        .expect("elm complexity query missing");
    let complexity = collect_captures(&lang, ELM_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in elm sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn elm_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elm_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elm").ok() else {
        eprintln!("Skipping elm_imports: elm grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("elm")
        .expect("elm imports query missing");
    let paths = collect_captures(&lang, ELM_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Html")),
        "expected 'Html' in elm import paths, got: {paths:?}"
    );
}

#[test]
fn elm_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping elm_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("elm").ok() else {
        eprintln!("Skipping elm_types: elm grammar .so not found");
        return;
    };
    let query_str = loader.get_types("elm").expect("elm types query missing");
    let refs = collect_captures(&lang, ELM_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "Html" || r == "Float" || r == "Int" || r == "String"),
        "expected a type reference in elm sample, got: {refs:?}"
    );
}

#[test]
fn elm_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping elm_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "elm",
        ELM_SAMPLE,
        &["-- Square a number"],
    );
}
