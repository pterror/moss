//! Query fixture tests for vb.
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
// Visual Basic .NET
// ---------------------------------------------------------------------------

const VB_SAMPLE: &str = include_str!("fixtures/vb/sample.vb");

#[test]
fn vb_tags_finds_methods_and_classes() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vb_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vb").ok() else {
        eprintln!("Skipping vb_tags: vb grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("vb").expect("vb tags query missing");
    let names = collect_captures(&lang, VB_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Classify".to_string()),
        "expected 'Classify' method in vb tags, got: {names:?}"
    );
    assert!(
        names.contains(&"Circle".to_string()),
        "expected 'Circle' class in vb tags, got: {names:?}"
    );
}

#[test]
fn vb_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vb_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vb").ok() else {
        eprintln!("Skipping vb_calls: vb grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("vb").expect("vb calls query missing");
    let calls = collect_captures(&lang, VB_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "WriteLine" || c == "Area"),
        "expected 'WriteLine' or 'Area' call in vb sample, got: {calls:?}"
    );
}

#[test]
fn vb_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vb_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vb").ok() else {
        eprintln!("Skipping vb_complexity: vb grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("vb")
        .expect("vb complexity query missing");
    let complexity = collect_captures(&lang, VB_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in vb sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn vb_imports_finds_namespace_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vb_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vb").ok() else {
        eprintln!("Skipping vb_imports: vb grammar .so not found");
        return;
    };
    let query_str = loader.get_imports("vb").expect("vb imports query missing");
    let paths = collect_captures(&lang, VB_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("System")),
        "expected System namespace in vb import paths, got: {paths:?}"
    );
}

#[test]
fn vb_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping vb_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("vb").ok() else {
        eprintln!("Skipping vb_types: vb grammar .so not found");
        return;
    };
    let query_str = loader.get_types("vb").expect("vb types query missing");
    let refs = collect_captures(&lang, VB_SAMPLE, &query_str, "type");
    assert!(
        !refs.is_empty(),
        "expected type references in vb sample, got: {refs:?}"
    );
}

#[test]
fn vb_decorations_finds_attribute_list_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping vb_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "vb",
        VB_SAMPLE,
        &["<Obsolete("],
    );
}
