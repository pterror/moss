//! Query fixture tests for verilog.
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
// Verilog
// ---------------------------------------------------------------------------

const VERILOG_SAMPLE: &str = include_str!("fixtures/verilog/sample.v");

#[test]
fn verilog_tags_finds_modules() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping verilog_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("verilog").ok() else {
        eprintln!("Skipping verilog_tags: verilog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("verilog")
        .expect("verilog tags query missing");
    let names = collect_captures(&lang, VERILOG_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"alu".to_string()),
        "expected 'alu' module in verilog tags, got: {names:?}"
    );
    assert!(
        names.contains(&"reg_file".to_string()),
        "expected 'reg_file' module in verilog tags, got: {names:?}"
    );
}

#[test]
fn verilog_calls_finds_task_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping verilog_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("verilog").ok() else {
        eprintln!("Skipping verilog_calls: verilog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("verilog")
        .expect("verilog calls query missing");
    let _calls = collect_captures(&lang, VERILOG_SAMPLE, &query_str, "call");
}

#[test]
fn verilog_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping verilog_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("verilog").ok() else {
        eprintln!("Skipping verilog_complexity: verilog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("verilog")
        .expect("verilog complexity query missing");
    let complexity = collect_captures(&lang, VERILOG_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node (always/case/if) in verilog sample, got: {complexity:?}"
    );
}

#[test]
fn verilog_imports_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping verilog_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("verilog").ok() else {
        eprintln!("Skipping verilog_imports: verilog grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("verilog")
        .expect("verilog imports query missing");
    let _paths = collect_captures(&lang, VERILOG_SAMPLE, &query_str, "import.path");
}

#[test]
fn verilog_decorations_finds_attribute_instance_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping verilog_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // attribute_instance is the verified node name for (* ... *) attributes in tree-sitter-verilog.
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "verilog",
        VERILOG_SAMPLE,
        &[
            "(* synthesis, keep *)",
            "// ALU module with basic arithmetic and logic operations",
        ],
    );
}
