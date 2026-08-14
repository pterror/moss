//! Query fixture tests for nix.
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
// Nix
// ---------------------------------------------------------------------------

const NIX_SAMPLE: &str = include_str!("fixtures/nix/sample.nix");

#[test]
fn nix_tags_finds_bindings() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nix_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nix").ok() else {
        eprintln!("Skipping nix_tags: nix grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("nix").expect("nix tags query missing");
    let names = collect_captures(&lang, NIX_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "greet" || n == "factorial"),
        "expected 'greet' or 'factorial' binding in nix tags, got: {names:?}"
    );
}

#[test]
fn nix_calls_finds_applications() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nix_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nix").ok() else {
        eprintln!("Skipping nix_calls: nix grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("nix").expect("nix calls query missing");
    let calls = collect_captures(&lang, NIX_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "factorial" || c == "greet" || c == "filter"),
        "expected an application in nix sample, got: {calls:?}"
    );
}

#[test]
fn nix_complexity_finds_if_expressions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nix_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nix").ok() else {
        eprintln!("Skipping nix_complexity: nix grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("nix")
        .expect("nix complexity query missing");
    let complexity = collect_captures(&lang, NIX_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in nix sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn nix_imports_finds_import_expressions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping nix_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("nix").ok() else {
        eprintln!("Skipping nix_imports: nix grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("nix")
        .expect("nix imports query missing");
    let paths = collect_captures(&lang, NIX_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("nixpkgs") || p.contains("src")),
        "expected an import path in nix sample, got: {paths:?}"
    );
}
