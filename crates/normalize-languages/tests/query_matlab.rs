//! Query fixture tests for matlab.
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
// MATLAB
// ---------------------------------------------------------------------------

const MATLAB_SAMPLE: &str = include_str!("fixtures/matlab/sample.m");

#[test]
fn matlab_tags_finds_functions_and_class() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping matlab_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("matlab").ok() else {
        eprintln!("Skipping matlab_tags: matlab grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("matlab")
        .expect("matlab tags query missing");
    let names = collect_captures(&lang, MATLAB_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "factorial"),
        "expected 'factorial' function in matlab tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "Shape"),
        "expected 'Shape' class in matlab tags, got: {names:?}"
    );
}

#[test]
fn matlab_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping matlab_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("matlab").ok() else {
        eprintln!("Skipping matlab_calls: matlab grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("matlab")
        .expect("matlab calls query missing");
    let calls = collect_captures(&lang, MATLAB_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "factorial" || c == "fprintf" || c == "length"),
        "expected a function call in matlab sample, got: {calls:?}"
    );
}

#[test]
fn matlab_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping matlab_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("matlab").ok() else {
        eprintln!("Skipping matlab_complexity: matlab grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("matlab")
        .expect("matlab complexity query missing");
    let complexity = collect_captures(&lang, MATLAB_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 3,
        "expected at least 3 complexity nodes in matlab sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn matlab_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping matlab_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("matlab").ok() else {
        eprintln!("Skipping matlab_imports: matlab grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("matlab")
        .expect("matlab imports query missing");
    let paths = collect_captures(&lang, MATLAB_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p == "matlab.io.*"),
        "expected 'matlab.io.*' in matlab imports, got: {paths:?}"
    );
    assert!(
        paths.iter().any(|p| p == "matlab.net.http.RequestMessage"),
        "expected 'matlab.net.http.RequestMessage' in matlab imports, got: {paths:?}"
    );
}
