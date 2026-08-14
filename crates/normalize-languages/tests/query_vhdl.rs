//! Query fixture tests for vhdl.
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
// VHDL
// ---------------------------------------------------------------------------

const VHDL_SAMPLE: &str = include_str!("fixtures/vhdl/sample.vhd");

#[test]
fn vhdl_tags_finds_entity_and_architecture() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vhdl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vhdl").ok() else {
        eprintln!("Skipping vhdl_tags: vhdl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("vhdl").expect("vhdl tags query missing");
    let names = collect_captures(&lang, VHDL_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"fifo".to_string()),
        "expected 'fifo' entity in vhdl tags, got: {names:?}"
    );
    assert!(
        names.contains(&"rtl".to_string()),
        "expected 'rtl' architecture in vhdl tags, got: {names:?}"
    );
}

#[test]
fn vhdl_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vhdl_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vhdl").ok() else {
        eprintln!("Skipping vhdl_calls: vhdl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("vhdl").expect("vhdl calls query missing");
    let calls = collect_captures(&lang, VHDL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "rising_edge" || c == "to_integer"),
        "expected function calls in vhdl sample, got: {calls:?}"
    );
}

#[test]
fn vhdl_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vhdl_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vhdl").ok() else {
        eprintln!("Skipping vhdl_complexity: vhdl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("vhdl")
        .expect("vhdl complexity query missing");
    let complexity = collect_captures(&lang, VHDL_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node (if/process) in vhdl sample, got: {complexity:?}"
    );
}

#[test]
fn vhdl_imports_finds_use_clauses() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vhdl_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vhdl").ok() else {
        eprintln!("Skipping vhdl_imports: vhdl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("vhdl")
        .expect("vhdl imports query missing");
    let paths = collect_captures(&lang, VHDL_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("std_logic_1164")
            || p.contains("numeric_std")
            || p.contains("ieee")),
        "expected use clause paths in vhdl sample, got: {paths:?}"
    );
}

#[test]
fn vhdl_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping vhdl_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "vhdl",
        VHDL_SAMPLE,
        &["-- Simple FIFO entity"],
    );
}
