//! Query fixture tests for typst.
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
// Typst
// ---------------------------------------------------------------------------

const TYPST_SAMPLE: &str = include_str!("fixtures/typst/sample.typ");

#[test]
fn typst_tags_finds_let_bindings() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typst_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typst").ok() else {
        eprintln!("Skipping typst_tags: typst grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("typst").expect("typst tags query missing");
    let names = collect_captures(&lang, TYPST_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "format_version" || n == "summary_table"),
        "expected function let bindings in typst tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "project_name" || n == "version"),
        "expected variable let bindings in typst tags, got: {names:?}"
    );
}

#[test]
fn typst_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typst_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typst").ok() else {
        eprintln!("Skipping typst_calls: typst grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("typst")
        .expect("typst calls query missing");
    let calls = collect_captures(&lang, TYPST_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "tablex" || c == "format_version" || c == "summary_table"),
        "expected function calls in typst sample, got: {calls:?}"
    );
}

#[test]
fn typst_complexity_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typst_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typst").ok() else {
        eprintln!("Skipping typst_complexity: typst grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("typst")
        .expect("typst complexity query missing");
    let _complexity = collect_captures(&lang, TYPST_SAMPLE, &query_str, "complexity");
}

#[test]
fn typst_imports_finds_import_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping typst_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("typst").ok() else {
        eprintln!("Skipping typst_imports: typst grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("typst")
        .expect("typst imports query missing");
    let paths = collect_captures(&lang, TYPST_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("template.typ") || p.contains("tablex")),
        "expected import paths in typst sample, got: {paths:?}"
    );
}
