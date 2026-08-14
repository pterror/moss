//! Query fixture tests for objc.
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
// Objective-C
// ---------------------------------------------------------------------------

const OBJC_SAMPLE: &str = include_str!("fixtures/objc/sample.m");

#[test]
fn objc_tags_finds_classes_and_functions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping objc_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("objc").ok() else {
        eprintln!("Skipping objc_tags: objc grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("objc").expect("objc tags query missing");
    let names = collect_captures(&lang, OBJC_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"Point".to_string()),
        "expected 'Point' class in objc tags, got: {names:?}"
    );
    assert!(
        names.contains(&"distance".to_string()),
        "expected 'distance' function in objc tags, got: {names:?}"
    );
}

#[test]
fn objc_calls_finds_function_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping objc_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("objc").ok() else {
        eprintln!("Skipping objc_calls: objc grammar .so not found");
        return;
    };
    let query_str = loader.get_calls("objc").expect("objc calls query missing");
    let calls = collect_captures(&lang, OBJC_SAMPLE, &query_str, "call");
    assert!(
        calls.iter().any(|c| c == "distance" || c == "classify"),
        "expected 'distance' or 'classify' call in objc sample, got: {calls:?}"
    );
}

#[test]
fn objc_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping objc_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("objc").ok() else {
        eprintln!("Skipping objc_complexity: objc grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("objc")
        .expect("objc complexity query missing");
    let complexity = collect_captures(&lang, OBJC_SAMPLE, &query_str, "complexity");
    assert!(
        complexity.len() >= 2,
        "expected at least 2 complexity nodes in objc sample, got {} ({complexity:?})",
        complexity.len()
    );
}

#[test]
fn objc_imports_finds_import_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping objc_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("objc").ok() else {
        eprintln!("Skipping objc_imports: objc grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("objc")
        .expect("objc imports query missing");
    let paths = collect_captures(&lang, OBJC_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("Foundation")),
        "expected Foundation in objc import paths, got: {paths:?}"
    );
}

#[test]
fn objc_types_finds_type_references() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping objc_types: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("objc").ok() else {
        eprintln!("Skipping objc_types: objc grammar .so not found");
        return;
    };
    let query_str = loader.get_types("objc").expect("objc types query missing");
    let refs = collect_captures(&lang, OBJC_SAMPLE, &query_str, "type");
    assert!(
        refs.iter()
            .any(|r| r == "NSString" || r == "NSLog" || r == "Point"),
        "expected type reference in objc sample, got: {refs:?}"
    );
}

#[test]
fn objc_decorations_finds_preproc_include_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping objc_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // In the ObjC grammar, #import is aliased into preproc_include (same rule handles both
    // #include and #import directives).
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "objc",
        OBJC_SAMPLE,
        &[
            "#import <Foundation/Foundation.h>",
            "// Initializes a Point with x and y coordinates.",
        ],
    );
}
