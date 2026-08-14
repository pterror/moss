//! Query fixture tests for zsh.
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
use tree_sitter::Parser;

// ---------------------------------------------------------------------------
// Zsh
// ---------------------------------------------------------------------------

const ZSH_SAMPLE: &str = include_str!("fixtures/zsh/sample.zsh");

/// Returns true if the zsh grammar can parse basic constructs correctly.
/// The arborium-zsh grammar is known to have severe parsing issues with
/// common zsh syntax (function definitions, control flow, commands).
/// When it's broken, we skip the query tests rather than fail them.
fn zsh_grammar_is_functional(lang: &tree_sitter::Language) -> bool {
    let mut parser = Parser::new();
    parser.set_language(lang).expect("set_language failed");
    // A well-formed zsh grammar should parse `function f { echo hi }` as
    // a function_definition, not as an ERROR node. Check that.
    let tree = parser
        .parse("function greet { echo hi; }", None)
        .expect("parse failed");
    let sexp = tree.root_node().to_sexp();
    sexp.contains("function_definition")
}

#[test]
fn zsh_tags_finds_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zsh_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zsh").ok() else {
        eprintln!("Skipping zsh_tags: zsh grammar .so not found");
        return;
    };
    if !zsh_grammar_is_functional(&lang) {
        eprintln!(
            "Skipping zsh_tags: zsh grammar cannot parse function definitions (known grammar limitation)"
        );
        return;
    }
    let query_str = loader.get_tags("zsh").expect("zsh tags query missing");
    let names = collect_captures(&lang, ZSH_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "classify" || n == "greet" || n == "sum_array"),
        "expected 'classify'/'greet'/'sum_array' in zsh tags, got: {names:?}"
    );
}

#[test]
fn zsh_calls_finds_command_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zsh_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zsh").ok() else {
        eprintln!("Skipping zsh_calls: zsh grammar .so not found");
        return;
    };
    if !zsh_grammar_is_functional(&lang) {
        eprintln!(
            "Skipping zsh_calls: zsh grammar cannot parse commands correctly (known grammar limitation)"
        );
        return;
    }
    let query_str = loader.get_calls("zsh").expect("zsh calls query missing");
    let calls = collect_captures(&lang, ZSH_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "greet" || c == "sum_array"),
        "expected a function call in zsh sample, got: {calls:?}"
    );
}

#[test]
fn zsh_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zsh_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zsh").ok() else {
        eprintln!("Skipping zsh_complexity: zsh grammar .so not found");
        return;
    };
    if !zsh_grammar_is_functional(&lang) {
        eprintln!(
            "Skipping zsh_complexity: zsh grammar cannot parse control flow (known grammar limitation)"
        );
        return;
    }
    let query_str = loader
        .get_complexity("zsh")
        .expect("zsh complexity query missing");
    let complexity = collect_captures(&lang, ZSH_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in zsh sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn zsh_imports_finds_source_commands() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zsh_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zsh").ok() else {
        eprintln!("Skipping zsh_imports: zsh grammar .so not found");
        return;
    };
    if !zsh_grammar_is_functional(&lang) {
        eprintln!(
            "Skipping zsh_imports: zsh grammar cannot parse source commands (known grammar limitation)"
        );
        return;
    }
    let query_str = loader
        .get_imports("zsh")
        .expect("zsh imports query missing");
    let paths = collect_captures(&lang, ZSH_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("utils") || p.contains("zsh") || p.contains("helpers")),
        "expected sourced file path in zsh imports, got: {paths:?}"
    );
}
