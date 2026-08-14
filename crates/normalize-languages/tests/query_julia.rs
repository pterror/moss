//! Query fixture tests for julia.
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

const JULIA_SAMPLE: &str = include_str!("fixtures/julia/sample.jl");

#[test]
fn julia_decorations_finds_macrocall_and_comment() {
    let Some(gdir) = require_grammar_dir() else {
        eprintln!("Skipping julia_decorations: run `cargo xtask build-grammars` first");
        return;
    };
    // In tree-sitter-julia, macro applications are macrocall_expression nodes.
    // @inline function classify(...) ... end parses as a macrocall_expression.
    assert_decorations_contains(
        &GrammarLoader::with_paths(vec![gdir]),
        "julia",
        JULIA_SAMPLE,
        &["# Classify a number", "@inline"],
    );
}
