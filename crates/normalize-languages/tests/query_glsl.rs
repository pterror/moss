//! Query fixture tests for glsl.
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
// ---------------------------------------------------------------------------
// GLSL
// ---------------------------------------------------------------------------

const GLSL_SAMPLE: &str = include_str!("fixtures/glsl/sample.glsl");

#[test]
fn glsl_tags_finds_functions_and_structs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping glsl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("glsl").ok() else {
        eprintln!("Skipping glsl_tags: glsl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("glsl").expect("glsl tags query missing");
    let names = collect_captures(&lang, GLSL_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"main".to_string()),
        "expected 'main' function in glsl tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "Material" || n == "calculateDiffuse" || n == "applyFog"),
        "expected a struct or function name in glsl tags, got: {names:?}"
    );
}

#[test]
fn glsl_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping glsl_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("glsl").ok() else {
        eprintln!("Skipping glsl_calls: glsl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("glsl").expect("glsl calls query missing");
    let calls = collect_captures(&lang, GLSL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "normalize" || c == "texture" || c == "calculateDiffuse"),
        "expected builtin or user function call in glsl sample, got: {calls:?}"
    );
}

#[test]
fn glsl_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping glsl_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("glsl").ok() else {
        eprintln!("Skipping glsl_complexity: glsl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("glsl")
        .expect("glsl complexity query missing");
    let complexity = collect_captures(&lang, GLSL_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in glsl sample, got: {complexity:?}"
    );
}

#[test]
fn glsl_imports_finds_include_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping glsl_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("glsl").ok() else {
        eprintln!("Skipping glsl_imports: glsl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("glsl")
        .expect("glsl imports query missing");
    let paths = collect_captures(&lang, GLSL_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("common") || p.contains("lighting")),
        "expected #include paths in glsl sample, got: {paths:?}"
    );
}
