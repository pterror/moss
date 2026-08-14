//! Query fixture tests for fsharp.
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

const FSHARP_SAMPLE: &str = include_str!("fixtures/fsharp/sample.fs");

#[test]
fn fsharp_decorations_finds_attribute_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping fsharp_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "fsharp",
        FSHARP_SAMPLE,
        &["[<EntryPoint>]", "// Type definition"],
    );
}

#[test]
fn fsharp_calls_finds_application_and_qualified_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping fsharp_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("fsharp").ok() else {
        eprintln!("Skipping fsharp_calls: fsharp grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("fsharp")
        .expect("fsharp calls query missing");
    let calls = collect_captures(&lang, FSHARP_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"classify".to_string()),
        "expected 'classify' application call in fsharp calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"sumEvens".to_string()),
        "expected 'sumEvens' application call in fsharp calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"Sqrt".to_string()),
        "expected qualified 'Math.Sqrt' call to capture 'Sqrt' in fsharp calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, FSHARP_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"Math".to_string()),
        "expected 'Math' qualifier in fsharp calls, got: {qualifiers:?}"
    );
}
