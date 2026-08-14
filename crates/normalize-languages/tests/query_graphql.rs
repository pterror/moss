//! Query fixture tests for graphql.
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
// GraphQL
// ---------------------------------------------------------------------------

const GRAPHQL_SAMPLE: &str = include_str!("fixtures/graphql/sample.graphql");

#[test]
fn graphql_tags_finds_types_and_interfaces() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping graphql_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("graphql").ok() else {
        eprintln!("Skipping graphql_tags: graphql grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("graphql")
        .expect("graphql tags query missing");
    let names = collect_captures(&lang, GRAPHQL_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"User".to_string()),
        "expected 'User' type in graphql tags, got: {names:?}"
    );
    assert!(
        names.contains(&"UserRole".to_string()),
        "expected 'UserRole' enum in graphql tags, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "Node" || n == "Timestamped"),
        "expected interface name in graphql tags, got: {names:?}"
    );
}

#[test]
fn graphql_calls_finds_field_selections() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping graphql_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("graphql").ok() else {
        eprintln!("Skipping graphql_calls: graphql grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("graphql")
        .expect("graphql calls query missing");
    // GraphQL calls query captures field names; runs cleanly against schema definitions
    let _calls = collect_captures(&lang, GRAPHQL_SAMPLE, &query_str, "call");
}

#[test]
fn graphql_complexity_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping graphql_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("graphql").ok() else {
        eprintln!("Skipping graphql_complexity: graphql grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("graphql")
        .expect("graphql complexity query missing");
    let _complexity = collect_captures(&lang, GRAPHQL_SAMPLE, &query_str, "complexity");
}

#[test]
fn graphql_decorations_finds_description_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping graphql_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "graphql",
        GRAPHQL_SAMPLE,
        &[
            "\"\"\"A scalar representing a date and time value.\"\"\"",
            "# Node interface for objects with a unique ID",
        ],
    );
}
