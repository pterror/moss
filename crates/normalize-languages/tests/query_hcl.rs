//! Query fixture tests for hcl.
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
// HCL (Terraform)
// ---------------------------------------------------------------------------

const HCL_SAMPLE: &str = include_str!("fixtures/hcl/sample.tf");

#[test]
fn hcl_tags_finds_blocks() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hcl_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hcl").ok() else {
        eprintln!("Skipping hcl_tags: hcl grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("hcl").expect("hcl tags query missing");
    let names = collect_captures(&lang, HCL_SAMPLE, &query_str, "name");
    assert!(
        names
            .iter()
            .any(|n| n == "resource" || n == "variable" || n == "output"),
        "expected a block type in hcl tags, got: {names:?}"
    );
}

#[test]
fn hcl_types_finds_type_constraints() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hcl_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hcl").ok() else {
        eprintln!("Skipping hcl_types: hcl grammar .so not found");
        return;
    };
    let query_str = loader.get_types("hcl").expect("hcl types query missing");
    let types = collect_captures(&lang, HCL_SAMPLE, &query_str, "type");
    assert!(
        !types.is_empty(),
        "expected at least one type constraint in hcl sample, got: {types:?}"
    );
}

#[test]
fn hcl_complexity_finds_conditionals() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hcl_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hcl").ok() else {
        eprintln!("Skipping hcl_complexity: hcl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("hcl")
        .expect("hcl complexity query missing");
    let complexity = collect_captures(&lang, HCL_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in hcl sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn hcl_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hcl_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hcl").ok() else {
        eprintln!("Skipping hcl_calls: hcl grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("hcl").expect("hcl calls query missing");
    let calls = collect_captures(&lang, HCL_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "merge" || c == "toset" || c == "lookup"),
        "expected a HCL function call in hcl sample, got: {calls:?}"
    );
}

#[test]
fn hcl_imports_finds_module_sources() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping hcl_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("hcl").ok() else {
        eprintln!("Skipping hcl_imports: hcl grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("hcl")
        .expect("hcl imports query missing");
    let paths = collect_captures(&lang, HCL_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p.contains("modules/vpc") || p.contains("vpc")),
        "expected a module source path in hcl sample, got: {paths:?}"
    );
}
