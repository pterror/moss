//! Query fixture tests for hlsl.
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
// HLSL
// ---------------------------------------------------------------------------

const HLSL_SAMPLE: &str = include_str!("fixtures/hlsl/sample.hlsl");

#[test]
fn hlsl_tags_finds_functions_structs_and_cbuffers() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hlsl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hlsl").ok() else {
        eprintln!("Skipping hlsl_tags: hlsl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("hlsl").expect("hlsl tags query missing");
    let names = collect_captures(&lang, HLSL_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "VSMain" || n == "PSMain" || n == "ComputeLighting"),
        "expected a function name in hlsl tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "PerFrame" || n == "PerObject" || n == "VSInput"),
        "expected a cbuffer or struct name in hlsl tags, got: {names:?}"
    );
}

#[test]
fn hlsl_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hlsl_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hlsl").ok() else {
        eprintln!("Skipping hlsl_calls: hlsl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("hlsl").expect("hlsl calls query missing");
    let calls = collect_captures(&lang, HLSL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "normalize" || c == "mul" || c == "ComputeLighting"),
        "expected function calls in hlsl sample, got: {calls:?}"
    );
}

#[test]
fn hlsl_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hlsl_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hlsl").ok() else {
        eprintln!("Skipping hlsl_complexity: hlsl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("hlsl")
        .expect("hlsl complexity query missing");
    let complexity = collect_captures(&lang, HLSL_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in hlsl sample, got: {complexity:?}"
    );
}

#[test]
fn hlsl_imports_finds_include_directives() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hlsl_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hlsl").ok() else {
        eprintln!("Skipping hlsl_imports: hlsl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("hlsl")
        .expect("hlsl imports query missing");
    let paths = collect_captures(&lang, HLSL_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("common.hlsl") || p.contains("d3d11.h")),
        "expected include paths in hlsl imports, got: {paths:?}"
    );
}
