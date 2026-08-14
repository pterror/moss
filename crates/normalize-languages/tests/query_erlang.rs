//! Query fixture tests for erlang.
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

const ERLANG_SAMPLE: &str = include_str!("fixtures/erlang/sample.erl");

#[test]
fn erlang_decorations_finds_attribute_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping erlang_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "erlang",
        ERLANG_SAMPLE,
        &["-module(", "%% Classify"],
    );
}

#[test]
fn erlang_calls_finds_local_and_remote_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping erlang_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("erlang").ok() else {
        eprintln!("Skipping erlang_calls: erlang grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("erlang")
        .expect("erlang calls query missing");
    let calls = collect_captures(&lang, ERLANG_SAMPLE, &query_str, "call");
    assert!(
        calls.contains(&"factorial".to_string()),
        "expected recursive local 'factorial' call in erlang calls, got: {calls:?}"
    );
    assert!(
        calls.contains(&"sort".to_string()),
        "expected remote 'lists:sort' call to capture 'sort' in erlang calls, got: {calls:?}"
    );
    let qualifiers = collect_captures(&lang, ERLANG_SAMPLE, &query_str, "call.qualifier");
    assert!(
        qualifiers.contains(&"lists".to_string()),
        "expected 'lists' qualifier (without trailing ':') in erlang calls, got: {qualifiers:?}"
    );
}
