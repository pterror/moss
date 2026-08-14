//! Query fixture tests for thrift.
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

const THRIFT_SAMPLE: &str = include_str!("fixtures/thrift/sample.thrift");

#[test]
fn thrift_decorations_finds_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping thrift_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "thrift",
        THRIFT_SAMPLE,
        &["// Thrift IDL sample file"],
    );
}

#[test]
fn thrift_imports_finds_include_path() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping thrift_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("thrift").ok() else {
        eprintln!("Skipping thrift_imports: thrift grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("thrift")
        .expect("thrift imports query missing");
    let paths = collect_captures(&lang, THRIFT_SAMPLE, &query_str, "import.path");
    assert!(
        paths.iter().any(|p| p.contains("shared.thrift")),
        "expected 'shared.thrift' include path in thrift imports, got: {paths:?}"
    );
}
