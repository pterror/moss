//! Query fixture tests for fish.
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
// Fish
// ---------------------------------------------------------------------------

const FISH_SAMPLE: &str = include_str!("fixtures/fish/sample.fish");

#[test]
fn fish_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping fish_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("fish").ok() else {
        eprintln!("Skipping fish_tags: fish grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("fish").expect("fish tags query missing");
    let names = collect_captures(&lang, FISH_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "greet" || n == "sum_list"),
        "expected 'classify'/'greet'/'sum_list' in fish tags, got: {names:?}"
    );
}

#[test]
fn fish_calls_finds_command_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping fish_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("fish").ok() else {
        eprintln!("Skipping fish_calls: fish grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("fish").expect("fish calls query missing");
    let calls = collect_captures(&lang, FISH_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "greet" || c == "sum_list"),
        "expected a function call in fish sample, got: {calls:?}"
    );
}

#[test]
fn fish_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping fish_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("fish").ok() else {
        eprintln!("Skipping fish_complexity: fish grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("fish")
        .expect("fish complexity query missing");
    let complexity = collect_captures(&lang, FISH_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in fish sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn fish_imports_finds_source_commands() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping fish_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("fish").ok() else {
        eprintln!("Skipping fish_imports: fish grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("fish")
        .expect("fish imports query missing");
    let paths = collect_captures(&lang, FISH_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("utils") || p.contains("fish")),
        "expected sourced file path in fish imports, got: {paths:?}"
    );
}
