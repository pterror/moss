//! Query fixture tests for svelte.
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
// Svelte
// ---------------------------------------------------------------------------

const SVELTE_SAMPLE: &str = include_str!("fixtures/svelte/sample.svelte");

#[test]
fn svelte_tags_finds_script_and_style_blocks() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping svelte_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("svelte").ok() else {
        eprintln!("Skipping svelte_tags: svelte grammar .so not found");
        return;
    };
    let query_str = loader
        .get_tags("svelte")
        .expect("svelte tags query missing");
    let names = collect_captures(&lang, SVELTE_SAMPLE, &query_str, "name");
    assert!(
        names.iter().any(|n| n == "script" || n == "style"),
        "expected 'script' or 'style' block tags in svelte sample, got: {names:?}"
    );
}

#[test]
fn svelte_calls_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping svelte_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("svelte").ok() else {
        eprintln!("Skipping svelte_calls: svelte grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("svelte")
        .expect("svelte calls query missing");
    // Svelte calls query is intentionally empty (JS in <script> is raw_text)
    let _calls = collect_captures(&lang, SVELTE_SAMPLE, &query_str, "call");
}

#[test]
fn svelte_complexity_query_runs_cleanly() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping svelte_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("svelte").ok() else {
        eprintln!("Skipping svelte_complexity: svelte grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("svelte")
        .expect("svelte complexity query missing");
    let _complexity = collect_captures(&lang, SVELTE_SAMPLE, &query_str, "complexity");
}
