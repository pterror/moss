//! Query fixture tests for scss.
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
// SCSS
// ---------------------------------------------------------------------------

const SCSS_SAMPLE: &str = include_str!("fixtures/scss/sample.scss");

#[test]
fn scss_tags_finds_mixins_functions_and_rules() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scss_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scss").ok() else {
        eprintln!("Skipping scss_tags: scss grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("scss").expect("scss tags query missing");
    let names = collect_captures(&lang, SCSS_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "flex-center" || n == "responsive"),
        "expected mixin names in scss tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "rem" || n == "shade"),
        "expected function names in scss tags, got: {names:?}"
    );
}

#[test]
fn scss_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scss_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scss").ok() else {
        eprintln!("Skipping scss_calls: scss grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("scss").expect("scss calls query missing");
    let calls = collect_captures(&lang, SCSS_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "darken" || c == "rgba" || c == "shade"),
        "expected function calls in scss sample, got: {calls:?}"
    );
}

#[test]
fn scss_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scss_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scss").ok() else {
        eprintln!("Skipping scss_complexity: scss grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("scss")
        .expect("scss complexity query missing");
    let complexity = collect_captures(&lang, SCSS_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node (@if/@each) in scss sample, got: {complexity:?}"
    );
}

#[test]
fn scss_imports_finds_use_and_import_statements() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping scss_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("scss").ok() else {
        eprintln!("Skipping scss_imports: scss grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("scss")
        .expect("scss imports query missing");
    let paths = collect_captures(&lang, SCSS_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("sass:math") || p.contains("variables") || p.contains("mixins")),
        "expected import paths in scss sample, got: {paths:?}"
    );
}
