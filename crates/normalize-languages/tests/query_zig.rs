//! Query fixture tests for zig.
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
// Zig
// ---------------------------------------------------------------------------

const ZIG_SAMPLE: &str = include_str!("fixtures/zig/sample.zig");

#[test]
fn zig_tags_finds_functions_and_structs() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zig_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zig").ok() else {
        eprintln!("Skipping zig_tags: zig grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("zig").expect("zig tags query missing");
    let names = collect_captures(&lang, ZIG_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"classify".to_string()),
        "expected 'classify' function in zig tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' struct in zig tags, got: {names:?}"
    );
}

#[test]
fn zig_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zig_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zig").ok() else {
        eprintln!("Skipping zig_calls: zig grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("zig").expect("zig calls query missing");
    let calls = collect_captures(&lang, ZIG_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "classify" || c == "sumSlice" || c == "origin"),
        "expected a function call in zig sample, got: {calls:?}"
    );
}

#[test]
fn zig_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zig_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zig").ok() else {
        eprintln!("Skipping zig_complexity: zig grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("zig")
        .expect("zig complexity query missing");
    let complexity = collect_captures(&lang, ZIG_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in zig sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn zig_imports_finds_module_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zig_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zig").ok() else {
        eprintln!("Skipping zig_imports: zig grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("zig")
        .expect("zig imports query missing");
    let paths = collect_captures(&lang, ZIG_SAMPLE, &query_str, "import");
    assert!(
        !paths.is_empty(),
        "expected at least one import in zig sample, got: {paths:?}"
    );
}

#[test]
fn zig_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping zig_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("zig").ok() else {
        eprintln!("Skipping zig_types: zig grammar .so not found");
        return;
    };
    let query_str = loader.get_types("zig").expect("zig types query missing");
    let refs = collect_captures(&lang, ZIG_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected at least one type reference in zig sample, got: {refs:?}"
    );
}

#[test]
fn zig_decorations_finds_doc_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping zig_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "zig",
        ZIG_SAMPLE,
        &["/// Classify a number as negative, zero, or positive."],
    );
}
